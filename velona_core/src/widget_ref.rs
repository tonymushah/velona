use std::{fmt::Debug, marker::PhantomData};

use imaging::kurbo::{Affine, Point};
use masonry_core::core::ErasedAction;
use masonry_core::core::FromDynWidget;
use masonry_core::core::PropertyStackId;
use masonry_core::core::{LayerType, NewWidget, Widget, WidgetId, WidgetMut, WidgetRef};
use winit::window::WindowId;

use crate::{
    app::{EventLoopEvent, proxy::EventProxyHandle},
    utils::ConsumeResult,
    window::{
        event_listener::HandlerId,
        handle::{WindowHandle, WindowHandleActionError},
    },
};

type EditFn = Box<dyn FnOnce(WidgetMut<dyn Widget>) + Send>;

type UseWidgetFn = Box<dyn FnOnce(WidgetRef<dyn Widget>) + Send>;

pub(crate) struct EditWidgetFnEvent {
    pub(crate) window_id: WindowId,
    pub(crate) widget_id: WidgetId,
    pub(crate) edit_fn: EditFn,
}

impl Debug for EditWidgetFnEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EditWidgetFnEvent")
            .field("window_id", &self.window_id)
            .field("widget_id", &self.widget_id)
            .field("edit_fn", &())
            .finish()
    }
}

pub(crate) struct UseWidgetFnEvent {
    pub(crate) window_id: WindowId,
    pub(crate) widget_id: WidgetId,
    pub(crate) use_fn: UseWidgetFn,
}

impl Debug for UseWidgetFnEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UseWidgetFnEvent")
            .field("window_id", &self.window_id)
            .field("widget_id", &self.widget_id)
            .field("use_fn", &())
            .finish()
    }
}

#[derive(Debug)]
pub struct VelonaWidgetRef<W>
where
    W: Widget + FromDynWidget + ?Sized,
{
    pub(crate) id: WidgetId,
    pub(crate) window: Option<Box<WindowHandle>>,
    pub(crate) phantom: PhantomData<W>,
}

