use std::ops::Range;

use masonry::{core::NewWidget, widgets::VirtualScroll};

use crate::NewWidgetExt;

/// A [new](NewWidget) [`VirtualScroll`] trait extension.
pub trait NewVirtualScrollExt {
    /// Sets the valid range of ids.
    ///
    /// That is, the children which the virtual scrolling area will request within. Reactive equivalent of [`with_valid_range`](VirtualScroll::with_valid_range).
    ///
    /// # Panics
    ///
    /// If `valid_range.start >= valid_range.end`.
    /// Note that other empty ranges are fine,
    /// although the exact behaviour hasn’t been carefully validated.
    fn valid_range<V>(self, valid_range: V) -> Self
    where
        V: Fn() -> Range<i64> + 'static;
    /// Forcefully aligns the top of the item at `idx` with the top of the virtual scroll area.
    ///
    /// That is, scroll to the item at `idx`, losing any scroll progress by the user.
    ///
    /// This method is mostly useful for tests,
    /// but can be used outside of tests
    /// (for example, in certain scrollbar schemes).
    ///
    /// The reactive equivalent of [`overwrite_anchor`].
    fn anchor<A>(self, overwrite_anchor: A) -> Self
    where
        A: Fn() -> i64 + 'static;
}

impl NewVirtualScrollExt for NewWidget<VirtualScroll> {
    fn valid_range<V>(self, valid_range: V) -> Self
    where
        V: Fn() -> Range<i64> + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            VirtualScroll::set_valid_range(&mut this, valid_range());
        })
    }

    fn anchor<A>(self, idx: A) -> Self
    where
        A: Fn() -> i64 + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            VirtualScroll::overwrite_anchor(&mut this, idx());
        })
    }
}
