use imaging::kurbo::Vec2;
use masonry::{
    core::NewWidget,
    widgets::{BadgePlacement, Badged},
};
use reactive_graph::effect::Effect;

use crate::{AnyNewWidget, NewWidgetExt, utils::ConsumeResult};

/// A [`NewWidget<Badged>`] trait extension
pub trait NewBadgedTrait {
    /// Change the badged [`content`](Badged::set_content) reactively.
    fn content<C>(self, content_fn: C) -> Self
    where
        C: Fn() -> AnyNewWidget + 'static;
    /// Change the badged [`badge`](Badged::set_badge) reactively.
    ///
    /// The current badge will be cleared if the badge_fn return `None`.
    fn badge<B>(self, badge_fn: B) -> Self
    where
        B: Fn() -> Option<AnyNewWidget> + 'static;
    /// Change the [badge placement](Badged::set_badge_placement) reactively.
    fn badge_placement<P>(self, placement_fn: P) -> Self
    where
        P: Fn() -> BadgePlacement + 'static;
    /// Change the [badge offset](Badged::set_badge_offset) reactively.
    fn badge_offset<O>(self, offset_fn: O) -> Self
    where
        O: Fn() -> Vec2 + 'static;
}

impl NewBadgedTrait for NewWidget<Badged> {
    fn content<C>(self, content_fn: C) -> Self
    where
        C: Fn() -> AnyNewWidget + 'static,
    {
        let b_ref = self.create_velona_ref();
        Effect::new(move || {
            let content = content_fn();
            b_ref
                .edit_local_now(|mut this| {
                    Badged::set_content(&mut this, content);
                })
                .consume_with_log_err();
        });
        self
    }

    fn badge<B>(self, badge_fn: B) -> Self
    where
        B: Fn() -> Option<AnyNewWidget> + 'static,
    {
        let b_ref = self.create_velona_ref();
        Effect::new(move || {
            let badge = badge_fn();

            b_ref
                .edit_local_now(|mut this| {
                    if let Some(badge) = badge {
                        Badged::set_badge(&mut this, badge);
                    } else if this.widget.has_badge() {
                        Badged::clear_badge(&mut this);
                    }
                })
                .consume_with_log_err();
        });
        self
    }

    fn badge_placement<P>(self, placement_fn: P) -> Self
    where
        P: Fn() -> BadgePlacement + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Badged::set_badge_placement(&mut this, placement_fn());
        })
    }

    fn badge_offset<O>(self, offset_fn: O) -> Self
    where
        O: Fn() -> Vec2 + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Badged::set_badge_offset(&mut this, offset_fn());
        })
    }
}
