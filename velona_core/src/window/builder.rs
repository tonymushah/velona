use futures_channel::oneshot;
use masonry_core::{
    core::{NewWidget, Widget},
    peniko::color::{AlphaColor, Srgb},
};
use winit::{
    dpi::{Position, Size},
    window::{Fullscreen, WindowAttributes, WindowButtons, WindowLevel},
};

use crate::window::handle::WindowHandle;

pub struct WindowBuilder {
    pub(crate) view: Box<dyn FnOnce() -> NewWidget<dyn Widget + 'static> + Send>,
    pub(crate) window_attributes: WindowAttributes,
    pub(crate) base_color: Option<AlphaColor<Srgb>>,
    pub(crate) window_handle_send: Option<oneshot::Sender<WindowHandle>>,
}

impl WindowBuilder {
    pub fn new<F>(view_fn: F) -> Self
    where
        F: FnOnce() -> NewWidget<dyn Widget + 'static> + Send + 'static,
    {
        Self {
            view: Box::new(view_fn),
            window_attributes: WindowAttributes::default(),
            base_color: None,
            window_handle_send: None,
        }
        .with_title("velona window")
    }
    pub fn window_attributes(mut self, window_attributes: WindowAttributes) -> Self {
        self.window_attributes = window_attributes;
        self
    }
    pub fn update_window_attributes<U>(mut self, update_fn: U) -> Self
    where
        U: FnOnce(WindowAttributes) -> WindowAttributes,
    {
        self.window_attributes = update_fn(self.window_attributes);
        self
    }

    pub fn base_color(mut self, base_color: AlphaColor<Srgb>) -> Self {
        self.base_color = Some(base_color);
        self
    }
    // TODO implement other winit and masonry builder options
}

/// Winit based methods
impl WindowBuilder {
    /// Requests the window to be of specific dimensions.
    ///
    /// If this is not set, some platform-specific dimensions will be used.
    ///
    /// See [`winit::Window::request_inner_size`](winit::window::Window::request_inner_size) for details.
    pub fn with_inner_size<S>(self, size: S) -> Self
    where
        S: Into<Size>,
    {
        self.update_window_attributes(|att| att.with_inner_size(size))
    }
    /// Sets the minimum dimensions a window can have.
    ///
    /// If this is not set, the window will have no minimum dimensions (aside from reserved).
    ///
    /// See [`Window::set_min_inner_size`](winit::window::Window::set_min_inner_size) for details.
    pub fn with_min_inner_size<S>(self, size: S) -> Self
    where
        S: Into<Size>,
    {
        self.update_window_attributes(|att| att.with_min_inner_size(size))
    }
    /// Sets the maximum dimensions a window can have.
    ///
    /// If this is not set, the window will have no maximum or will be set
    /// to the primary monitor’s dimensions by the platform.
    ///
    /// See [`Window::set_max_inner_size`](winit::window::Window::set_max_inner_size) for details.
    pub fn with_max_inner_size<S>(self, size: S) -> Self
    where
        S: Into<Size>,
    {
        self.update_window_attributes(|att| att.with_max_inner_size(size))
    }
    /// Sets a desired initial position for the window.
    ///
    /// If this is not set, some platform-specific position will be chosen.
    ///
    /// See [`Window::set_outer_position`](winit::window::Window::set_outer_position)
    /// and [`WindowAttributes::with_position`] for details.
    pub fn with_position<S>(self, size: S) -> Self
    where
        S: Into<Position>,
    {
        self.update_window_attributes(|att| att.with_position(size))
    }
    /// Sets whether the window is resizable or not.
    ///
    /// The default is `true`.
    ///
    /// See [`Window::set_resizable`](winit::window::Window::set_resizable) for details.
    pub fn with_resizable(self, resizable: bool) -> Self {
        self.update_window_attributes(|att| att.with_resizable(resizable))
    }
    /// Sets the enabled window buttons.
    ///
    /// The default is [`WindowButtons::all`]
    ///
    /// See [`Window::set_enabled_buttons`](winit::window::Window::set_enabled_buttons) for details.
    pub fn with_enabled_buttons(self, buttons: WindowButtons) -> Self {
        self.update_window_attributes(|att| att.with_enabled_buttons(buttons))
    }
    /// Sets the initial title of the window in the title bar.
    ///
    /// The default is "velona window".
    ///
    /// See [`window::Window::set_title`](winit::window::Window::set_title) for details.
    pub fn with_title<T>(self, title: T) -> Self
    where
        T: Into<String>,
    {
        self.update_window_attributes(|att| att.with_title(title))
    }
    /// Sets whether the window should be put into fullscreen upon creation.
    ///
    /// The default is `None`.
    ///
    /// See [`Window::set_fullscreen`](winit::window::Window::set_fullscreen) for details.
    pub fn with_fullscreen(self, fullscreen: Option<Fullscreen>) -> Self {
        self.update_window_attributes(|att| att.with_fullscreen(fullscreen))
    }
    /// Request that the window is maximized upon creation.
    ///
    /// The default is `false`.
    ///
    /// See [`Window::set_maximized`](winit::window::Window::set_maximized) for details.
    pub fn with_maximized(self, maximized: bool) -> Self {
        self.update_window_attributes(|att| att.with_maximized(maximized))
    }
    /// Sets whether the window will be initially visible or hidden.
    ///
    /// The default is to show the window.
    ///
    /// See [`Window::set_visible`](winit::window::Window::set_visible) for details.
    pub fn with_visible(self, visible: bool) -> Self {
        self.update_window_attributes(|att| att.with_visible(visible))
    }
    /// Sets whether the background of the window should be transparent.
    ///
    /// If this is `true`, writing colors with alpha values different than `1.0` will produce a transparent window.
    /// On some platforms this is more of a hint for the system and you’d still have the alpha buffer.
    /// To control it see [`Window::set_transparent`](winit::window::Window::set_transparent).
    ///
    /// The default is `false`.
    pub fn with_transparent(self, transparent: bool) -> Self {
        self.update_window_attributes(|att| att.with_transparent(transparent))
    }
    /// Sets whether the background of the window should be blurred by the system.
    ///
    /// The default is `false`.
    ///
    /// See [`Window::set_blur`](winit::window::Window::set_blur) for details.
    pub fn with_blur(self, blur: bool) -> Self {
        self.update_window_attributes(|att| att.with_blur(blur))
    }
    /// Sets whether the window should have a border, a title bar, etc.
    ///
    /// The default is `true`.
    ///
    /// See [`Window::set_decorations`](winit::window::Window::set_decorations) for details.
    pub fn with_decorations(self, decoration: bool) -> Self {
        self.update_window_attributes(|att| att.with_decorations(decoration))
    }
    /// Sets the window level.
    ///
    /// This is just a hint to the OS, and the system could ignore it.
    ///
    /// The default is [`WindowLevel::Normal`].
    ///
    /// See [`WindowLevel`] for details.
    pub fn with_window_level(self, level: WindowLevel) -> Self {
        self.update_window_attributes(|att| att.with_window_level(level))
    }
    /// Sets the window icon.
    ///
    /// The default is `None`.
    ///
    /// See [`Window::set_window_icon`](winit::window::Window::set_window_icon) for details.
    pub fn with_window_icon(self, window_icon: Option<winit::window::Icon>) -> Self {
        self.update_window_attributes(|att| att.with_window_icon(window_icon))
    }
}
