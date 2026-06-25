use masonry::{
    core::{NewWidget, WidgetMut},
    widgets::ScrollBar,
};

use crate::NewWidgetExt;

/// A struct for [`ScrollBar::set_sizes`].
#[derive(Debug, Clone, Copy, Default)]
pub struct ScrollBarSizes {
    pub portal_size: f64,
    pub content_size: f64,
}

impl ScrollBarSizes {
    /// Apply this to a [`ScrollBar`].
    ///
    /// _I guess this is self-explainatory :)_.
    pub fn apply(self, this: &mut WidgetMut<ScrollBar>) {
        ScrollBar::set_sizes(this, self.portal_size, self.content_size);
    }
}

/// A [new](NewWidget) [`ScrollBar`] trait extension.
pub trait NewScrollBarExt {
    /// Set the [scrollbar sizes](ScrollBar::set_sizes) reactively.
    fn sizes<S>(self, sizes: S) -> Self
    where
        S: Fn() -> ScrollBarSizes + 'static;
    /// Set the [scrollbar content size](ScrollBar::set_content_size) reactively.
    fn content_size<S>(self, size: S) -> Self
    where
        S: Fn() -> f64 + 'static;
}

impl NewScrollBarExt for NewWidget<ScrollBar> {
    fn sizes<S>(self, sizes: S) -> Self
    where
        S: Fn() -> ScrollBarSizes + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            sizes().apply(&mut this);
        })
    }

    fn content_size<S>(self, content_size: S) -> Self
    where
        S: Fn() -> f64 + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            ScrollBar::set_content_size(&mut this, content_size());
        })
    }
}
