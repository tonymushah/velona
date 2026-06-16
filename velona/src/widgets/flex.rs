use imaging::kurbo::Axis;
use masonry::{
    core::NewWidget,
    properties::types::{CrossAxisAlignment, MainAxisAlignment},
    widgets::Flex,
};

use crate::NewWidgetExt;

/// a [new](NewWidget) [`Flex`] trait extension.
// TODO add example
pub trait NewFlexExt {
    /// Set the [flex direction](Flex::set_direction) reactively.
    fn direction<D>(self, direction: D) -> Self
    where
        D: Fn() -> Axis + 'static;
    /// Set the [flex `cross_axis_alignment`](Flex::set_cross_axis_alignment) reactively.
    fn cross_axis_alignment<C>(self, alignment: C) -> Self
    where
        C: Fn() -> CrossAxisAlignment + 'static;
    /// Set the [flex `main_axis_alignment`](Flex::set_main_axis_alignment) reactively.
    fn main_axis_alignment<M>(self, alignment: M) -> Self
    where
        M: Fn() -> MainAxisAlignment + 'static;
}

impl NewFlexExt for NewWidget<Flex> {
    fn direction<D>(self, direction: D) -> Self
    where
        D: Fn() -> Axis + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Flex::set_direction(&mut this, direction());
        })
    }

    fn cross_axis_alignment<C>(self, alignment: C) -> Self
    where
        C: Fn() -> CrossAxisAlignment + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Flex::set_cross_axis_alignment(&mut this, alignment());
        })
    }

    fn main_axis_alignment<M>(self, alignment: M) -> Self
    where
        M: Fn() -> MainAxisAlignment + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Flex::set_main_axis_alignment(&mut this, alignment());
        })
    }
}
