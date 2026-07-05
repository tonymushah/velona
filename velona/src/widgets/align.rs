//! Various [`Align`] trait implementations.
//!
//! The most important thing in the module is the [`NewAlignExt`]
//! which is implemented for [`NewWidget<Align>`].
//!
//! The [`NewWidget<Align>`] also implements the [`ReactiveSingleChildExt`][reactive-child] trait
//! and the [`SingleChildWidget`][single-widget] trait.
//!
//! _See the [widget](Align) documentation for more information_.
//!
//! [single-widget]: super::SingleChildWidget
//! [reactive-child]: super::ReactiveSingleChildExt

use masonry::{core::NewWidget, layout::UnitPoint, widgets::Align};

use crate::NewWidgetExt;

/// A [new](NewWidget) [`Align`] trait extension
pub trait NewAlignExt {
    /// Make the [`Align::set_alignment`] reactive
    fn alignment<A>(self, alignment: A) -> Self
    where
        A: Fn() -> UnitPoint + 'static;
}

impl NewAlignExt for NewWidget<Align> {
    fn alignment<A>(self, alignment: A) -> Self
    where
        A: Fn() -> UnitPoint + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Align::set_alignment(&mut this, alignment());
        })
    }
}
