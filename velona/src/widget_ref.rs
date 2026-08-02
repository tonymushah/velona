use std::{
    any::TypeId,
    backtrace::{Backtrace, BacktraceStatus},
    fmt::Debug,
    marker::PhantomData,
    thread::{self, ThreadId},
};

use imaging::kurbo::Affine;
use masonry::core::{Widget, WidgetId, WidgetMut, WidgetRef};
use winit::window::WindowId;

use crate::{
    app::{EventLoopEvent, el_event::EventProxyHandle},
    render_root::use_window_render_root_ref,
    utils::ConsumeResult,
    window::handle::WindowHandle,
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
    W: Widget + 'static,
{
    pub(crate) id: WidgetId,
    pub(crate) window: Option<Box<WindowHandle>>,
    pub(crate) phantom: PhantomData<W>,
    pub(crate) thread_id: ThreadId,
}

impl<W> Clone for VelonaWidgetRef<W>
where
    W: Widget + 'static,
{
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            window: self.window.clone(),
            phantom: self.phantom,
            thread_id: thread::current().id(),
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

#[derive(Debug, thiserror::Error)]
pub enum EditWidgetLocalError {
    #[error("You tried to edit a widget outside the componnent three")]
    OutsideTree,
    #[error("The widget specified is not found")]
    WidgetNotFound,
    #[error("The tree was dropped or mutably used somewhere")]
    UnaccessibleTree,
    #[error("Widget found but the type is not correct [{:?} != {:?}]", .original_cast, .current_cast)]
    InvalidWidgetCast {
        original_cast: TypeId,
        current_cast: TypeId,
    },
    #[error("You are trying to edit a `VelonaWidgetRef` outside the main thread")]
    OutsideMainThread,
}

impl<A> ConsumeResult for Result<A, EditWidgetLocalError> {
    /// [`log::error`] the [`EditWidgetLocalError`]
    /// and [`log::trace`] the backtrace if available
    fn consume_with_log_err(self) {
        if let Err(err) = self {
            log::error!("Cannot edit the widget locally => {err}");

            let backtrace = Backtrace::capture();
            if backtrace.status() == BacktraceStatus::Captured {
                log::trace!("Backtrace: \n{backtrace}");
            }
        }
    }
}

// #[cfg_attr(feature = "hotpath", hotpath::measure_all)]
impl<W> VelonaWidgetRef<W>
where
    W: Widget + 'static,
{
    pub(crate) fn disarm(mut self) -> Self {
        self.window.take();
        self
    }
    /// Edit the current widget right now.
    ///
    /// This function will always fail if called outside the main thread.
    pub fn edit_local_now<F, O>(&self, edit_fn: F) -> Result<O, EditWidgetLocalError>
    where
        F: FnOnce(WidgetMut<W>) -> O,
    {
        if self.thread_id != thread::current().id() {
            return Err(EditWidgetLocalError::OutsideMainThread);
        }
        let weak_root = use_window_render_root_ref().ok_or(EditWidgetLocalError::OutsideTree)?;
        weak_root
            .use_inner_render_root_mut(|render_root| {
                if render_root.tree.has_widget(self.id) {
                    render_root.tree.edit_widget(self.id, |mut widget_mut| {
                        let Some(widget_mut) = widget_mut.try_downcast::<W>() else {
                            return Err(EditWidgetLocalError::InvalidWidgetCast {
                                original_cast: TypeId::of::<W>(),
                                current_cast: widget_mut.widget.type_id(),
                            });
                        };
                        Ok(edit_fn(widget_mut))
                    })
                } else {
                    Err(EditWidgetLocalError::WidgetNotFound)
                }
            })
            .ok_or(EditWidgetLocalError::UnaccessibleTree)?
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
            edit_fn: Box::new(|mut widget_mut| {
                let Some(widget_mut) = widget_mut.try_downcast::<W>() else {
                    log::warn!("Invalid cast {}", widget_mut.widget.short_type_name());
                    return;
                };
                edit_fn(widget_mut);
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
                let Some(widget_ref) = widget_ref.downcast::<W>() else {
                    log::warn!("Invalid cast {}", widget_ref.inner().short_type_name());
                    return;
                };
                use_fn(widget_ref);
            }),
        };
        self.send_event(EventLoopEvent::UseWidget(Box::new(event)))
    }
    /// Similar to [`Self::edit`] but allows you to return a value.
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
        use masonry::widgets::SizedBox;

        Self {
            id: SizedBox::empty().prepare().id(),
            window: None,
            phantom: PhantomData,
            thread_id: thread::current().id(),
        }
    }
    /// Set the [`WidgetId`] that this reference belongs too
    pub fn set_id(&mut self, widget_id: WidgetId) {
        self.id = widget_id;
    }
    /// Change the widget signature
    pub fn cast<W1: Widget + 'static>(self) -> VelonaWidgetRef<W1> {
        VelonaWidgetRef::<W1> {
            phantom: PhantomData::<W1>,
            id: self.id,
            window: self.window,
            thread_id: self.thread_id,
        }
    }
    /// Queues a callback that will be called with a [`WidgetMut`] for this widget.
    ///
    /// Unlike [`edit`](Self::edit), the callbacks will be run in the order they were submitted during the mutate pass.
    ///
    /// You might never use this thing, _since [`edit`](Self::edit) is what you use most of the time_
    /// but who knows?
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

unsafe impl<W> Send for VelonaWidgetRef<W> where W: Widget + 'static {}

unsafe impl<W> Sync for VelonaWidgetRef<W> where W: Widget + 'static {}

#[cfg(test)]
mod tests {

    use masonry::widgets::{Label, ZStack};

    use crate::utils::{is_send, is_send_sync};

    use super::*;

    #[test]
    fn is_widget_ref_send_sync() {
        is_send_sync::<VelonaWidgetRef<ZStack>>();
    }
    #[test]
    fn is_edit_fn_event_send_sync() {
        is_send::<EditWidgetFnEvent>();
    }
    #[test]
    fn is_use_fn_event_send_sync() {
        is_send::<UseWidgetFnEvent>();
    }
    #[test]
    fn test_threading_test() {
        let empty = VelonaWidgetRef::<Label>::create_empty();
        assert!(matches!(
            empty.edit_local_now(|_| {}),
            Err(EditWidgetLocalError::OutsideTree)
        ));
        thread::spawn(move || {
            assert!(matches!(
                empty.edit_local_now(|_| {}),
                Err(EditWidgetLocalError::OutsideMainThread)
            ));
        })
        .join()
        .unwrap();
    }
}
