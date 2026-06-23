use imaging::kurbo::{Rect, Vec2};
use masonry::{
    core::{NewWidget, Widget, WidgetMut},
    kurbo::Point,
    widgets::{Portal, ScrollBar},
};
use reactive_graph::effect::Effect;

use crate::{
    utils::ConsumeResult,
    widgets::{ReactiveSingleTypedChildExt, TypedSingleChildWidget},
};

use super::NewWidgetExt;

/// A [new](NewWidget) [`Portal`] trait extension.
// TODO add example
pub trait NewPortalExt<W>
where
    W: Widget + 'static,
{
    /// Use the [`Portal` horizontal scrollbar](Portal::horizontal_scrollbar_mut).
    ///
    /// It is worth noting that the `use_fn` runs inside an [`Effect`].
    fn use_horizontal_scrollbar_mut<U>(self, use_fn: U) -> Self
    where
        U: FnMut(WidgetMut<ScrollBar>) + 'static;
    /// Use the [`Portal` vertical scrollbar](Portal::vertical_scrollbar_mut).
    ///
    /// It is worth noting that the `use_fn` runs inside an [`Effect`].
    fn use_vertical_scrollbar_mut<U>(self, use_fn: U) -> Self
    where
        U: FnMut(WidgetMut<ScrollBar>) + 'static;

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
    W: Widget + 'static,
{
    fn use_horizontal_scrollbar_mut<U>(self, mut use_fn: U) -> Self
    where
        U: FnMut(WidgetMut<ScrollBar>) + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            use_fn(Portal::horizontal_scrollbar_mut(&mut this))
        })
    }

    fn use_vertical_scrollbar_mut<U>(self, mut use_fn: U) -> Self
    where
        U: FnMut(WidgetMut<ScrollBar>) + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            use_fn(Portal::vertical_scrollbar_mut(&mut this))
        })
    }

    fn constrain_horizontal<C>(self, contrain: C) -> Self
    where
        C: Fn() -> bool + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Portal::set_constrain_horizontal(&mut this, contrain())
        })
    }

    fn constrain_vertical<C>(self, contrain: C) -> Self
    where
        C: Fn() -> bool + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Portal::set_constrain_vertical(&mut this, contrain())
        })
    }

    fn content_must_fill<C>(self, must_fill: C) -> Self
    where
        C: Fn() -> bool + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Portal::set_content_must_fill(&mut this, must_fill())
        })
    }

    fn viewport_pos<C>(self, pos: C) -> Self
    where
        C: Fn() -> Point + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Portal::set_viewport_pos(&mut this, pos());
        })
    }

    fn pan_viewport_by<C>(self, translation: C) -> Self
    where
        C: Fn() -> Vec2 + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Portal::pan_viewport_by(&mut this, translation());
        })
    }

    fn pan_viewport_to<C>(self, target: C) -> Self
    where
        C: Fn() -> Rect + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Portal::pan_viewport_to(&mut this, target());
        })
    }
}

impl<W> TypedSingleChildWidget for NewWidget<Portal<W>>
where
    W: Widget + 'static,
{
    type Child = W;

    fn use_child<C>(self, mut use_child_fn: C) -> Self
    where
        C: FnMut(WidgetMut<'_, Self::Child>) + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| use_child_fn(Portal::child_mut(&mut this)))
    }
}

impl<W> ReactiveSingleTypedChildExt for NewWidget<Portal<W>>
where
    W: Widget + 'static,
{
    type Child = W;
    fn child<Cf>(self, child_fn: Cf) -> Self
    where
        Cf: Fn() -> NewWidget<Self::Child> + 'static,
    {
        let w_ref = self.create_velona_ref();
        Effect::new(move || {
            let new_child = child_fn();
            w_ref
                .edit_local_now(|mut this| {
                    Portal::set_child(&mut this, new_child);
                })
                .consume_with_log_err();
        });
        self
    }
}
