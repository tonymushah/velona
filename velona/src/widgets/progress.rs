use masonry::{core::NewWidget, widgets::ProgressBar};

use crate::NewWidgetExt;

/// A [new](NewWidget) [`ProgressBar`] trait extension.
// TODO add example
pub trait NewProgressBarExt {
    /// Set the [`progress`](ProgressBar::set_progress) reactively.
    fn progress<P>(self, progress: P) -> Self
    where
        P: Fn() -> Option<f64> + 'static;
}

impl NewProgressBarExt for NewWidget<ProgressBar> {
    fn progress<P>(self, progress: P) -> Self
    where
        P: Fn() -> Option<f64> + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            ProgressBar::set_progress(&mut this, progress());
        })
    }
}
