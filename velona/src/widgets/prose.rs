use masonry::{
    core::{NewWidget, WidgetMut},
    widgets::{Prose, TextArea},
};

#[cfg(doc)]
use reactive_graph::effect::Effect;

use crate::NewWidgetExt;

/// A [new](NewWidget) [`Prose`] trait extension.
pub trait NewProseExt {
    /// Whether to clip the text to the available space.
    ///
    /// Reactive variant of [`Prose::set_clip`]
    fn clip<C>(self, clip: C) -> Self
    where
        C: Fn() -> bool + 'static;
    /// Use the underlying text area.
    ///
    /// It is worth noting that the `use_fn` will run inside an [`Effect`].
    ///
    /// Used to modify most properties of the text.
    fn use_text_mut<T>(self, use_fn: T) -> Self
    where
        T: FnMut(WidgetMut<TextArea<false>>) + 'static;
}

impl NewProseExt for NewWidget<Prose> {
    fn clip<C>(self, clip: C) -> Self
    where
        C: Fn() -> bool + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Prose::set_clip(&mut this, clip());
        })
    }

    fn use_text_mut<T>(self, mut use_fn: T) -> Self
    where
        T: FnMut(WidgetMut<TextArea<false>>) + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            use_fn(Prose::text_mut(&mut this));
        })
    }
}
