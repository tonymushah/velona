//! Various [`Badged`] trait implementations
//!
//! The most important thing in the module is the [`NewBadgedTrait`]
//! which is implemented for [`NewWidget<Badged>`].
//!
//! _You will also notice that there is no `badge` module for the [`Badge`][badge-widget]._
//!
//! The [`Badge`][badge-widget] doesn't have anything interesting to have a dedicated trait
//! since a [new](NewWidget) [its][badge-widget] already implement:
//! - [`SingleChildWidget`] for using its child.
//! - [`ReactiveSingleChildExt`] for setting its child reactively.
//!
//! _See the [widget](Badged) documentation for more information_.
//!
//! There is also the [`IntoBadged`] trait for transforming [`View`]s into a [`Badged`].
//!
//! [badge-widget]: masonry::widgets::Badge
//! [`SingleChildWidget`]: super::SingleChildWidget
//! [`ReactiveSingleChildExt`]: super::ReactiveSingleChildExt

use masonry::imaging::kurbo::Vec2;
use masonry::{
    core::{NewWidget, Widget, WidgetMut},
    widgets::{BadgePlacement, Badged},
};
use velona_core::AnyNewWidget;
#[cfg(doc)]
use velona_core::reactive::effect::Effect;
use velona_core::widgets::{UseWidgetValResult, View};

use crate::NewWidgetExt;

/// A [new](NewWidget) [`Badged`] trait extension
#[must_use]
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
    /// Use a mutable reference to the content widget.
    ///
    /// It is worth noting that only `val_fn` will run inside an [`Effect`].
    fn use_content_mut_val<Vfn, Efn, V, O>(self, val_fn: Vfn, edit_fn: Efn) -> Self
    where
        Efn: FnMut(WidgetMut<'_, dyn Widget>, V) + 'static,
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static;
    /// Use a mutable reference to the badge widget.
    ///
    /// It is worth noting that the `val_fn` will run inside an [`Effect`]
    fn use_badge_mut_val<Vfn, Efn, V, O>(self, val_fn: Vfn, edit_fn: Efn) -> Self
    where
        Efn: FnMut(Option<WidgetMut<'_, dyn Widget>>, V) + 'static,
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static;
}

impl NewBadgedTrait for NewWidget<Badged> {
    fn content<C>(self, content_fn: C) -> Self
    where
        C: Fn() -> AnyNewWidget + 'static,
    {
        self.use_widget_mut(content_fn, |mut this, content| {
            Badged::set_content(&mut this, content);
        })
    }

    fn badge<B>(self, badge_fn: B) -> Self
    where
        B: Fn() -> Option<AnyNewWidget> + 'static,
    {
        self.use_widget_mut(badge_fn, |mut this, badge| {
            if let Some(badge) = badge {
                Badged::set_badge(&mut this, badge);
            } else if this.widget.has_badge() {
                Badged::clear_badge(&mut this);
            }
        })
    }

    fn badge_placement<P>(self, placement_fn: P) -> Self
    where
        P: Fn() -> BadgePlacement + 'static,
    {
        self.use_widget_mut(placement_fn, |mut this, placement| {
            Badged::set_badge_placement(&mut this, placement);
        })
    }

    fn badge_offset<O>(self, offset_fn: O) -> Self
    where
        O: Fn() -> Vec2 + 'static,
    {
        self.use_widget_mut(offset_fn, |mut this, offset| {
            Badged::set_badge_offset(&mut this, offset);
        })
    }

    fn use_content_mut_val<Vfn, Efn, V, O>(self, val_fn: Vfn, mut edit_fn: Efn) -> Self
    where
        Efn: FnMut(WidgetMut<'_, dyn Widget>, V) + 'static,
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static,
    {
        self.use_widget_mut_val(val_fn, move |mut this, val| {
            edit_fn(Badged::content_mut(&mut this), val);
        })
    }

    fn use_badge_mut_val<Vfn, Efn, V, O>(self, val_fn: Vfn, mut edit_fn: Efn) -> Self
    where
        Efn: FnMut(Option<WidgetMut<'_, dyn Widget>>, V) + 'static,
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static,
    {
        self.use_widget_mut_val(val_fn, move |mut this, val| {
            edit_fn(Badged::badge_mut(&mut this), val);
        })
    }
}

pub trait IntoBadged {
    fn into_badge_content<V>(self, badge: Option<V>) -> Badged
    where
        V: View + 'static;
    fn into_badge_content_with_reactive_badge<Vfn, V>(self, badge: Vfn) -> NewWidget<Badged>
    where
        V: View + 'static,
        Vfn: Fn() -> Option<V> + 'static;

    fn into_badge<V>(self, badge_content: V) -> Badged
    where
        V: View + 'static;
}

impl<V1> IntoBadged for V1
where
    V1: View + 'static,
{
    fn into_badge_content<V>(self, badge: Option<V>) -> Badged
    where
        V: View + 'static,
    {
        Badged::new_optional(self.into_new_widget(), badge.map(|v| v.into_erased()))
    }

    fn into_badge<V>(self, badge_content: V) -> Badged
    where
        V: View + 'static,
    {
        Badged::new(badge_content.into_new_widget(), self.into_new_widget())
    }

    fn into_badge_content_with_reactive_badge<Vfn, V>(self, badge: Vfn) -> NewWidget<Badged>
    where
        V: View + 'static,
        Vfn: Fn() -> Option<V> + 'static,
    {
        Badged::new_optional(self.into_new_widget(), None)
            .prepare()
            .badge(move || badge().map(|v| v.into_erased()))
    }
}
