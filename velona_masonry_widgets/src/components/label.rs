use masonry::{
    core::{ArcStr, NewWidget, Widget},
    widgets::Label,
};
use velona_core::reactive::graph::untrack;

use crate::widgets::label::NewLabelExt;

// TODO move to [`crate::components`]
pub fn label<S, T>(text: S) -> NewWidget<Label>
where
    S: Fn() -> T + 'static,
    T: Into<ArcStr>,
{
    Label::new(untrack(&text)).prepare().text(text)
}
