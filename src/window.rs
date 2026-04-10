use masonry::{
    core::{NewWidget, Widget},
    peniko::color::{AlphaColor, Srgb},
};
use winit::{dpi::Size, window::WindowAttributes};

pub struct WindowBuilder {
    pub(crate) view: Box<dyn FnOnce() -> NewWidget<dyn Widget + 'static> + Send + Sync>,
    pub(crate) window_attributes: WindowAttributes,
    pub(crate) base_color: Option<AlphaColor<Srgb>>,
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
        }
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
    pub fn with_title<T>(self, title: T) -> Self
    where
        T: Into<String>,
    {
        self.update_window_attributes(|att| att.with_title(title))
    }
    pub fn with_inner_size<S>(self, size: S) -> Self
    where
        S: Into<Size>,
    {
        self.update_window_attributes(|att| att.with_inner_size(size))
    }
    pub fn base_color(mut self, base_color: AlphaColor<Srgb>) -> Self {
        self.base_color = Some(base_color);
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::is_send_sync;

    #[test]
    fn test_if_window_builder_is_send_sync() {
        is_send_sync::<super::WindowBuilder>();
    }
}
