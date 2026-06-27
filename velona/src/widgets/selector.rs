use masonry::{
    core::NewWidget,
    widgets::{Label, Selector},
};

use crate::{NewWidgetExt, widgets::TypedSingleChildWidget};

/// A [new](NewWidget) [`Selector`] trait extension.
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
        O: Fn() -> Vec<String> + 'static;
    /// Selects the given option reactively.
    ///
    /// # Panics
    ///
    /// Panics when debug assertions are on if selected_option is out of bounds.
    fn select_option<O>(self, select_option: O) -> Self
    where
        O: Fn() -> usize + 'static;
}

impl NewSelectorExt for NewWidget<Selector> {
    fn options<O>(self, options: O) -> Self
    where
        O: Fn() -> Vec<String> + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Selector::set_options(&mut this, options());
        })
    }

    fn select_option<O>(self, selected_option: O) -> Self
    where
        O: Fn() -> usize + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Selector::select_option(&mut this, selected_option());
        })
    }
}

impl TypedSingleChildWidget for NewWidget<Selector> {
    type Child = Label;

    fn use_child<C>(self, mut use_child_fn: C) -> Self
    where
        C: FnMut(masonry::core::WidgetMut<'_, Self::Child>) + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            use_child_fn(Selector::child_mut(&mut this));
        })
    }
}
