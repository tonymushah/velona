//! Various [`RadioButton`] implementations
//!
//! The most important thing in the module is the [`NewRadioButtonExt`]
//! which is implemented for [`NewWidget<RadioButton>`].
//!
//! [`NewWidget<Radio>`] also implement the [`NewLabelExt`] trait
//! (since a [radio](RadioButton) is just a [label](Label) wrapper).
//!
//! _See the [widget](RadioButton) documentation for more information_.

use std::mem::{Discriminant, discriminant};

use masonry::{
    TextAlign,
    core::{ArcStr, NewWidget, StyleProperty, WidgetMut},
    widgets::{Label, RadioButton},
};

#[cfg(doc)]
use velona_core::reactive::effect::Effect;

use crate::{NewWidgetExt, widgets::label::NewLabelExt};

/// A [new](NewWidget) [`RadioButton`] extension trait.
pub trait NewRadioButtonExt {
    /// [Check or uncheck the box](RadioButton::set_checked) reactively.
    fn checked<C>(self, checked: C) -> Self
    where
        C: Fn() -> bool + 'static;
    /// Use a mutable reference to the label.
    ///
    /// It is worth noting that the `use_fn` runs inside an [`Effect`].
    fn use_label_mut<F>(self, use_fn: F) -> Self
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

    fn use_label_mut<F>(self, mut use_fn: F) -> Self
    where
        F: FnMut(WidgetMut<Label>) + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            use_fn(RadioButton::label_mut(&mut this));
        })
    }
}

impl NewLabelExt for NewWidget<RadioButton> {
    fn text<S, T>(self, text: S) -> Self
    where
        S: Fn() -> T + 'static,
        T: Into<ArcStr>,
    {
        self.use_label_mut(move |mut this| {
            Label::set_text(&mut this, text());
        })
    }

    fn style_opt<S, T>(self, style: S) -> Self
    where
        S: Fn() -> Option<T> + 'static,
        T: Into<StyleProperty>,
    {
        self.use_reactive_widget_mut_with_effect_val::<_, Discriminant<StyleProperty>>(
            move |mut this, old_style| {
                let mut this = RadioButton::label_mut(&mut this);
                if let Some(old_style) = old_style {
                    Label::remove_style(&mut this, old_style);
                }
                if let Some(style) = style() {
                    Label::insert_style(&mut this, style)
                        .as_ref()
                        .map(discriminant)
                } else {
                    None
                }
            },
        )
    }
    fn style<S, T>(self, style: S) -> Self
    where
        S: Fn() -> T + 'static,
        T: Into<StyleProperty>,
    {
        self.style_opt(move || Some(style()))
    }

    fn hint<S>(self, hint: S) -> Self
    where
        S: Fn() -> bool + 'static,
    {
        self.use_label_mut(move |mut this| {
            Label::set_hint(&mut this, hint());
        })
    }

    fn text_alignment<S>(self, align: S) -> Self
    where
        S: Fn() -> TextAlign + 'static,
    {
        // {
        //     self.widget = Box::new(self.widget.with_text_alignment(untrack(&align)));
        // }
        self.use_label_mut(move |mut this| {
            Label::set_text_alignment(&mut this, align());
        })
    }
}
