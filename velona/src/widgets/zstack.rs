use masonry::{core::NewWidget, layout::UnitPoint, widgets::ZStack};

use crate::NewWidgetExt;

/// A [new](NewWidget) [`ZStack`] trait extension.
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
        self.use_reactive_widget_mut(move |mut this| {
            ZStack::set_alignment(&mut this, alignment());
        })
    }
}
