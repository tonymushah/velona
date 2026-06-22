use masonry::{core::NewWidget, widgets::Pagination};

use crate::NewWidgetExt;

/// A [new](NewWidget) [`Pagination`] extension trait.
// TODO add example
pub trait NewPaginationExt {
    /// Set the [pagination page count](Pagination::set_page_count) reactively.
    ///
    /// The active page index is clamped to this new total.
    fn page_count<C>(self, page_count: C) -> Self
    where
        C: Fn() -> usize + 'static;
    /// Set the [pagination active page](Pagination::set_active_page) reactively.
    ///
    /// This is a 0-based index.
    ///
    /// It is clamped to the total page count.
    fn active_page<C>(self, active_page: C) -> Self
    where
        C: Fn() -> usize + 'static;
    /// Set the [pagination buttons start](Pagination::set_buttons_start) reactively.
    fn buttons_start<C>(self, buttons_start: C) -> Self
    where
        C: Fn() -> u8 + 'static;
    /// Set the [pagination buttons end](Pagination::set_buttons_end) reactively.
    fn buttons_end<C>(self, buttons_end: C) -> Self
    where
        C: Fn() -> u8 + 'static;
    /// Set the [pagination buttons total](Pagination::set_buttons_total) reactively.
    ///
    /// The effective button limit also depends on the total page count.
    fn buttons_total<C>(self, buttons_total: C) -> Self
    where
        C: Fn() -> u8 + 'static;
}

impl NewPaginationExt for NewWidget<Pagination> {
    fn page_count<C>(self, page_count: C) -> Self
    where
        C: Fn() -> usize + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Pagination::set_page_count(&mut this, page_count());
        })
    }

    fn active_page<C>(self, active_page: C) -> Self
    where
        C: Fn() -> usize + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Pagination::set_active_page(&mut this, active_page());
        })
    }

    fn buttons_start<C>(self, buttons_start: C) -> Self
    where
        C: Fn() -> u8 + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Pagination::set_buttons_start(&mut this, buttons_start());
        })
    }

    fn buttons_end<C>(self, buttons_end: C) -> Self
    where
        C: Fn() -> u8 + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Pagination::set_buttons_end(&mut this, buttons_end());
        })
    }

    fn buttons_total<C>(self, buttons_total: C) -> Self
    where
        C: Fn() -> u8 + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Pagination::set_buttons_total(&mut this, buttons_total());
        })
    }
}
