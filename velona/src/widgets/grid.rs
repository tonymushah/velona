use masonry::{core::NewWidget, widgets::Grid};

use crate::NewWidgetExt;

/// A [new](NewWidget) [`Grid`] extension trait.
// TODO add example
pub trait NewGridExt {
    /// [Set the grid column count](Grid::set_column_count) reactively.
    fn column_count<C>(self, count: C) -> Self
    where
        C: Fn() -> i32 + 'static;
    /// [Set the row column count](Grid::set_row_count) reactively.
    fn row_count<C>(self, count: C) -> Self
    where
        C: Fn() -> i32 + 'static;
}

impl NewGridExt for NewWidget<Grid> {
    fn column_count<C>(self, count: C) -> Self
    where
        C: Fn() -> i32 + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Grid::set_column_count(&mut this, count());
        })
    }

    fn row_count<C>(self, count: C) -> Self
    where
        C: Fn() -> i32 + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Grid::set_row_count(&mut this, count());
        })
    }
}
