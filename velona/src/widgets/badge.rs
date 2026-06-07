// TODO add badge components for
// - [`Badge::with_text`]
// - [`Badge::count`]
// - [`Badge::count_with_overflow`]
// - [`Badge::count_non_zero`]

use masonry::{core::NewWidget, widgets::Badge};
use reactive_graph::effect::Effect;

use crate::{AnyNewWidget, NewWidgetExt};

/// A [`NewWidget<Badge>`] trait extension
pub trait NewBadgeExt {
    /// Make the [child badge](Badge::set_child) reactive
    fn child<C>(self, child_fn: C) -> Self
    where
        C: Fn() -> AnyNewWidget + 'static;
}

impl NewBadgeExt for NewWidget<Badge> {
    fn child<C>(self, child_fn: C) -> Self
    where
        C: Fn() -> AnyNewWidget + 'static,
    {
        let b_ref = self.create_velona_ref();
        Effect::new(move || {
            // NOTE we used use_reactive_widget_mut before
            // but that might block others `edit_local_now` in the nested childs
            let child = child_fn();
            let _ = b_ref
                .edit_local_now(|mut this| {
                    Badge::set_child(&mut this, child);
                })
                .inspect_err(|err| {
                    log::error!("cannot set badge child => {err}");
                });
        });
        self
    }
}
