//! Various [`VariableLabel`] implementations.
//!
//! The most important thing in the module is the [`NewVariableLabelExt`]
//! which is implemented for [`NewWidget<VariableLabel>`].
//!
//! [`NewWidget<VariableLabel>`] also implements the [`NewLabelExt`] trait.
//!
//! _See the [widget](VariableLabel) documentation for more information_.

use std::mem::{Discriminant, discriminant};

use masonry::{
    TextAlign,
    core::{ArcStr, NewWidget, StyleProperty, WidgetMut},
    widgets::{Label, VariableLabel},
};

#[cfg(doc)]
use velona_core::reactive::effect::Effect;

use crate::{NewWidgetExt, widgets::label::NewLabelExt};

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
    /// It is worth noting that the `use_fn` will run inside an [`Effect`].
    fn use_label_mut<L>(self, use_fn: L) -> Self
    where
        L: FnMut(WidgetMut<Label>) + 'static;
    /// Sets the weight which this font will target.
    ///
    /// The reactive variant of [`set_target_weight`](VariableLabel::set_target_weight).
    fn target_weight<T>(self, target_weight: T) -> Self
    where
        T: Fn() -> VariableLabelTargetWeight + 'static;
}

impl NewVariableLabelExt for NewWidget<VariableLabel> {
    fn use_label_mut<L>(self, mut use_fn: L) -> Self
    where
        L: FnMut(WidgetMut<Label>) + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| use_fn(VariableLabel::label_mut(&mut this)))
    }

    fn target_weight<T>(self, target_weight: T) -> Self
    where
        T: Fn() -> VariableLabelTargetWeight + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            target_weight().apply(&mut this);
        })
    }
}

impl NewLabelExt for NewWidget<VariableLabel> {
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
                let mut this = VariableLabel::label_mut(&mut this);
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
        self.use_label_mut(move |mut this| {
            Label::set_text_alignment(&mut this, align());
        })
    }
}
