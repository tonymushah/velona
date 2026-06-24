use masonry::{
    core::{ArcStr, NewWidget, Widget},
    widgets::RadioButton,
};
use reactive_graph::graph::untrack;

use crate::widgets::radio::NewRadioButtonExt;

/// We move most the [`RadioButton`] custom component here.
///
/// For static `checked` status and `text`, use [`RadioButton::new`].
pub struct NewRadioButtonBuilder;

impl NewRadioButtonBuilder {
    /// Create a [`RadioButton`] with a reactive `text` and `checked` status
    pub fn create<C, T>(checked: C, text: T) -> NewWidget<RadioButton>
    where
        C: Fn() -> bool + 'static,
        T: Fn() -> ArcStr + 'static,
    {
        RadioButton::new(untrack(&checked), untrack(&text))
            .prepare()
            .checked(checked)
            .text(text)
    }
    /// Create a [`RadioButton`] with a static `text` and reactive `checked` status.
    pub fn create_with_static_text<C>(checked: C, text: ArcStr) -> NewWidget<RadioButton>
    where
        C: Fn() -> bool + 'static,
    {
        RadioButton::new(untrack(&checked), text)
            .prepare()
            .checked(checked)
    }
    /// Create a [`RadioButton`] with a reactive `text` and static `checked` status.
    pub fn create_with_static_checked<T>(checked: bool, text: T) -> NewWidget<RadioButton>
    where
        T: Fn() -> ArcStr + 'static,
    {
        RadioButton::new(checked, untrack(&text))
            .prepare()
            .text(text)
    }
}
