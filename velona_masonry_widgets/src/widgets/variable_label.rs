//! Various [`VariableLabel`] implementations.
//!
//! The most important thing in the module is the [`NewVariableLabelExt`]
//! which is implemented for [`NewWidget<VariableLabel>`].
//!
//! [`NewWidget<VariableLabel>`] also implements the [`NewLabelExt`] trait.
//!
//! _See the [widget](VariableLabel) documentation for more information_.

use std::mem::Discriminant;

use masonry::{
    TextAlign,
    core::{ArcStr, NewWidget, StyleProperty, WidgetMut},
    widgets::{Label, VariableLabel},
};

#[cfg(doc)]
use velona_core::reactive::effect::Effect;
use velona_core::widgets::UseWidgetValResult;

use crate::{
    NewWidgetExt,
    utils::text_style::{apply_label_style_actions, get_style_opt_action},
    widgets::label::NewLabelExt,
};

/// A utility struct for [`VariableLabel::set_target_weight`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VariableLabelTargetWeight {
    pub target: f32,
    pub over_millis: f32,
}

impl VariableLabelTargetWeight {
    /// Apply this [`VariableLabelTargetWeight`].
    pub fn apply(self, this: &mut WidgetMut<VariableLabel>) {
        VariableLabel::set_target_weight(this, self.target, self.over_millis);
    }
}

/// A [new](NewWidget) [`VariableLabel`] trait extension.
// TODO add example
pub trait NewVariableLabelExt {
    /// Use the underlying label for this widget.
    ///
    /// It is worth noting that only the `val_fn` will run inside an [`Effect`].
    fn use_label_mut<Vfn, Efn, V, O>(self, val_fn: Vfn, edit_fn: Efn) -> Self
    where
        Efn: FnMut(WidgetMut<'_, Label>, V) + 'static,
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static;
    /// Sets the weight which this font will target.
    ///
    /// The reactive variant of [`set_target_weight`](VariableLabel::set_target_weight).
    fn target_weight<T>(self, target_weight: T) -> Self
    where
        T: Fn() -> VariableLabelTargetWeight + 'static;
}

impl NewVariableLabelExt for NewWidget<VariableLabel> {
    fn target_weight<T>(self, target_weight: T) -> Self
    where
        T: Fn() -> VariableLabelTargetWeight + 'static,
    {
        self.use_widget_mut(target_weight, |mut this, target_weight| {
            target_weight.apply(&mut this);
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
            edit_fn(VariableLabel::label_mut(&mut this), val);
        })
    }
}

impl NewLabelExt for NewWidget<VariableLabel> {
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

    fn style<S, T>(self, style: S) -> Self
    where
        S: Fn() -> T + 'static,
        T: Into<StyleProperty>,
    {
        self.style_opt(move || Some(style()))
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
        self.use_label_mut(
            move |_| UseWidgetValResult::to_edit_fn(align()),
            |mut this, alignment| {
                Label::set_text_alignment(&mut this, alignment);
            },
        )
    }
}
