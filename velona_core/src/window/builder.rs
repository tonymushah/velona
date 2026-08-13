use futures_channel::oneshot;
use masonry_core::{
    core::{NewWidget, Widget},
    peniko::color::{AlphaColor, Srgb},
};
use winit::{
    dpi::{Position, Size},
    window::WindowAttributes,
};

use crate::window::handle::WindowHandle;

pub struct WindowBuilder {
    pub(crate) view: Box<dyn FnOnce() -> NewWidget<dyn Widget + 'static> + Send + Sync>,
    pub(crate) window_attributes: WindowAttributes,
    pub(crate) base_color: Option<AlphaColor<Srgb>>,
    pub(crate) window_handle_send: Option<oneshot::Sender<WindowHandle>>,
}

impl WindowBuilder {
    pub fn new<F>(view_fn: F) -> Self
    where
        F: FnOnce() -> NewWidget<dyn Widget + 'static> + Send + Sync + 'static,
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
}
