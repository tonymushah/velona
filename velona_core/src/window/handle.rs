use std::sync::Weak;

use masonry_core::core::WidgetId;
use winit::window::{Window, WindowId};

use crate::{
    Manager,
    app::{
        self, AppHandle, EventLoopEvent,
        el_event::{EventProxyHandle, RegisterOnWindowDestroyHandler, RegisterWidgetActionHandler},
        proxy::AppEventLoopProxy,
    },
    window_event_handler::{HandlerFn, HandlerId, NoParamHandlerFn},
};

#[derive(Debug, Clone)]
pub struct WindowHandle {
    pub(crate) window: Weak<Window>,
    pub(crate) app_handle: AppHandle,
}

#[derive(Debug, thiserror::Error)]
pub enum WindowHandleActionError {
    #[error("The window has already closed")]
    WindowClosed,
    #[error("The app has already exited")]
    AppExited,
}

impl WindowHandle {
    fn use_raw_window<F, O>(&self, to_use: F) -> Option<O>
    where
        F: FnOnce(&Window) -> O,
    {
        self.window.upgrade().map(|window| to_use(&window))
    }
    pub fn id(&self) -> Result<WindowId, WindowHandleActionError> {
        self.use_raw_window(|window| window.id())
            .ok_or(WindowHandleActionError::WindowClosed)
    }
    pub fn request_redraw(&self) -> Result<(), WindowHandleActionError> {
        self.use_raw_window(|window| {
            window.request_redraw();
        })
        .ok_or(WindowHandleActionError::WindowClosed)
    }
    pub fn set_title(&self, title: &str) -> Result<(), WindowHandleActionError> {
        self.use_raw_window(|window| {
            window.set_title(title);
        })
        .ok_or(WindowHandleActionError::WindowClosed)
    }
}

/// Register event
impl WindowHandle {
    pub fn register_action_handler(
        &self,
        widget_id: WidgetId,
        handler_fn: HandlerFn,
    ) -> Result<HandlerId, WindowHandleActionError> {
        let handler_id = HandlerId::next();
        self.app_handle
            .send_event(EventLoopEvent::RegisterWidgetActionHandler(Box::new(
                RegisterWidgetActionHandler {
                    window_id: self.id()?,
                    widget_id,
                    handler_fn,
                    handler_id,
                },
            )))
            .map_err(|_| WindowHandleActionError::AppExited)?;

        Ok(handler_id)
    }
    pub fn remove_handler(&self, handler_id: HandlerId) -> Result<(), WindowHandleActionError> {
        self.app_handle
            .send_event(EventLoopEvent::UnregisterEventHandler(Box::new(
                app::el_event::UnregisterHandler {
                    handler_id,
                    window_id: Some(self.id()?),
                },
            )))
            .map_err(|_| WindowHandleActionError::AppExited)?;
        Ok(())
    }
    pub fn register_on_destroy_handler(
        &self,
        handler_fn: NoParamHandlerFn,
    ) -> Result<HandlerId, WindowHandleActionError> {
        let handler_id = HandlerId::next();
        self.app_handle
            .send_event(EventLoopEvent::RegisterOnWindowDestroy(Box::new(
                RegisterOnWindowDestroyHandler {
                    window_id: self.id()?,
                    handler_id,
                    handler: handler_fn,
                },
            )))
            .map_err(|_| WindowHandleActionError::AppExited)?;

        Ok(handler_id)
    }
}

impl app::el_event::EventProxyHandle for WindowHandle {
    fn get_proxy(&self) -> &AppEventLoopProxy {
        self.app_handle.get_proxy()
    }
}

impl Manager for WindowHandle {}
