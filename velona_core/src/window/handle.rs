use std::sync::Weak;

use masonry_core::app::RenderRoot;
use masonry_core::core::WidgetId;
use winit::{
    dpi::PhysicalPosition,
    window::{Window, WindowId},
};

use crate::{
    Manager,
    app::{
        self, AppHandle, EventLoopEvent,
        el_event::{
            EventProxyHandle, RegisterOnWindowDestroyHandler, RegisterWidgetActionHandler,
            UseWindowRenderRootOnMain, UseWinitWindowOnMain,
        },
        proxy::{AppEventLoopProxy, AppProxySendError},
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
    #[error("Not supported operation")]
    NotSupported,
}

impl From<AppProxySendError> for WindowHandleActionError {
    fn from(_: AppProxySendError) -> Self {
        Self::AppExited
    }
}

impl From<winit::error::NotSupportedError> for WindowHandleActionError {
    fn from(_: winit::error::NotSupportedError) -> Self {
        Self::NotSupported
    }
}

impl WindowHandle {
    /// Use the underlying window handle right now.
    pub fn use_raw_window_now<F, O>(&self, to_use: F) -> Result<O, WindowHandleActionError>
    where
        F: FnOnce(&Window) -> O,
    {
        self.window
            .upgrade()
            .map(|window| to_use(&window))
            .ok_or(WindowHandleActionError::WindowClosed)
    }
    /// Use the underlying render root of the current window.
    pub fn use_render_root<U>(&self, use_fn: U) -> Result<(), WindowHandleActionError>
    where
        U: FnOnce(&mut RenderRoot) + Send + 'static,
    {
        let window_id = self.id()?;
        self.send_event(EventLoopEvent::UseWindowRenderRoot(Box::new(
            UseWindowRenderRootOnMain {
                window_id,
                use_fn: Box::new(use_fn),
            },
        )))?;
        Ok(())
    }
    /// Use the underlying render root of the current window.
    pub fn use_winit_window_on_main<U>(&self, use_fn: U) -> Result<(), WindowHandleActionError>
    where
        U: FnOnce(&Window) + Send + 'static,
    {
        let window_id = self.id()?;
        self.send_event(EventLoopEvent::UseWinitWindow(Box::new(
            UseWinitWindowOnMain {
                window_id,
                use_fn: Box::new(use_fn),
            },
        )))?;
        Ok(())
    }
}

/// Base window functions
impl WindowHandle {
    /// Return the unique identifier of the window
    pub fn id(&self) -> Result<WindowId, WindowHandleActionError> {
        self.use_raw_window_now(|window| window.id())
    }
    /// Returns the scale factor that can be used to map logical pixels to physical pixels, and vice versa.
    ///
    /// See [`Window::scale_factor`](winit::window::Window::scale_factor) for more details.
    pub fn scale_factor(&self) -> Result<f64, WindowHandleActionError> {
        self.use_raw_window_now(|window| window.scale_factor())
    }
    /// See [`Window::request_redraw`](winit::window::Window::request_redraw) for more details.
    pub fn request_redraw(&self) -> Result<(), WindowHandleActionError> {
        self.use_raw_window_now(|window| {
            window.request_redraw();
        })
    }
    /// See [`Window::pre_present_notify`](winit::window::Window::pre_present_notify) for more details.
    pub fn pre_present_notify(&self) -> Result<(), WindowHandleActionError> {
        self.use_raw_window_now(|window| {
            window.pre_present_notify();
        })
    }
    /// Reset the dead key state of the keyboard.
    ///
    /// See [`Window::reset_dead_keys`](winit::window::Window::reset_dead_keys) for more details.
    pub fn reset_dead_keys(&self) -> Result<(), WindowHandleActionError> {
        self.use_raw_window_now(|window| {
            window.reset_dead_keys();
        })
    }
}

/// Position and size functions
impl WindowHandle {
    /// Returns the position of the top-left hand corner
    /// of the window’s client area
    /// relative to the top-left hand corner
    /// of the desktop.
    ///
    /// See [`Window::inner_positions`](winit::window::Window::inner_positions) for more details.
    pub fn inner_position(&self) -> Result<PhysicalPosition<i32>, WindowHandleActionError> {
        Ok(self.use_raw_window_now(|window| window.inner_position())??)
    }
    /// Returns the position of the top-left hand corner of the window relative
    /// to the top-left hand corner of the desktop.
    ///
    /// See [`Window::outer_position`](winit::window::Window::outer_position) for more details.
    pub fn outer_position(&self) -> Result<PhysicalPosition<i32>, WindowHandleActionError> {
        Ok(self.use_raw_window_now(|window| window.outer_position())??)
    }
    pub fn set_outer_position(&self);
    pub fn inner_size(&self);
    pub fn request_inner_size(&self);
    pub fn outer_size(&self);
    pub fn set_min_inner_size(&self);
    pub fn set_max_inner_size(&self);
    pub fn resize_increments(&self);
    pub fn set_resize_increment(&self);
}

/// Misc. attribute functions
impl WindowHandle {
    pub fn set_title(&self, title: &str) -> Result<(), WindowHandleActionError> {
        self.use_raw_window_now(|window| {
            window.set_title(title);
        })
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
            )))?;

        Ok(handler_id)
    }
    pub fn remove_handler(&self, handler_id: HandlerId) -> Result<(), WindowHandleActionError> {
        self.app_handle
            .send_event(EventLoopEvent::UnregisterEventHandler(Box::new(
                app::el_event::UnregisterHandler {
                    handler_id,
                    window_id: Some(self.id()?),
                },
            )))?;
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
            )))?;

        Ok(handler_id)
    }
}

impl app::el_event::EventProxyHandle for WindowHandle {
    fn get_proxy(&self) -> &AppEventLoopProxy {
        self.app_handle.get_proxy()
    }
}

impl Manager for WindowHandle {}
