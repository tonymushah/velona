//! Various [`Selector`] implementations.
//!
//! The most important thing in the module is the [`NewSelectorExt`]
//! which is implemented for [`NewWidget<Selector>`].
//!
//! _See the [widget](Selector) documentation for more information_.

use masonry::{core::NewWidget, widgets::Selector};
use velona_core::reactive::traits::SignalOrFn;

use crate::NewWidgetExt;

/// A [new](NewWidget) [`Selector`] trait extension.
// TODO Add example
#[must_use]
pub trait NewSelectorExt {
    /// Sets [the list of options with a new one](Selector::set_options) reactively.
    ///
    /// Selects the first option.
    ///
    /// # Panics
    ///
    /// Panics when debug assertions are on if options is empty.
    fn options<O>(self, options: O) -> Self
    where
        O: SignalOrFn<Output = Vec<String>> + 'static;
    /// Selects the given option reactively.
    ///
    /// # Panics
    ///
    /// Panics when debug assertions are on if selected_option is out of bounds.
    fn select_option<O>(self, select_option: O) -> Self
    where
        O: SignalOrFn<Output = usize> + 'static;
}

impl NewSelectorExt for NewWidget<Selector> {
    fn options<O>(self, options: O) -> Self
    where
        O: SignalOrFn<Output = Vec<String>> + 'static,
    {
        self.use_widget_mut(options, |mut this, options| {
            Selector::set_options(&mut this, options);
        })
    }

    fn select_option<O>(self, selected_option: O) -> Self
    where
        O: SignalOrFn<Output = usize> + 'static,
    {
        self.use_widget_mut(selected_option, |mut this, selected_option| {
            Selector::select_option(&mut this, selected_option);
        })
    }
}
