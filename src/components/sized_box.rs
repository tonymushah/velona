//! A reactive widget

use masonry::{
    core::{NewWidget, Widget},
    widgets::SizedBox,
};
use reactive_graph::effect::Effect;

use crate::{AnyNewWidget, NewWidgetExt};

/// The function signature and the signature might sound dumb,
/// but this allows you to show widgets based on conditional logic, ect...
pub fn sized_box<F>(mut widget_fn: F) -> NewWidget<SizedBox>
where
    F: FnMut() -> AnyNewWidget + 'static,
{
    let indexed = SizedBox::empty()
        // .with_child(untrack(&mut widget_fn))
        .with_auto_id();
    let indexed_ref = indexed.create_velona_ref();
    Effect::new(move || {
        let new_widget = widget_fn();
        if let Err(err) = indexed_ref.edit_local_now(|mut widget_mut| {
            SizedBox::set_child(&mut widget_mut, new_widget);
        }) {
            log::error!("cannot edit fragment inner {err}");
        }
    });
    indexed
}
