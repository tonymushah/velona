//! A reactive widget

use masonry::{core::Widget, widgets::IndexedStack};
use reactive_graph::effect::Effect;

use crate::{AnyNewWidget, NewWidgetExt};

/// The function signature and the signature might sound dumb,
/// but this allows you to show widgets based on conditional logic, ect...
pub fn fragment<F>(mut widget_fn: F) -> AnyNewWidget
where
    F: FnMut() -> AnyNewWidget + 'static,
{
    let indexed = IndexedStack::new()
        // .with_child(untrack(&mut widget_fn))
        .with_auto_id();
    let indexed_ref = indexed.create_velona_ref();
    Effect::new(move || {
        let new_widget = widget_fn();
        if let Err(err) = indexed_ref.edit_local_now(|mut widget_mut| {
            if widget_mut.widget.is_empty() {
                IndexedStack::add_child(&mut widget_mut, new_widget);
            } else {
                IndexedStack::remove_child(&mut widget_mut, 0);
                IndexedStack::add_child(&mut widget_mut, new_widget);
            }
        }) {
            log::error!("cannot edit fragment inner {err}");
        }
    });
    indexed.erased()
}
