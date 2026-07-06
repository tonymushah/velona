//! Various [`VirtualScroll`] implementations.
//!
//! The most important thing in the module is the [`NewVirtualScrollExt`]
//! which is implemented for [`NewWidget<VirtualScroll>`].
//!
//! _See the [widget](VirtualScroll) documentation for more information_.

use masonry::{
    core::NewWidget,
    widgets::{ScrollDirection, VirtualScroll},
};

use crate::NewWidgetExt;

/// A [new](NewWidget) [`VirtualScroll`] trait extension.
pub trait NewVirtualScrollExt {
    /// Sets the valid number of items.
    ///
    /// That is, the children
    /// which the virtual scrolling area will request within.
    ///
    /// Reactive equivalent of [`with_len`](VirtualScroll::with_len).
    fn len<L>(self, len: L) -> Self
    where
        L: Fn() -> usize + 'static;
    /// Sets the point (as ratio of the main-axis length)
    /// where the first item starts in the viewport.
    ///
    /// Reactive equivalent of [`set_start`](VirtualScroll::set_start).
    fn start<S>(self, start_at: S) -> Self
    where
        S: Fn() -> f64 + 'static;
    /// Sets the point (as ratio of the main-axis length)
    /// where the last item ends in the viewport.
    ///
    /// Reactive equivalent of [`set_end`](VirtualScroll::set_end).
    fn end<E>(self, end_at: E) -> Self
    where
        E: Fn() -> f64 + 'static;
    /// Sets the direction in which children are laid out.
    ///
    /// Reactive equivalent of [`set_direction`](VirtualScroll::set_direction).
    fn direction<D>(self, direction: D) -> Self
    where
        D: Fn() -> ScrollDirection + 'static;
    /// Sets scrolling state.
    ///
    /// Adjusts pixel snapping for animations.
    ///
    /// Reactive equivalent of [`set_scrolling`](VirtualScroll:set_scrolling).
    fn scrolling<S>(self, scrolling: S) -> Self
    where
        S: Fn() -> bool + 'static;
    /// Forcefully aligns the top of the item at `idx`
    /// with the top of the virtual scroll area.
    ///
    /// The reactive equivalent of [`scroll_to`](VirtualScroll::scroll_to).
    ///
    /// That is, scroll to the item at `idx`,
    /// losing any scroll progress by the user.
    ///
    /// This method is mostly useful for tests,
    /// but can be used outside of tests (for example, in certain scrollbar schemes).
    fn scroll_to<I>(self, idx: I) -> Self
    where
        I: Fn() -> usize + 'static;
}

impl NewVirtualScrollExt for NewWidget<VirtualScroll> {
    fn len<L>(self, len: L) -> Self
    where
        L: Fn() -> usize + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            VirtualScroll::set_len(&mut this, len());
        })
    }

    fn start<S>(self, start_at: S) -> Self
    where
        S: Fn() -> f64 + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            VirtualScroll::set_start(&mut this, start_at());
        })
    }

    fn end<E>(self, end_at: E) -> Self
    where
        E: Fn() -> f64 + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            VirtualScroll::set_end(&mut this, end_at());
        })
    }

    fn direction<D>(self, direction: D) -> Self
    where
        D: Fn() -> ScrollDirection + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            VirtualScroll::set_direction(&mut this, direction());
        })
    }

    fn scrolling<S>(self, scrolling: S) -> Self
    where
        S: Fn() -> bool + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            VirtualScroll::set_scrolling(&mut this, scrolling());
        })
    }

    fn scroll_to<I>(self, idx: I) -> Self
    where
        I: Fn() -> usize + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            VirtualScroll::scroll_to(&mut this, idx());
        })
    }
}
