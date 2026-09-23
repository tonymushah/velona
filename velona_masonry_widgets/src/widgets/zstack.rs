//! Various [`ZStack`] implementations.
//!
//! The most important thing in the module is the [`NewZStackExt`]
//! which is implemented for [`NewWidget<ZStack>`].
//!
//! _See the [widget](ZStack) documentation for more information_.

use masonry::{core::NewWidget, layout::UnitPoint, widgets::ZStack};

use crate::NewWidgetExt;

/// A [new](NewWidget) [`ZStack`] trait extension.
#[must_use]
pub trait NewZStackExt {
    /// Changes the alignment of the [`ZStack`].
    ///
    /// See also [`with_alignment`](ZStack::with_alignment).
    fn alignment<A, U>(self, alignment: A) -> Self
    where
        A: Fn() -> U + 'static,
        U: Into<UnitPoint> + 'static;
}

impl NewZStackExt for NewWidget<ZStack> {
    fn alignment<A, U>(self, alignment: A) -> Self
    where
        A: Fn() -> U + 'static,
        U: Into<UnitPoint> + 'static,
    {
        self.use_widget_mut(alignment, |mut this, alignment| {
            ZStack::set_alignment(&mut this, alignment);
        })
    }
}
