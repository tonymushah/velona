use masonry::{
    core::{ArcStr, NewWidget, Widget},
    widgets::Checkbox,
};
use velona_core::reactive::graph::untrack;

use crate::widgets::checkbox::NewCheckboxExt;

/// Create a new reactive checkbox.
pub fn checkbox<Cf, Tf, T>(checked: Cf, text: Tf) -> NewWidget<Checkbox>
where
    Cf: Fn() -> bool + 'static,
    Tf: Fn() -> T + 'static,
    T: Into<ArcStr>,
{
    Checkbox::new(untrack(&checked), untrack(&text))
        .prepare()
        .checked(checked)
        .text(text)
}
