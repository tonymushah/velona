//! Various [`RadioButton`] implementations
//!
//! The most important thing in the module is the [`NewRadioButtonExt`]
//! which is implemented for [`NewWidget<RadioButton>`].
//!
//! [`NewWidget<Radio>`] also implement the [`NewLabelExt`] trait
//! (since a [radio](RadioButton) is just a [label](Label) wrapper).
//!
//! _See the [widget](RadioButton) documentation for more information_.

use std::mem::Discriminant;

use masonry::{
    TextAlign,
    core::{ArcStr, NewWidget, StyleProperty, WidgetMut},
    widgets::{Label, RadioButton},
};

#[cfg(doc)]
use velona_core::reactive::effect::Effect;
use velona_core::widgets::UseWidgetValResult;

use crate::{
    NewWidgetExt,
    utils::text_style::{apply_label_style_actions, get_style_opt_action},
    widgets::label::NewLabelExt,
};

/// A [new](NewWidget) [`RadioButton`] extension trait.
pub trait NewRadioButtonExt {
    /// [Check or uncheck the box](RadioButton::set_checked) reactively.
    fn checked<C>(self, checked: C) -> Self
    where
        C: Fn() -> bool + 'static;
    /// Use a mutable reference to the label.
    ///
    /// It is worth noting that only the `use_fn` runs inside an [`Effect`].
    fn use_label_mut<Vfn, Efn, V, O>(self, val_fn: Vfn, edit_fn: Efn) -> Self
    where
        Efn: FnMut(WidgetMut<'_, Label>, V) + 'static,
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static;
}

impl NewRadioButtonExt for NewWidget<RadioButton> {
    fn checked<C>(self, checked: C) -> Self
    where
        C: Fn() -> bool + 'static,
    {
        self.use_widget_mut(checked, |mut this, checked| {
            RadioButton::set_checked(&mut this, checked);
        })
    }

    fn use_label_mut<Vfn, Efn, V, O>(self, val_fn: Vfn, mut edit_fn: Efn) -> Self
    where
        Efn: FnMut(WidgetMut<'_, Label>, V) + 'static,
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static,
    {
        self.use_widget_mut_val(val_fn, move |mut this, val| {
            edit_fn(RadioButton::label_mut(&mut this), val);
        })
    }
}

impl NewLabelExt for NewWidget<RadioButton> {
    fn text<S, T>(self, text: S) -> Self
    where
        S: Fn() -> T + 'static,
        T: Into<ArcStr>,
    {
        self.use_label_mut(
            move |_| UseWidgetValResult::to_edit_fn(text().into()),
            |mut this, text| {
                Label::set_text(&mut this, text);
            },
        )
    }

    fn style_opt<S, T>(self, style: S) -> Self
    where
        S: Fn() -> Option<T> + 'static,
        T: Into<StyleProperty>,
    {
        self.use_label_mut(
            move |old_style: Option<Discriminant<StyleProperty>>| {
                let new_style = style().map(Into::<StyleProperty>::into);
                get_style_opt_action(old_style, new_style)
            },
            apply_label_style_actions,
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
        self.use_label_mut(
            move |_| UseWidgetValResult::to_edit_fn(hint()),
            |mut this, hint| {
                Label::set_hint(&mut this, hint);
            },
        )
    }

    fn text_alignment<S>(self, align: S) -> Self
    where
        S: Fn() -> TextAlign + 'static,
    {
        // {
        //     self.widget = Box::new(self.widget.with_text_alignment(untrack(&align)));
        // }
        self.use_label_mut(
            move |_| UseWidgetValResult::to_edit_fn(align()),
            |mut this, align| {
                Label::set_text_alignment(&mut this, align);
            },
        )
    }
}
