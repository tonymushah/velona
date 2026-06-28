use masonry::{core::NewWidget, widgets::Switch};

use crate::NewWidgetExt;

/// A [new](NewWidget) [`Switch`] trait extension.
pub trait NewSwitchExt {
    /// Sets the [switch state](Switch::set_on)
    /// reactively.
    fn on<S>(self, on: S) -> Self
    where
        S: Fn() -> bool + 'static;
}

impl NewSwitchExt for NewWidget<Switch> {
    fn on<S>(self, on: S) -> Self
    where
        S: Fn() -> bool + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Switch::set_on(&mut this, on());
        })
    }
}
