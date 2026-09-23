//! Various [`Grid`] implementations.
//!
//! The most important thing in the module is the [`NewGridExt`]
//! which is implemented for [`NewWidget<Grid>`].
//!
//! _See the [widget](Grid) documentation for more information_.

use masonry::{
    core::NewWidget,
    widgets::{Grid, GridTrackSize},
};

use crate::NewWidgetExt;

/// A [new](NewWidget) [`Grid`] extension trait.
// TODO add example
#[must_use]
pub trait NewGridExt {
    /// [Set the grid columns](Grid::set_columns) reactively.
    fn columns<C>(self, track_sizes: C) -> Self
    where
        C: Fn() -> Vec<GridTrackSize> + 'static;
    /// [Set the grid rows](Grid::set_rows) reactively.
    fn rows<C>(self, track_sizes: C) -> Self
    where
        C: Fn() -> Vec<GridTrackSize> + 'static;
}

impl NewGridExt for NewWidget<Grid> {
    fn columns<C>(self, track_sizes: C) -> Self
    where
        C: Fn() -> Vec<GridTrackSize> + 'static,
    {
        self.use_widget_mut(track_sizes, |mut this, track_sizes| {
            Grid::set_columns(&mut this, track_sizes);
        })
    }

    fn rows<C>(self, track_sizes: C) -> Self
    where
        C: Fn() -> Vec<GridTrackSize> + 'static,
    {
        self.use_widget_mut(track_sizes, |mut this, track_sizes| {
            Grid::set_rows(&mut this, track_sizes);
        })
    }
}
