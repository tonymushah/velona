//! Various [`DisclosureButton`] implementations.
//!
//! _See the [widget](DisclosureButton) documentation for more information_.
//!
//! The most important thing in the module is the [`NewDisclosureButtonExt`]
//! which is implemented for [`NewWidget<DisclosureButton>`].

use masonry::{core::NewWidget, widgets::DisclosureButton};

use crate::NewWidgetExt;

/// A [`DisclosureButton`] extension trait
// TODO add example
pub trait NewDisclosureButtonExt {
    /// Set the [disclosed](DisclosureButton::set_disclosed) reactively.
    fn disclosed<F>(self, disclosed: F) -> Self
    where
        F: Fn() -> bool + 'static;
}

impl NewDisclosureButtonExt for NewWidget<DisclosureButton> {
    fn disclosed<F>(self, disclosed: F) -> Self
    where
        F: Fn() -> bool + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            DisclosureButton::set_disclosed(&mut this, disclosed());
        })
    }
}
