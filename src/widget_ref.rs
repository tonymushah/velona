use std::marker::PhantomData;

use masonry::core::{Widget, WidgetId, WidgetMut, WidgetRef};
use winit::window::WindowId;

use crate::{
    app::{EventLoopEvent, el_event::EventProxyHandle},
    window::handle::WindowHandle,
};

type EditFn = Box<dyn FnOnce(WidgetMut<dyn Widget>) + Send + Sync>;

type UseWidgetFn = Box<dyn FnOnce(WidgetRef<dyn Widget>) + Send + Sync>;

pub(crate) struct EditWidgetFnEvent {
    pub(crate) window_id: WindowId,
    pub(crate) widget_id: WidgetId,
    pub(crate) edit_fn: EditFn,
}

pub(crate) struct UseWidgetFnEvent {
    pub(crate) window_id: WindowId,
    pub(crate) widget_id: WidgetId,
    pub(crate) use_fn: UseWidgetFn,
}

#[derive(Debug, Clone)]
pub struct VelonaWidgetRef<W>
where
    W: Widget + 'static,
{
    pub(crate) id: WidgetId,
    pub(crate) window: WindowHandle,
    pub(crate) phantom: PhantomData<W>,
}

#[derive(Debug, thiserror::Error)]
pub enum UseWidgetFromRefError {
    #[error("The window was already been closed")]
    WindowClosed,
    #[error("The app was already been exited")]
    AppExited,
    #[error("The widget was not found")]
    WidgetNotFound,
}

impl<W> VelonaWidgetRef<W>
where
    W: Widget + 'static,
{
    pub fn edit<F>(&self, edit_fn: F) -> Result<(), UseWidgetFromRefError>
    where
        F: FnOnce(WidgetMut<W>) + Send + Sync + 'static,
    {
        let window_id = {
            let Some(window) = self.window.window.upgrade() else {
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
        if self
            .window
            .send_event(EventLoopEvent::EditWidget(Box::new(event)))
            .is_err()
        {
            Err(UseWidgetFromRefError::AppExited)
        } else {
            Ok(())
        }
    }
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
    pub fn use_widget<F>(&self, use_fn: F) -> Result<(), UseWidgetFromRefError>
    where
        F: FnOnce(WidgetRef<W>) + Send + Sync + 'static,
    {
        let window_id = {
            let Some(window) = self.window.window.upgrade() else {
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
        if self
            .window
            .send_event(EventLoopEvent::UseWidget(Box::new(event)))
            .is_err()
        {
            Err(UseWidgetFromRefError::AppExited)
        } else {
            Ok(())
        }
    }
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
}

unsafe impl<W> Send for VelonaWidgetRef<W> where W: Widget + 'static {}

unsafe impl<W> Sync for VelonaWidgetRef<W> where W: Widget + 'static {}

#[cfg(test)]
mod tests {

    use masonry::widgets::ZStack;

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
}
