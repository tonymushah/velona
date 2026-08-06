use std::{marker::PhantomData, sync::Weak};

use futures_channel::oneshot;
use masonry_core::app::RenderRoot;
use masonry_core::core::Widget;
use masonry_core::core::WidgetId;
use winit::{
    dpi::{self, PhysicalPosition, PhysicalSize},
    monitor::MonitorHandle,
    window::{
        Cursor, CursorGrabMode, Fullscreen, Icon, ImePurpose, ResizeDirection, Theme,
        UserAttentionType, Window, WindowButtons, WindowId, WindowLevel,
    },
};

use crate::{
    Manager,
    app::{
        self, AppHandle, EventLoopEvent,
        el_event::{
            EventProxyHandle, GetWindowChildReactiveOwner, RegisterOnWindowDestroyHandler,
            RegisterWidgetActionHandler, UseWindowRenderRootOnMain, UseWinitWindowOnMain,
        },
        proxy::{AppEventLoopProxy, AppProxySendError},
    },
    widget_ref::VelonaWidgetRef,
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
    #[error("The requested operation is not supported by Winit")]
    NotSupported,
    #[error("The operation was ignored.")]
    Ignored,
    #[error(transparent)]
    Os(#[from] winit::error::OsError),
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

impl From<winit::error::ExternalError> for WindowHandleActionError {
    fn from(value: winit::error::ExternalError) -> Self {
        match value {
            winit::error::ExternalError::NotSupported(_) => WindowHandleActionError::NotSupported,
            winit::error::ExternalError::Ignored => WindowHandleActionError::Ignored,
            winit::error::ExternalError::Os(os_error) => WindowHandleActionError::Os(os_error),
        }
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
    /// Use the underlying render root of the current window with a return value.
    pub async fn use_render_root_with_return<U, O>(
        &self,
        use_fn: U,
    ) -> Result<O, WindowHandleActionError>
    where
        U: FnOnce(&mut RenderRoot) -> O + Send + 'static,
        O: Send + 'static,
    {
        let (sender, receiver) = oneshot::channel();
        self.use_render_root(move |roots| {
            if sender.send(use_fn(roots)).is_err() {
                log::warn!("sent operation result");
            }
        })?;
        receiver
            .await
            .map_err(|_| WindowHandleActionError::AppExited)
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

/// [`winit::Window`](winit::window::Window) Base window functions
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

/// [`winit::Window`](winit::window::Window) Position and size functions
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

/// [`winit::Window`](winit::window::Window) Misc. attribute functions
impl WindowHandle {
    /// Modifies the title of the window.
    ///
    /// See [`Window::set_title`](winit::window::Window::set_title) for more details.
    pub fn set_title(&self, title: &str) -> Result<(), WindowHandleActionError> {
        self.use_raw_window_now(|window| {
            window.set_title(title);
        })
    }
    /// Change the window transparency state.
    ///
    /// See [`Window::set_transparent`](winit::window::Window::set_transparent) for more details.
    pub fn set_transparent(&self, transparent: bool) -> Result<(), WindowHandleActionError> {
        self.use_raw_window_now(|window| {
            window.set_transparent(transparent);
        })
    }
    /// Change the window blur state.
    ///
    /// See [`Window::set_blur`](winit::window::Window::set_blur) for more details.
    pub fn set_blur(&self, blur: bool) -> Result<(), WindowHandleActionError> {
        self.use_raw_window_now(|window| {
            window.set_blur(blur);
        })
    }
    /// Modifies the window’s visibility.
    ///
    /// See [`Window::set_visible`](winit::window::Window::set_visible) for more details.
    pub fn set_visible(&self, visible: bool) -> Result<(), WindowHandleActionError> {
        self.use_winit_window_on_main(move |window| {
            window.set_visible(visible);
        })
    }
    /// Gets the window’s current visibility state.
    ///
    /// See [`Window::is_visible`](winit::window::Window::is_visible) for more details.
    pub fn is_visible(&self) -> Result<Option<bool>, WindowHandleActionError> {
        self.use_raw_window_now(|window| window.is_visible())
    }
    /// Sets whether the window is resizable or not.
    ///
    /// See [`Window::set_resizable`](winit::window::Window::set_resizable) for more details.
    pub fn set_resizable(&self, resizable: bool) -> Result<(), WindowHandleActionError> {
        self.use_raw_window_now(|window| {
            window.set_resizable(resizable);
        })
    }
    /// Gets the window’s current resizable state.
    ///
    /// See [`Window::is_resizable`](winit::window::Window::is_resizable) for more details.
    pub fn is_resizable(&self) -> Result<bool, WindowHandleActionError> {
        self.use_raw_window_now(|window| window.is_resizable())
    }
    /// Sets the enabled window buttons.
    ///
    /// See [`Window::set_enabled_buttons`](winit::window::Window::set_enabled_buttons) for more details.
    pub fn set_enabled_buttons(
        &self,
        buttons: WindowButtons,
    ) -> Result<(), WindowHandleActionError> {
        self.use_raw_window_now(|window| {
            window.set_enabled_buttons(buttons);
        })
    }
    /// Gets the enabled window buttons.
    ///
    /// See [`Window::enabled_buttons`](winit::window::Window::enabled_buttons) for more details.
    pub fn enabled_buttons(&self) -> Result<WindowButtons, WindowHandleActionError> {
        self.use_raw_window_now(|window| window.enabled_buttons())
    }
    /// Sets the window to minimized or back.
    ///
    /// See [`Window::set_minimized`](winit::window::Window::set_minimized) for more details.
    pub fn set_minimized(&self, minimized: bool) -> Result<(), WindowHandleActionError> {
        self.use_raw_window_now(|window| {
            window.set_minimized(minimized);
        })
    }
    /// Gets the window’s current minimized state.
    ///
    /// See [`Window::is_minimized`](winit::window::Window::is_minimized) for more details.
    pub fn is_minimized(&self) -> Result<Option<bool>, WindowHandleActionError> {
        self.use_raw_window_now(|window| window.is_minimized())
    }
    /// Sets the window to maximized or back.
    ///
    /// See [`Window::set_maximized`](winit::window::Window::set_maximized) for more details.
    pub fn set_maximized(&self, minimized: bool) -> Result<(), WindowHandleActionError> {
        self.use_raw_window_now(|window| {
            window.set_maximized(minimized);
        })
    }
    /// Gets the window’s current maximized state.
    ///
    /// See [`Window::is_maximized`](winit::window::Window::is_maximized) for more details.
    pub fn is_maximized(&self) -> Result<bool, WindowHandleActionError> {
        self.use_raw_window_now(|window| window.is_maximized())
    }
    /// Sets the window to fullscreen or back.
    ///
    /// See [`Window::set_fullscreen`](winit::window::Window::set_fullscreen) for more details.
    pub fn set_fullscreen(
        &self,
        fullscreen: Option<Fullscreen>,
    ) -> Result<(), WindowHandleActionError> {
        self.use_winit_window_on_main(move |window| {
            window.set_fullscreen(fullscreen);
        })
    }
    /// Gets the window’s current fullscreen state.
    ///
    /// Due to some limitation on iOS
    /// (as [it](winit::window::Window::fullscreen) can only be called on the main thread there),
    /// this function is not available there.
    ///
    /// We recommend using [`fullscreen_async`](Self::fullscreen_async) instead.
    ///
    /// See [`Window::fullscreen`](winit::window::Window::fullscreen) for more details.
    #[cfg(not(target_os = "ios"))]
    #[cfg_attr(docsrs, doc(not(target_os = "ios")))]
    pub fn fullscreen(&self) -> Result<Option<Fullscreen>, WindowHandleActionError> {
        self.use_raw_window_now(|window| window.fullscreen())
    }
    /// Gets the window’s current fullscreen state.
    ///
    /// _Due to some limitation on iOS
    /// (as [it](winit::window::Window::fullscree) can only be called on the main thread there),
    /// this function is `async`_.
    ///
    /// See [`Window::fullscreen`](winit::window::Window::fullscreen) for more details.
    pub async fn fullscreen_async(&self) -> Result<Option<Fullscreen>, WindowHandleActionError> {
        let (sender, receiver) = futures_channel::oneshot::channel::<_>();
        self.use_winit_window_on_main(move |window| {
            let _ = sender.send(window.fullscreen());
        })?;
        receiver
            .await
            .map_err(|_| WindowHandleActionError::AppExited)
    }
    /// Turn window decorations on or off.
    ///
    /// See [`Window::set_decorations`](winit::window::Window::set_decorations) for more details.
    pub fn set_decorations(&self, decorated: bool) -> Result<(), WindowHandleActionError> {
        self.use_raw_window_now(|window| {
            window.set_decorations(decorated);
        })
    }
    /// Gets the window’s current decorations state.
    ///
    /// See [`Window::is_decorated`](winit::window::Window::is_decorated) for more details.
    pub fn is_decorated(&self) -> Result<bool, WindowHandleActionError> {
        self.use_raw_window_now(|window| window.is_maximized())
    }
    /// Change the window level.
    ///
    /// See [`Window::set_window_level`](winit::window::Window::set_window_level) for more details.
    pub fn set_window_level(&self, level: WindowLevel) -> Result<(), WindowHandleActionError> {
        self.use_raw_window_now(|window| {
            window.set_window_level(level);
        })
    }
    /// Sets the window icon.
    ///
    /// See [`Window::set_window_icon`](winit::window::Window::set_window_icon) for more details.
    pub fn set_window_icon(&self, level: Option<Icon>) -> Result<(), WindowHandleActionError> {
        self.use_raw_window_now(|window| {
            window.set_window_icon(level);
        })
    }
    /// Set the IME cursor editing area,
    /// where the `position` is the top left corner of that area
    /// and `size` is the size of this area starting from the position.
    ///
    /// See [`Window::set_ime_cursor_area`](winit::window::Window::set_ime_cursor_area) for more details.
    pub fn set_ime_cursor_area<P, S>(
        &self,
        position: P,
        size: S,
    ) -> Result<(), WindowHandleActionError>
    where
        P: Into<dpi::Position>,
        S: Into<dpi::Size>,
    {
        self.use_raw_window_now(|window| {
            window.set_ime_cursor_area(position, size);
        })
    }
    /// Sets whether the window should get IME events.
    ///
    /// See [`Window::set_ime_allowed`](winit::window::Window::set_ime_allowed) for more details.
    pub fn set_ime_allowed(&self, allowed: bool) -> Result<(), WindowHandleActionError> {
        self.use_raw_window_now(|window| {
            window.set_ime_allowed(allowed);
        })
    }
    /// Sets the IME purpose for the window using [`ImePurpose`](winit::window::ImePurpose).
    ///
    /// See [`Window::set_ime_purpose`](winit::window::Window::set_ime_purpose) for more details.
    pub fn set_ime_purpose(&self, purpose: ImePurpose) -> Result<(), WindowHandleActionError> {
        self.use_raw_window_now(|window| {
            window.set_ime_purpose(purpose);
        })
    }
    /// Brings the window to the front and sets input focus.
    /// Has no effect if the window is already in focus, minimized, or not visible.
    ///
    /// This method steals input focus from other applications.
    /// Do not use this method unless you are certain that’s what the user wants.
    /// Focus stealing can cause an extremely disruptive user experience.
    ///
    /// See [`Window::focus_window`](winit::window::Window::focus_window) for more details.
    pub fn focus_window(&self) -> Result<(), WindowHandleActionError> {
        self.use_raw_window_now(|window| window.focus_window())
    }
    /// Gets whether the window has keyboard focus.
    ///
    /// See [`Window::has_focus`](winit::window::Window::has_focus) for more details.
    pub fn has_focus(&self) -> Result<bool, WindowHandleActionError> {
        self.use_raw_window_now(|window| window.has_focus())
    }
    /// Requests user attention to the window,
    /// this has no effect if the application is already focused.
    /// How requesting for user attention manifests is platform dependent,
    /// see [`UserAttentionType`] for details.
    ///
    /// See [`Window::request_user_attention`](winit::window::Window::request_user_attention) for more details.
    pub fn request_user_attention(
        &self,
        request_type: Option<UserAttentionType>,
    ) -> Result<(), WindowHandleActionError> {
        self.use_raw_window_now(|window| {
            window.request_user_attention(request_type);
        })
    }
    /// Set or override the window theme.
    ///
    /// Specify `None` to reset the theme to the system default.
    ///
    /// See [`Window::set_theme`](winit::window::Window::set_theme) for more details.
    pub fn set_theme(&self, theme: Option<Theme>) -> Result<(), WindowHandleActionError> {
        self.use_raw_window_now(|window| {
            window.set_theme(theme);
        })
    }
    /// Returns the current window theme.
    ///
    /// See [`Window::theme`](winit::window::Window::theme) for more details.
    pub fn theme(&self) -> Result<Option<Theme>, WindowHandleActionError> {
        self.use_raw_window_now(|window| window.theme())
    }
    /// Prevents the window contents from being captured by other apps.
    ///
    /// See [`Window::set_content_protected`](winit::window::Window::set_content_protected) for more details.
    pub fn set_content_protected(&self, protected: bool) -> Result<(), WindowHandleActionError> {
        self.use_raw_window_now(|window| {
            window.set_content_protected(protected);
        })
    }
    /// Gets the current title of the window.
    ///
    /// See [`Window::title`](winit::window::Window::title) for more details.
    pub fn title(&self) -> Result<String, WindowHandleActionError> {
        self.use_raw_window_now(|window| window.title())
    }
}

/// [`winit::Window`](winit::window::Window) Cursor functions.
impl WindowHandle {
    /// Modifies the cursor icon of the window.
    ///
    /// See [`Window::set_cursor`](winit::window::Window::set_cursor) for more details.
    pub fn set_cursor<C>(&self, cursor: C) -> Result<(), WindowHandleActionError>
    where
        C: Into<Cursor>,
    {
        self.use_raw_window_now(|window| window.set_cursor(cursor))
    }
    /// Changes the position of the cursor in window coordinates.
    ///
    /// See [`Window::set_cursor_position`](winit::window::Window::set_cursor_position) for more details.
    pub fn set_cursor_position<C>(&self, position: C) -> Result<(), WindowHandleActionError>
    where
        C: Into<dpi::Position>,
    {
        Ok(self.use_raw_window_now(|window| window.set_cursor_position(position))??)
    }
    /// Set grabbing [mode](CursorGrabMode) on the cursor preventing it from leaving the window.
    ///
    /// See [`Window::set_cursor_grab`](winit::window::Window::set_cursor_grab) for more details.
    pub fn set_cursor_grab(&self, mode: CursorGrabMode) -> Result<(), WindowHandleActionError> {
        Ok(self.use_raw_window_now(|window| window.set_cursor_grab(mode))??)
    }
    /// Modifies the cursor’s visibility.
    ///
    /// See [`Window::set_cursor_visible`](winit::window::Window::set_cursor_visible) for more details.
    pub fn set_cursor_visible(&self, visible: bool) -> Result<(), WindowHandleActionError> {
        self.use_raw_window_now(|window| {
            window.set_cursor_visible(visible);
        })
    }
    /// Moves the window with the left mouse button until the button is released.
    ///
    /// See [`Window::drag_window`](winit::window::Window::drag_window) for more details.
    pub fn drag_window(&self) -> Result<(), WindowHandleActionError> {
        Ok(self.use_raw_window_now(|window| window.drag_window())??)
    }
    /// Resizes the window with the left mouse button until the button is released.
    ///
    /// See [`Window::drag_resize_window`](winit::window::Window::drag_resize_window) for more details.
    pub fn drag_resize_window(
        &self,
        direction: ResizeDirection,
    ) -> Result<(), WindowHandleActionError> {
        Ok(self.use_raw_window_now(|window| window.drag_resize_window(direction))??)
    }
    /// Show [window menu](https://en.wikipedia.org/wiki/Common_menus_in_Microsoft_Windows#System_menu) at a specified position.
    ///
    /// See [`Window::show_window_menu`](winit::window::Window::show_window_menu) for more details.
    pub fn show_window_menu<P>(&self, position: P) -> Result<(), WindowHandleActionError>
    where
        P: Into<dpi::Position>,
    {
        self.use_raw_window_now(|window| {
            window.show_window_menu(position);
        })
    }
    /// Modifies whether the window catches cursor events.
    ///
    /// See [`Window::set_cursor_hittest`](winit::window::Window::set_cursor_hittest) for more details.
    pub fn set_cursor_hittest(&self, hittest: bool) -> Result<(), WindowHandleActionError> {
        Ok(self.use_raw_window_now(|window| window.set_cursor_hittest(hittest))??)
    }
}

/// [`winit::Window`](winit::window::Window) Monitor info functions.
impl WindowHandle {
    /// Returns the monitor on which the window currently resides.
    ///
    /// See [`Window::current_monitor`](winit::window::Window::current_monitor) for more details.
    pub fn current_monitor(&self) -> Result<Option<MonitorHandle>, WindowHandleActionError> {
        self.use_raw_window_now(|window| window.current_monitor())
    }
    /// Returns the list of all the monitors available on the system.
    ///
    /// See [`Window::available_monitors`](winit::window::Window::available_monitors) for more details.
    pub fn available_monitors(
        &self,
    ) -> Result<impl Iterator<Item = MonitorHandle>, WindowHandleActionError> {
        self.use_raw_window_now(|window| window.available_monitors())
    }
    /// Returns the primary monitor of the system.
    ///
    /// See [`Window::primary_monitor`](winit::window::Window::primary_monitor) for more details.
    pub fn primary_monitor(&self) -> Result<Option<MonitorHandle>, WindowHandleActionError> {
        self.use_raw_window_now(|window| window.primary_monitor())
    }
}

impl WindowHandle {
    /// Return a child [`Owner`](reactive_graph::owner::Owner) of this window handle
    pub async fn child_reactive_owner(
        &self,
    ) -> Result<reactive_graph::owner::Owner, WindowHandleActionError> {
        let (sender, receiver) = oneshot::channel();
        self.get_proxy()
            .send_event(EventLoopEvent::GetWindowChildReactiveOwner(Box::new(
                GetWindowChildReactiveOwner {
                    window_id: self.id()?,
                    sender,
                },
            )))?;
        receiver
            .await
            .map_err(|_| WindowHandleActionError::AppExited)
    }
}

/// [`RenderRoot`] getter functions
impl WindowHandle {
    /// Checks if a widget with the given id is in the tree.
    pub async fn has_widget(&self, widget_id: WidgetId) -> Result<bool, WindowHandleActionError> {
        self.use_render_root_with_return(move |root| root.has_widget(widget_id))
            .await
    }
    /// Get a [`reference`](VelonaWidgetRef) of the current `widget_id`
    pub fn get_widget_ref(&self, widget_id: WidgetId) -> VelonaWidgetRef<dyn Widget> {
        VelonaWidgetRef {
            id: widget_id,
            window: Some(Box::new(self.clone())),
            phantom: PhantomData::<dyn Widget>,
            thread_id: std::thread::current().id(),
        }
    }

    // TODO from tag?

    /// Returns the [`WidgetId`] of the focused widget.
    pub async fn focused_widget(&self) -> Result<Option<WidgetId>, WindowHandleActionError> {
        self.use_render_root_with_return(|root| root.focused_widget())
            .await
    }
    /// Returns the [`reference`](VelonaWidgetRef) of the focused widget.
    pub async fn focused_widget_ref(
        &self,
    ) -> Result<Option<VelonaWidgetRef<dyn Widget>>, WindowHandleActionError> {
        let Some(focused_id) = self
            .use_render_root_with_return(|root| root.focused_widget())
            .await?
        else {
            return Ok(None);
        };
        Ok(Some(self.get_widget_ref(focused_id)))
    }
    /// Returns the [`WidgetId`] of the widget which captures pointer events.
    pub async fn pointer_capture_target(
        &self,
    ) -> Result<Option<WidgetId>, WindowHandleActionError> {
        self.use_render_root_with_return(|root| root.pointer_capture_target())
            .await
    }
    /// Returns the [`reference`](VelonaWidgetRef) of the widget which captures pointer events.
    pub async fn pointer_capture_target_ref(
        &self,
    ) -> Result<Option<VelonaWidgetRef<dyn Widget>>, WindowHandleActionError> {
        let Some(focused_id) = self
            .use_render_root_with_return(|root| root.pointer_capture_target())
            .await?
        else {
            return Ok(None);
        };
        Ok(Some(self.get_widget_ref(focused_id)))
    }
    /// Sets the [focused widget](masonry_core::doc::masonry_concepts#text-focus)
    /// and the [focus anchor](masonry_core::doc::masonry_concepts#focus-anchor).
    pub fn focus_on(&self, widget_id: Option<WidgetId>) -> Result<(), WindowHandleActionError> {
        self.use_render_root(move |root| {
            root.focus_on(widget_id);
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