impl<W> Clone for VelonaWidgetRef<W>
where
    W: Widget + FromDynWidget + ?Sized,
{
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            window: self.window.clone(),
            phantom: self.phantom,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UseWidgetFromRefError {
    #[error("The window was already been closed")]
    WindowClosed,
    #[error("The app was already been exited")]
    AppExited,
    #[error("The widget was not found")]
    WidgetNotFound,
    #[error("No `WindowHandle` is provided")]
    NoWindowHandleProvided,
}

impl<T> ConsumeResult for Result<T, UseWidgetFromRefError> {
    #[track_caller]
    fn consume_with_log_err(self) {
        if let Err(err) = self {
            log::error!(
                "cannot use widget from a velona ref ({err}) at {}",
                std::panic::Location::caller()
            );
        }
    }
}

// #[cfg_attr(feature = "hotpath", hotpath::measure_all)]
impl<W> VelonaWidgetRef<W>
where
    W: Widget + FromDynWidget + ?Sized,
{
    /// Change the widget signature
    pub fn cast<W1: Widget + FromDynWidget + ?Sized>(self) -> VelonaWidgetRef<W1> {
        VelonaWidgetRef::<W1> {
            phantom: PhantomData::<W1>,
            id: self.id,
            window: self.window,
        }
    }
    /// Set the [`WidgetId`] that this reference belongs too
    pub fn set_id(&mut self, widget_id: WidgetId) {
        self.id = widget_id;
    }
    fn send_event(&self, event: EventLoopEvent) -> Result<(), UseWidgetFromRefError> {
        if self
            .window
            .as_ref()
            .ok_or(UseWidgetFromRefError::NoWindowHandleProvided)?
            .send_event(event)
            .is_err()
        {
            Err(UseWidgetFromRefError::AppExited)
        } else {
            Ok(())
        }
    }
    /// Edit the underlying widget "safely".
    ///
    /// Unlike the [`Self::edit_local_now`], this function is safe to use between threads.
    /// If you want to get a return value, use [`Self::edit_with_return`].
    #[track_caller]
    pub fn edit<F>(&self, edit_fn: F) -> Result<(), UseWidgetFromRefError>
    where
        F: FnOnce(WidgetMut<W>) + Send + 'static,
    {
        let window_id = {
            let Some(window) = self
                .window
                .as_ref()
                .ok_or(UseWidgetFromRefError::NoWindowHandleProvided)?
                .window
                .upgrade()
            else {
                return Err(UseWidgetFromRefError::AppExited);
            };
            window.id()
        };
        let event = EditWidgetFnEvent {
            widget_id: self.id,
            window_id,
            edit_fn: Box::new(move |mut widget_mut| {
                if let Some(widget_mut) = widget_mut.try_downcast::<W>() {
                    edit_fn(widget_mut);
                } else {
                    log::warn!(
                        "Invalid cast {} for edit at {}",
                        widget_mut.widget.short_type_name(),
                        std::panic::Location::caller()
                    );
                }
            }),
        };
        self.send_event(EventLoopEvent::EditWidget(Box::new(event)))
    }
    /// Similar to [`Self::edit`] but allows you to return a value.
    pub async fn edit_with_return<F, R>(&self, edit_fn: F) -> Result<R, UseWidgetFromRefError>
    where
        F: FnOnce(WidgetMut<W>) -> R + Send + 'static,
        R: Send + 'static,
    {
        let (sender, receiver) = futures_channel::oneshot::channel::<R>();
        self.edit(move |widget_mut| {
            let _ = sender.send(edit_fn(widget_mut));
        })?;
        if let Ok(res) = receiver.await {
            Ok(res)
        } else {
            Err(UseWidgetFromRefError::WidgetNotFound)
        }
    }
    /// Use the underlying widget "safely".
    ///
    /// If you want to get a return value, use [`Self::use_with_return`].
    #[track_caller]
    pub fn use_widget<F>(&self, use_fn: F) -> Result<(), UseWidgetFromRefError>
    where
        F: FnOnce(WidgetRef<W>) + Send + 'static,
    {
        let window_id = {
            let Some(window) = self
                .window
                .as_ref()
                .ok_or(UseWidgetFromRefError::NoWindowHandleProvided)?
                .window
                .upgrade()
            else {
                return Err(UseWidgetFromRefError::AppExited);
            };
            window.id()
        };
        let event = UseWidgetFnEvent {
            widget_id: self.id,
            window_id,
            use_fn: Box::new(|widget_ref| {
                if let Some(widget_ref) = widget_ref.downcast::<W>() {
                    use_fn(widget_ref)
                } else {
                    log::warn!(
                        "Invalid cast {} for use at {}",
                        widget_ref.inner().short_type_name(),
                        std::panic::Location::caller()
                    );
                }
            }),
        };
        self.send_event(EventLoopEvent::UseWidget(Box::new(event)))
    }
    /// Similar to [`Self::use_widget`] but allows you to return a value.
    pub async fn use_with_return<F, R>(&self, use_fn: F) -> Result<R, UseWidgetFromRefError>
    where
        F: FnOnce(WidgetRef<W>) -> R + Send + 'static,
        R: Send + 'static,
    {
        let (sender, receiver) = futures_channel::oneshot::channel::<R>();
        self.use_widget(move |widget_ref| {
            let _ = sender.send(use_fn(widget_ref));
        })?;
        if let Ok(res) = receiver.await {
            Ok(res)
        } else {
            Err(UseWidgetFromRefError::WidgetNotFound)
        }
    }
    #[cfg(test)]
    #[cfg_attr(docsrs, doc(cfg(test)))]
    /// Create an empty reference for testing purposes
    pub fn create_empty() -> Self {
        use masonry_raw_box::RawBox;

        Self {
            id: RawBox::empty().prepare().id(),
            window: None,
            phantom: PhantomData,
        }
    }
    /// Queues a callback that will be called with a [`WidgetMut`] for this widget.
    ///
    /// Unlike [`edit`](Self::edit), the callbacks will be run in the order they were submitted during the mutate pass.
    ///
    /// You might never use this thing, _since [`edit`](Self::edit) is what you use most of the time_
    /// but who knows?
    #[track_caller]
    pub fn mutate_later<Fn>(&self, mutate_fn: Fn) -> Result<(), UseWidgetFromRefError>
    where
        Fn: FnOnce(WidgetMut<'_, W>) + Send + 'static,
    {
        self.edit(move |mut widget_mut| {
            widget_mut
                .ctx
                .mutate_later(widget_mut.id(), move |mut this| {
                    if let Some(this) = this.try_downcast::<W>() {
                        mutate_fn(this);
                    } else {
                        log::error!("Invalid downcast for mutate later");
                    }
                });
        })
    }
    /// Similar to [`mutate_later`](Self::mutate_later) but with a return value.
    pub async fn mutate_later_with_output<Fn, O>(
        &self,
        mutate_fn: Fn,
    ) -> Result<O, UseWidgetFromRefError>
    where
        Fn: FnOnce(WidgetMut<'_, W>) -> O + Send + 'static,
        O: Send + 'static,
    {
        let (tx, rx) = futures_channel::oneshot::channel::<O>();
        self.mutate_later(move |this| {
            let _ = tx.send(mutate_fn(this));
        })?;
        if let Ok(res) = rx.await {
            Ok(res)
        } else {
            Err(UseWidgetFromRefError::WidgetNotFound)
        }
    }

    pub async fn create_attached_layer<L, LFn>(
        &self,
        layer_type: LayerType,
        layer: LFn,
        position: Point,
    ) -> Result<WidgetId, UseWidgetFromRefError>
    where
        LFn: FnOnce() -> NewWidget<L> + Send + 'static,
        L: Widget + 'static,
    {
        self.edit_with_return(move |mut this| {
            let layer = layer();
            let layer_id = layer.id();
            this.ctx.create_attached_layer(layer_type, layer, position);
            layer_id
        })
        .await
    }
    #[track_caller]
    pub fn into_dyn(self) -> VelonaWidgetRef<dyn Widget> {
        VelonaWidgetRef {
            phantom: PhantomData::<dyn Widget>,
            id: self.id,
            window: self.window,
        }
    }
    /// Checks if the current widget is present in the tree.
    pub async fn is_present(&self) -> bool {
        if let Some(window) = self.window.as_ref() {
            window.has_widget(self.id).await.unwrap_or_default()
        } else {
            false
        }
    }

    /// Sets this widget as the [focused widget](masonry_core::doc::masonry_concepts#text-focus)
    /// and the [focus anchor](masonry_core::doc::masonry_concepts#focus-anchor).
    pub fn set_focus(&self) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        if let Err(err) = window.focus_on(Some(self.id)) {
            log::error!("cannot set focus on the current widget: {err}");
        }
    }
    /// Sets this widget as the [focus fallback](masonry_core::doc::masonry_concepts#focus-fallback).
    pub fn set_focus_callback(&self) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        if let Err(err) = window.set_focus_callback(Some(self.id)) {
            log::error!("cannot set focus callback on the current widget: {err}");
        }
    }
    /// Sets which property stack this widget uses for property resolution.
    pub fn set_property_stack_id(
        &self,
        property_stack_id: PropertyStackId,
    ) -> Result<(), UseWidgetFromRefError> {
        self.edit(move |mut this| {
            this.ctx.set_property_stack(property_stack_id);
        })
    }
    /// Sets the contents of the platform clipboard.
    ///
    /// For example, text widgets should call this for "cut" and "copy" user interactions.
    /// Note that we currently don't support the "Primary" selection buffer on X11/Wayland.
    pub fn set_clipboard(&self, contents: String) -> Result<(), UseWidgetFromRefError> {
        self.edit(move |mut this| {
            this.ctx.set_clipboard(contents);
        })
    }
    /// Sets the local transform for this widget.
    ///
    /// This maps this widget's border-box coordinate space
    /// to the parent's border-box coordinate space.
    ///
    /// It behaves similarly as CSS transforms.
    pub fn set_transform(&self, transform: Affine) -> Result<(), UseWidgetFromRefError> {
        self.edit(move |mut this| {
            this.ctx.set_transform(transform);
        })
    }
}

#[derive(derive_more::Debug, thiserror::Error)]
pub enum ListenToWidgetActionFromRefError {
    #[error("This ref doesn't not have a internal WindowHandle")]
    WindowHandleNotPresent,
    #[error(transparent)]
    WindowAction(#[from] WindowHandleActionError),
}

// ----- Event Handlers ------
impl<W> VelonaWidgetRef<W>
where
    W: Widget + FromDynWidget + ?Sized,
{
    pub fn listen_to_widget_action_erased<H>(
        &self,
        handler: H,
    ) -> Result<HandlerId, ListenToWidgetActionFromRefError>
    where
        H: Fn(&ErasedAction) + Send + 'static,
    {
        Ok(self
            .window
            .as_ref()
            .ok_or(ListenToWidgetActionFromRefError::WindowHandleNotPresent)?
            .register_action_handler(self.id, Box::new(handler))?)
    }
}

impl<W> VelonaWidgetRef<W>
where
    W: Widget + 'static,
{
    pub fn listen_to_widget_action<H>(
        &self,
        handler: H,
    ) -> Result<HandlerId, ListenToWidgetActionFromRefError>
    where
        H: Fn(&W::Action) + Send + 'static,
    {
        Ok(self
            .window
            .as_ref()
            .ok_or(ListenToWidgetActionFromRefError::WindowHandleNotPresent)?
            .register_action_handler(
                self.id,
                Box::new(move |action| {
                    if let Some(action) = action.downcast_ref::<W::Action>() {
                        handler(action);
                    } else {
                        log::warn!("Invalid widget action cast");
                    }
                }),
            )?)
    }
}

unsafe impl<W> Send for VelonaWidgetRef<W> where W: Widget + FromDynWidget + ?Sized {}

unsafe impl<W> Sync for VelonaWidgetRef<W> where W: Widget + FromDynWidget + ?Sized {}

#[cfg(test)]
mod tests {

    use masonry::widgets::ZStack;

    use crate::utils::{is_send, is_send_sync};

    use super::*;

    #[test]
    fn is_widget_ref_send_sync() {
        is_send_sync::<VelonaWidgetRef<ZStack>>();
        is_send_sync::<VelonaWidgetRef<dyn Widget>>();
    }
    #[test]
    fn is_edit_fn_event_send_sync() {
        is_send::<EditWidgetFnEvent>();
    }
    #[test]
    fn is_use_fn_event_send_sync() {
        is_send::<UseWidgetFnEvent>();
    }
}
