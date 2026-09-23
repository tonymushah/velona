//! Various [`Portal`] implementations.
//!
//! The most important thing in the module is the [`NewPortalExt`]
//! which is implemented for [`NewWidget<Portal>`].
//!
//! _See the [widget](Portal) documentation for more information_.
use imaging::kurbo::{Rect, Vec2};
use masonry::{
    core::{FromDynWidget, NewWidget, Widget, WidgetMut},
    imaging,
    kurbo::Point,
    widgets::{Portal, ScrollBar},
};

#[cfg(doc)]
use velona_core::reactive::effect::Effect;
use velona_core::widgets::UseWidgetValResult;

use super::NewWidgetExt;

/// A [new](NewWidget) [`Portal`] trait extension.
// TODO add example
pub trait NewPortalExt<W>
where
    W: Widget + FromDynWidget + ?Sized,
{
    /// Use the [`Portal` horizontal scrollbar](Portal::horizontal_scrollbar_mut).
    ///
    /// It is worth noting that only the `val_fn` runs inside an [`Effect`].
    fn use_horizontal_scrollbar_mut<Vfn, Efn, V, O>(self, val_fn: Vfn, edit_fn: Efn) -> Self
    where
        Efn: FnMut(WidgetMut<'_, ScrollBar>, V) + 'static,
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static;
    /// Use the [`Portal` vertical scrollbar](Portal::vertical_scrollbar_mut).
    ///
    /// It is worth noting that the only `val_fn` runs inside an [`Effect`].
    fn use_vertical_scrollbar_mut<Vfn, Efn, V, O>(self, val_fn: Vfn, edit_fn: Efn) -> Self
    where
        Efn: FnMut(WidgetMut<'_, ScrollBar>, V) + 'static,
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static;
    /// Set the [`Portal` horizontal constrain](Portal::set_constrain_horizontal) reactively.
    fn constrain_horizontal<C>(self, contrain: C) -> Self
    where
        C: Fn() -> bool + 'static;
    /// Set the [`Portal` vertical constrain](Portal::set_constrain_vertical) reactively.
    fn constrain_vertical<C>(self, contrain: C) -> Self
    where
        C: Fn() -> bool + 'static;
    /// Set the [`Portal` _content_must_fill_](Portal::set_content_must_fill) reactively.
    fn content_must_fill<C>(self, must_fill: C) -> Self
    where
        C: Fn() -> bool + 'static;
    /// Set the [`Portal` viewport position](Portal::set_constrain_vertical) reactively.
    fn viewport_pos<C>(self, pos: C) -> Self
    where
        C: Fn() -> Point + 'static;
    /// A reactive version of [`Portal::pan_viewport_by`].
    fn pan_viewport_by<C>(self, translation: C) -> Self
    where
        C: Fn() -> Vec2 + 'static;
    /// A reactive version of [`Portal::pan_viewport_to`].
    fn pan_viewport_to<C>(self, target: C) -> Self
    where
        C: Fn() -> Rect + 'static;
}

impl<W> NewPortalExt<W> for NewWidget<Portal<W>>
where
    W: Widget + FromDynWidget + ?Sized,
{
    fn constrain_horizontal<C>(self, contrain: C) -> Self
    where
        C: Fn() -> bool + 'static,
    {
        self.use_widget_mut(contrain, |mut this, contrain| {
            Portal::set_constrain_horizontal(&mut this, contrain)
        })
    }

    fn constrain_vertical<C>(self, contrain: C) -> Self
    where
        C: Fn() -> bool + 'static,
    {
        self.use_widget_mut(contrain, |mut this, contrain| {
            Portal::set_constrain_vertical(&mut this, contrain)
        })
    }

    fn content_must_fill<C>(self, must_fill: C) -> Self
    where
        C: Fn() -> bool + 'static,
    {
        self.use_widget_mut(must_fill, |mut this, must_fill| {
            Portal::set_content_must_fill(&mut this, must_fill)
        })
    }

    fn viewport_pos<C>(self, pos: C) -> Self
    where
        C: Fn() -> Point + 'static,
    {
        self.use_widget_mut(pos, |mut this, pos| {
            Portal::set_viewport_pos(&mut this, pos);
        })
    }

    fn pan_viewport_by<C>(self, translation: C) -> Self
    where
        C: Fn() -> Vec2 + 'static,
    {
        self.use_widget_mut(translation, |mut this, translation| {
            Portal::pan_viewport_by(&mut this, translation);
        })
    }

    fn pan_viewport_to<C>(self, target: C) -> Self
    where
        C: Fn() -> Rect + 'static,
    {
        self.use_widget_mut(target, |mut this, target| {
            Portal::pan_viewport_to(&mut this, target);
        })
    }

    fn use_horizontal_scrollbar_mut<Vfn, Efn, V, O>(self, val_fn: Vfn, mut edit_fn: Efn) -> Self
    where
        Efn: FnMut(WidgetMut<'_, ScrollBar>, V) + 'static,
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static,
    {
        self.use_widget_mut_val(val_fn, move |mut this, val| {
            edit_fn(Portal::horizontal_scrollbar_mut(&mut this), val)
        })
    }

    fn use_vertical_scrollbar_mut<Vfn, Efn, V, O>(self, val_fn: Vfn, mut edit_fn: Efn) -> Self
    where
        Efn: FnMut(WidgetMut<'_, ScrollBar>, V) + 'static,
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static,
    {
        self.use_widget_mut_val(val_fn, move |mut this, val| {
            edit_fn(Portal::vertical_scrollbar_mut(&mut this), val)
        })
    }
}
