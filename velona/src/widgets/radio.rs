use masonry::{
    core::{ArcStr, NewWidget, WidgetMut},
    widgets::{Label, RadioButton},
};

#[cfg(doc)]
use reactive_graph::effect::Effect;

use crate::NewWidgetExt;

/// A [new](NewWidget) [`RadioButton`] extension trait.
// TODO add example
pub trait NewRadioButtonExt {
    /// [Check or uncheck the box](RadioButton::set_checked) reactively.
    fn checked<C>(self, checked: C) -> Self
    where
        C: Fn() -> bool + 'static;
    /// [Set the text](RadioButton::set_text) reactively.
    ///
    /// We enforce this to be an ArcStr to make the allocation explicit.
    fn text<F>(self, text: F) -> Self
    where
        F: Fn() -> ArcStr + 'static;
    /// Use a mutable reference to the label.
    ///
    /// It is worth noting that the `use_fn` runs inside an [`Effect`].
    fn label_mut<F>(self, use_fn: F) -> Self
    where
        F: FnMut(WidgetMut<Label>) + 'static;
}

impl NewRadioButtonExt for NewWidget<RadioButton> {
    fn checked<C>(self, checked: C) -> Self
    where
        C: Fn() -> bool + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            RadioButton::set_checked(&mut this, checked());
        })
    }

    fn text<F>(self, text: F) -> Self
    where
        F: Fn() -> ArcStr + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            RadioButton::set_text(&mut this, text());
        })
    }

    fn label_mut<F>(self, mut use_fn: F) -> Self
    where
        F: FnMut(WidgetMut<Label>) + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            use_fn(RadioButton::label_mut(&mut this));
        })
    }
}
