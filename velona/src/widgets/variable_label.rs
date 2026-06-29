use masonry::{
    core::{NewWidget, WidgetMut},
    widgets::{Label, VariableLabel},
};

#[cfg(doc)]
use reactive_graph::effect::Effect;

use crate::NewWidgetExt;

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
