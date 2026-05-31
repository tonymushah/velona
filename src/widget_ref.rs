use std::{
    any::TypeId,
    fmt::Debug,
    marker::PhantomData,
    thread::{self, ThreadId},
};

use masonry::core::{Widget, WidgetId, WidgetMut, WidgetRef};
use winit::window::WindowId;

use crate::{
    app::{EventLoopEvent, el_event::EventProxyHandle},
    render_root::use_window_render_root_ref,
    window::handle::WindowHandle,
};

type EditFn = Box<dyn FnOnce(WidgetMut<dyn Widget>) + Send + Sync>;

type UseWidgetFn = Box<dyn FnOnce(WidgetRef<dyn Widget>) + Send + Sync>;

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

#[derive(Debug, Clone)]
pub struct VelonaWidgetRef<W>
where
    W: Widget + 'static,
{
    pub(crate) id: WidgetId,
    pub(crate) window: Option<Box<WindowHandle>>,
    pub(crate) phantom: PhantomData<W>,
    pub(crate) thread_id: ThreadId,
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
        F: FnOnce(WidgetMut<W>) + Send + Sync + 'static,
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
        F: FnOnce(WidgetMut<W>) -> R + Send + Sync + 'static,
        R: Send + Sync + 'static,
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
        F: FnOnce(WidgetRef<W>) + Send + Sync + 'static,
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
        F: FnOnce(WidgetRef<W>) -> R + Send + Sync + 'static,
        R: Send + Sync + 'static,
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
        Self {
            id: WidgetId::next(),
            window: None,
            phantom: PhantomData,
            thread_id: thread::current().id(),
        }
    }
}

unsafe impl<W> Send for VelonaWidgetRef<W> where W: Widget + 'static {}

unsafe impl<W> Sync for VelonaWidgetRef<W> where W: Widget + 'static {}

#[cfg(test)]
mod tests {

    use masonry::widgets::{Label, ZStack};

    use crate::utils::is_send_sync;

    use super::*;

    #[test]
    fn is_widget_ref_send_sync() {
        is_send_sync::<VelonaWidgetRef<ZStack>>();
    }
    #[test]
    fn is_edit_fn_event_send_sync() {
        is_send_sync::<EditWidgetFnEvent>();
    }
    #[test]
    fn is_use_fn_event_send_sync() {
        is_send_sync::<UseWidgetFnEvent>();
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
