//! Various [`Split`] implementations.
//!
//! The most important thing in the module is the [`NewSplitExt`]
//! which is implemented for [`NewWidget<Split>`].
//!
//! _See the [widget](Split) documentation for more information_.

use imaging::kurbo::Axis;
use masonry::{
    core::{FromDynWidget, NewWidget, Widget, WidgetMut},
    imaging,
    layout::Length,
    widgets::{Split, SplitPoint},
};
#[cfg(doc)]
use velona_core::reactive::effect::Effect;
use velona_core::widgets::UseWidgetValResult;

use crate::NewWidgetExt;

/// A utility struct for [`Split::set_min_lengths`].
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SplitMinLengths {
    pub first: Length,
    pub second: Length,
}

impl SplitMinLengths {
    pub fn apply<A, B>(self, this: &mut WidgetMut<Split<A, B>>)
    where
        A: Widget + FromDynWidget + ?Sized,
        B: Widget + FromDynWidget + ?Sized,
    {
        Split::set_min_lengths(this, self.first, self.second);
    }
}

/// A [new](NewWidget) [`Split`] trait extension.
// TODO add example
pub trait NewSplitExt<ChildA, ChildB>
where
    ChildA: Widget + FromDynWidget + ?Sized,
    ChildB: Widget + FromDynWidget + ?Sized,
{
    /// Replaces the [first child widget](Split::set_child1) with a new one
    /// reactively.
    fn child1<C>(self, child1: C) -> Self
    where
        C: Fn() -> NewWidget<ChildA> + 'static;
    /// Replaces the [second child widget](Split::set_child2) with a new one
    /// reactively
    fn child2<C>(self, child2: C) -> Self
    where
        C: Fn() -> NewWidget<ChildB> + 'static;
    /// Use a mutable reference to the first child widget.
    ///
    /// It is worth noting that only the `val_fn` runs inside an [`Effect`].
    fn use_child1<Vfn, Efn, V, O>(self, val_fn: Vfn, edit_fn: Efn) -> Self
    where
        Efn: FnMut(WidgetMut<'_, ChildA>, V) + 'static,
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static;
    /// Use a mutable reference to the second child widget.
    ///
    /// It is worth noting that only the `val_fn` runs inside an [`Effect`].
    fn use_child2<Vfn, Efn, V, O>(self, val_fn: Vfn, edit_fn: Efn) -> Self
    where
        Efn: FnMut(WidgetMut<'_, ChildB>, V) + 'static,
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static;
    /// Sets the [split axis](Split::set_split_axis) reactively.
    fn split_axis<A>(self, split_axis: A) -> Self
    where
        A: Fn() -> Axis + 'static;
    /// Sets the [split point](Split::set_split_point) as a fraction of the split axis.
    ///
    /// The value must be between `0.0` and `1.0`, inclusive. The default split point is `0.5`.
    fn split_point<P>(self, split_point: P) -> Self
    where
        P: Fn() -> SplitPoint + 'static;
    /// Set the [minimum lengths](Split::set_min_lengths) for both sides of the split axis
    /// reactively.
    fn min_lengths<Ml>(self, min_lengths: Ml) -> Self
    where
        Ml: Fn() -> SplitMinLengths + 'static;
    /// Sets the thickness of the splitter bar
    /// reactively.
    ///
    /// The default splitter bar thickness is `6.0`.
    fn bar_thickness<B>(self, bar_thickness: B) -> Self
    where
        B: Fn() -> Length + 'static;
    /// Sets the [minimum thickness](Split::set_min_bar_area) of the splitter bar area
    /// reactively.
    ///
    /// The minimum splitter bar area defines the minimum thickness of the area
    /// where pointer hit detection is done for the splitter bar.
    /// The final hit detection area thickness is either this minimum or the splitter bar thickness,
    /// whichever is greater.
    ///
    /// This can be useful when you want to use a very narrow visual splitter bar,
    /// but don’t want to sacrifice user experience by making it hard to click on.
    ///
    /// The default minimum splitter bar area thickness is `6.0`.
    fn min_bar_area<B>(self, min_bar_area: B) -> Self
    where
        B: Fn() -> Length + 'static;
    /// [Sets whether the split point can be changed by dragging](Split::set_draggable)
    /// reactively.
    fn draggable<D>(self, draggable: D) -> Self
    where
        D: Fn() -> bool + 'static;
    /// # Reactive version of [`Split::set_bar_solid`].
    ///
    /// Sets whether the splitter bar is drawn as a solid rectangle.
    ///
    ///
    ///
    /// If this is `false` (the default), the bar will be drawn as two parallel lines.
    fn bar_solid<B>(self, bar_solid: B) -> Self
    where
        B: Fn() -> bool + 'static;
}

impl<ChildA, ChildB> NewSplitExt<ChildA, ChildB> for NewWidget<Split<ChildA, ChildB>>
where
    ChildA: Widget + FromDynWidget + ?Sized,
    ChildB: Widget + FromDynWidget + ?Sized,
{
    fn child1<C>(self, child1: C) -> Self
    where
        C: Fn() -> NewWidget<ChildA> + 'static,
    {
        self.use_widget_mut(child1, |mut this, child1| {
            Split::set_child1(&mut this, child1);
        })
    }

    fn child2<C>(self, child2: C) -> Self
    where
        C: Fn() -> NewWidget<ChildB> + 'static,
    {
        self.use_widget_mut(child2, |mut this, child2| {
            Split::set_child2(&mut this, child2);
        })
    }

    fn split_axis<A>(self, split_axis: A) -> Self
    where
        A: Fn() -> Axis + 'static,
    {
        self.use_widget_mut(split_axis, |mut this, split_axis| {
            Split::set_split_axis(&mut this, split_axis);
        })
    }

    fn split_point<P>(self, split_point: P) -> Self
    where
        P: Fn() -> SplitPoint + 'static,
    {
        self.use_widget_mut(split_point, |mut this, split_point| {
            Split::set_split_point(&mut this, split_point);
        })
    }

    fn min_lengths<Ml>(self, min_lengths: Ml) -> Self
    where
        Ml: Fn() -> SplitMinLengths + 'static,
    {
        self.use_widget_mut(min_lengths, |mut this, min_lengths| {
            min_lengths.apply(&mut this);
        })
    }

    fn bar_thickness<B>(self, bar_thickness: B) -> Self
    where
        B: Fn() -> Length + 'static,
    {
        self.use_widget_mut(bar_thickness, |mut this, bar_thickness| {
            Split::set_bar_thickness(&mut this, bar_thickness);
        })
    }

    fn min_bar_area<B>(self, min_bar_area: B) -> Self
    where
        B: Fn() -> Length + 'static,
    {
        self.use_widget_mut(min_bar_area, |mut this, min_bar_area| {
            Split::set_min_bar_area(&mut this, min_bar_area);
        })
    }

    fn draggable<D>(self, draggable: D) -> Self
    where
        D: Fn() -> bool + 'static,
    {
        self.use_widget_mut(draggable, |mut this, draggable| {
            Split::set_draggable(&mut this, draggable);
        })
    }

    fn bar_solid<B>(self, bar_solid: B) -> Self
    where
        B: Fn() -> bool + 'static,
    {
        self.use_widget_mut(bar_solid, |mut this, bar_solid| {
            Split::set_bar_solid(&mut this, bar_solid);
        })
    }

    fn use_child1<Vfn, Efn, V, O>(self, val_fn: Vfn, mut edit_fn: Efn) -> Self
    where
        Efn: FnMut(WidgetMut<'_, ChildA>, V) + 'static,
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static,
    {
        self.use_widget_mut_val(val_fn, move |mut this, val| {
            edit_fn(Split::child1_mut(&mut this), val);
        })
    }

    fn use_child2<Vfn, Efn, V, O>(self, val_fn: Vfn, mut edit_fn: Efn) -> Self
    where
        Efn: FnMut(WidgetMut<'_, ChildB>, V) + 'static,
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static,
    {
        self.use_widget_mut_val(val_fn, move |mut this, val| {
            edit_fn(Split::child2_mut(&mut this), val);
        })
    }
}
