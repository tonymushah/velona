use std::sync::Arc;

use masonry::{
    core::{ArcStr, NewWidget},
    widgets::Svg,
};
pub use resvg::usvg;

use usvg::Tree;

use crate::NewWidgetExt;

/// A [new](NewWidget) [`Svg`] trait extension.
pub trait NewSvgExt {
    /// [Sets a new inner SVG](Svg::set_tree) reactively.
    fn tree<T>(self, tree: T) -> Self
    where
        T: Fn() -> Arc<Tree> + 'static;
    /// *Reactive variant of [`Svg::set_decorative`].*
    ///
    /// Sets whether the SVG is decorative,
    /// meaning it doesn’t have meaningful content and is only for visual presentation.
    ///
    /// See [`Svg::decorative`] for details.
    fn decorative<D>(self, is_decorative: D) -> Self
    where
        D: Fn() -> bool + 'static;
    /// *Reactive variant of [`Svg::set_alt_text`].*
    ///
    /// Sets the text that will describe the SVG to screen readers.
    ///
    /// See [`Svg::with_alt_text`] for details.
    fn alt_text<A>(self, alt_text: A) -> Self
    where
        A: Fn() -> Option<ArcStr> + 'static;
}

impl NewSvgExt for NewWidget<Svg> {
    fn tree<T>(self, tree: T) -> Self
    where
        T: Fn() -> Arc<Tree> + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Svg::set_tree(&mut this, tree());
        })
    }

    fn decorative<D>(self, is_decorative: D) -> Self
    where
        D: Fn() -> bool + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Svg::set_decorative(&mut this, is_decorative());
        })
    }

    fn alt_text<A>(self, alt_text: A) -> Self
    where
        A: Fn() -> Option<ArcStr> + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Svg::set_alt_text(&mut this, alt_text());
        })
    }
}
