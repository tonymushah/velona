// TODO add badge components for
// - [`Badge::with_text`]
// - [`Badge::count`]
// - [`Badge::count_with_overflow`]
// - [`Badge::count_non_zero`]

use masonry::{core::NewWidget, widgets::Badge};

use crate::{AnyNewWidget, NewWidgetExt};

pub trait NewBadgeExt {
    fn child<C>(self, child_fn: C) -> Self
    where
        C: Fn() -> AnyNewWidget + 'static;
}

impl NewBadgeExt for NewWidget<Badge> {
    fn child<C>(self, child_fn: C) -> Self
    where
        C: Fn() -> AnyNewWidget + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Badge::set_child(&mut this, child_fn());
        })
    }
}
