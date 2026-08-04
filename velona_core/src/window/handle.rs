use std::sync::Weak;

use masonry_core::app::RenderRoot;
use masonry_core::core::WidgetId;
use winit::{
    dpi::{self, PhysicalPosition, PhysicalSize},
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
    /// Due to some limitation on iOS
    /// (as [it](winit::window::Window::inner_position) can only be called on the main thread there),
    /// this function is not available there.
    ///
    /// We recommend using [`inner_position_async`](Self::inner_position_async) instead.
    ///
    /// See [`Window::inner_position`](winit::window::Window::inner_position) for more details.
    #[cfg(not(target_os = "ios"))]
    #[cfg_attr(docsrs, doc(not(target_os = "ios")))]
    pub fn inner_position(&self) -> Result<PhysicalPosition<i32>, WindowHandleActionError> {
        Ok(self.use_raw_window_now(|window| window.inner_position())??)
    }
    /// Returns the position of the top-left hand corner
    /// of the window’s client area
    /// relative to the top-left hand corner
    /// of the desktop.
    ///
    /// _Due to some limitation on iOS
    /// (as [it](winit::window::Window::inner_position) can only be called on the main thread there),
    /// this `async` function allow the [`inner_position`](winit::window::Window::inner_position) to be called on the main thread_.
    ///
    /// See [`Window::inner_position`](winit::window::Window::inner_position) for more details.
    pub async fn inner_position_async(
        &self,
    ) -> Result<PhysicalPosition<i32>, WindowHandleActionError> {
        let (sender, receiver) = futures_channel::oneshot::channel::<_>();
        self.use_winit_window_on_main(move |window| {
            let _ = sender.send(window.inner_position());
        })?;
        Ok(receiver
            .await
            .map_err(|_| WindowHandleActionError::AppExited)??)
    }
    /// Returns the position of the top-left hand corner of the window relative
    /// to the top-left hand corner of the desktop.
    ///
    /// Due to some limitation on iOS
    /// (as [it](winit::window::Window::outer_position) can only be called on the main thread there),
    /// this function is not available there.
    ///
    /// See [`Window::outer_position`](winit::window::Window::outer_position) for more details.
    #[cfg(not(target_os = "ios"))]
    #[cfg_attr(docsrs, doc(not(target_os = "ios")))]
    pub fn outer_position(&self) -> Result<PhysicalPosition<i32>, WindowHandleActionError> {
        Ok(self.use_raw_window_now(|window| window.outer_position())??)
    }

    /// Returns the position of the top-left hand corner of the window relative
    /// to the top-left hand corner of the desktop.
    ///
    /// _Due to some limitation on iOS
    /// (as [it](winit::window::Window::outer_position) can only be called on the main thread there),
    /// this `async` function allow the [`outerer_position`](winit::window::Window::outer_position) to be called on the main thread_.
    ///
    /// See [`Window::outer_position`](winit::window::Window::outer_position) for more details.
    pub async fn outer_position_async(
        &self,
    ) -> Result<PhysicalPosition<i32>, WindowHandleActionError> {
        let (sender, receiver) = futures_channel::oneshot::channel::<_>();
        self.use_winit_window_on_main(move |window| {
            let _ = sender.send(window.outer_position());
        })?;
        Ok(receiver
            .await
            .map_err(|_| WindowHandleActionError::AppExited)??)
    }

    /// Modifies the position of the window.
    ///
    /// See [`Window::set_outer_position`](winit::window::Window::set_outer_position) for more details.
    pub fn set_outer_position<P>(&self, position: P) -> Result<(), WindowHandleActionError>
    where
        P: Into<dpi::Position> + Send + 'static,
    {
        self.use_winit_window_on_main(move |window| {
            window.set_outer_position(position);
        })
    }

    /// Request the new size for the window.
    ///
    /// Due to some limitation on iOS
    /// (as [it](winit::window::Window::inner_size) can only be called on the main thread there),
    /// this function is not available there.
    ///
    /// We recommend using [`inner_size_async`](Self::inner_size_async) instead.
    ///
    /// See [`Window::inner_size`](winit::window::Window::inner_size) for more details.
    #[cfg(not(target_os = "ios"))]
    #[cfg_attr(docsrs, doc(not(target_os = "ios")))]
    pub fn inner_size(&self) -> Result<PhysicalSize<u32>, WindowHandleActionError> {
        self.use_raw_window_now(|window| window.inner_size())
    }
    /// Returns the physical size of the window’s client area.
    ///
    /// _Due to some limitation on iOS
    /// (as [it](winit::window::Window::inner_size) can only be called on the main thread there),
    /// this function is `async`_.
    ///
    /// See [`Window::inner_size`](winit::window::Window::inner_size) for more details.
    pub async fn inner_size_async(&self) -> Result<PhysicalSize<u32>, WindowHandleActionError> {
        let (sender, receiver) = futures_channel::oneshot::channel::<_>();
        self.use_winit_window_on_main(move |window| {
            let _ = sender.send(window.inner_size());
        })?;
        receiver
            .await
            .map_err(|_| WindowHandleActionError::AppExited)
    }
    /// Request the new size for the window.
    ///
    /// See [`Window::request_inner_size`](winit::window::Window::request_inner_size) for more details.
    ///
    /// _You can safely drop the future if you don't need it since it is just a [`futures_channel::oneshot::Receiver`]
    /// awaiting for [`Window::request_inner_size`](winit::window::Window::request_inner_size) return value._
    pub async fn request_inner_size<S>(
        &self,
        size: S,
    ) -> Result<Option<PhysicalSize<u32>>, WindowHandleActionError>
    where
        S: Into<dpi::Size> + Send + 'static,
    {
        let (sender, receiver) = futures_channel::oneshot::channel::<_>();
        self.use_winit_window_on_main(move |window| {
            let _ = sender.send(window.request_inner_size(size));
        })?;
        receiver
            .await
            .map_err(|_| WindowHandleActionError::AppExited)
    }
    /// Returns the physical size of the entire window.
    ///
    /// Due to some limitation on iOS
    /// (as [it](winit::window::Window::outer_size) can only be called on the main thread there),
    /// this function is not available there.
    ///
    /// We recommend using [`outer_size_async`](Self::outer_size_async) instead.
    ///
    /// See [`Window::outer_size`](winit::window::Window::outer_size) for more details.
    #[cfg(not(target_os = "ios"))]
    #[cfg_attr(docsrs, doc(not(target_os = "ios")))]
    pub fn outer_size(&self) -> Result<PhysicalSize<u32>, WindowHandleActionError> {
        self.use_raw_window_now(|window| window.outer_size())
    }
    /// Returns the physical size of the entire window.
    ///
    /// _Due to some limitation on iOS
    /// (as [it](winit::window::Window::outer_size) can only be called on the main thread there),
    /// this function is `async`_.
    ///
    /// See [`Window::outer_size`](winit::window::Window::outer_size) for more details.
    pub async fn outer_size_async(&self) -> Result<PhysicalSize<u32>, WindowHandleActionError> {
        let (sender, receiver) = futures_channel::oneshot::channel::<_>();
        self.use_winit_window_on_main(move |window| {
            let _ = sender.send(window.outer_size());
        })?;
        receiver
            .await
            .map_err(|_| WindowHandleActionError::AppExited)
    }
    /// Sets a minimum dimension size for the window.
    ///
    /// See [`Window::set_min_size`](winit::window::Window::set_min_size) for more details.
    pub fn set_min_inner_size<S>(&self, min_size: Option<S>) -> Result<(), WindowHandleActionError>
    where
        S: Into<dpi::Size> + Send + 'static,
    {
        self.use_winit_window_on_main(move |window| {
            window.set_min_inner_size(min_size);
        })
    }

    /// Sets a maximum dimension size for the window.
    ///
    /// See [`Window::set_max_inner_size`](winit::window::Window::max_inner_size) for more details.
    pub fn set_max_inner_size<S>(&self, max_size: Option<S>) -> Result<(), WindowHandleActionError>
    where
        S: Into<dpi::Size> + Send + 'static,
    {
        self.use_winit_window_on_main(move |window| {
            window.set_max_inner_size(max_size);
        })
    }
    /// Returns window resize increments if any were set.
    ///
    /// See [`Window::resize_increments`](winit::window::Window::max_inner_size) for more details.
    pub fn resize_increments(&self) -> Result<Option<PhysicalSize<u32>>, WindowHandleActionError> {
        self.use_raw_window_now(|window| window.resize_increments())
    }
    /// Sets window resize increments.
    ///
    /// See [`Window::set_resize_increments`](winit::window::Window::set_resize_increments) for more details.
    pub fn set_resize_increment<S>(
        &self,
        increments: Option<S>,
    ) -> Result<(), WindowHandleActionError>
    where
        S: Into<dpi::Size> + Send + 'static,
    {
        self.use_winit_window_on_main(move |window| {
            window.set_resize_increments(increments);
        })
    }
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
