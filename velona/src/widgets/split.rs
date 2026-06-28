use imaging::kurbo::Axis;
use masonry::{
    core::{FromDynWidget, NewWidget, Widget, WidgetMut},
    layout::Length,
    widgets::{Split, SplitPoint},
};

use reactive_graph::effect::Effect;

use crate::{NewWidgetExt, utils::ConsumeResult};

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
    /// It is worth noting that the `use_fn` runs inside an [`Effect`].
    fn use_child1<U>(self, use_fn: U) -> Self
    where
        U: FnMut(WidgetMut<ChildA>) + 'static;
    /// Use a mutable reference to the second child widget.
    ///
    /// It is worth noting that the `use_fn` runs inside an [`Effect`].
    fn use_child2<U>(self, use_fn: U) -> Self
    where
        U: FnMut(WidgetMut<ChildB>) + 'static;
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
        let s_ref = self.create_velona_ref();
        Effect::new(move || {
            let child = child1();
            s_ref
                .edit_local_now(|mut this| {
                    Split::set_child1(&mut this, child);
                })
                .consume_with_log_err();
        });
        self
    }

    fn child2<C>(self, child2: C) -> Self
    where
        C: Fn() -> NewWidget<ChildB> + 'static,
    {
        let s_ref = self.create_velona_ref();
        Effect::new(move || {
            let child = child2();
            s_ref
                .edit_local_now(|mut this| {
                    Split::set_child2(&mut this, child);
                })
                .consume_with_log_err();
        });
        self
    }

    fn use_child1<U>(self, mut use_fn: U) -> Self
    where
        U: FnMut(WidgetMut<ChildA>) + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| use_fn(Split::child1_mut(&mut this)))
    }

    fn use_child2<U>(self, mut use_fn: U) -> Self
    where
        U: FnMut(WidgetMut<ChildB>) + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| use_fn(Split::child2_mut(&mut this)))
    }

    fn split_axis<A>(self, split_axis: A) -> Self
    where
        A: Fn() -> Axis + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Split::set_split_axis(&mut this, split_axis());
        })
    }

    fn split_point<P>(self, split_point: P) -> Self
    where
        P: Fn() -> SplitPoint + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Split::set_split_point(&mut this, split_point());
        })
    }

    fn min_lengths<Ml>(self, min_lengths: Ml) -> Self
    where
        Ml: Fn() -> SplitMinLengths + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            min_lengths().apply(&mut this);
        })
    }

    fn bar_thickness<B>(self, bar_thickness: B) -> Self
    where
        B: Fn() -> Length + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Split::set_bar_thickness(&mut this, bar_thickness());
        })
    }

    fn min_bar_area<B>(self, min_bar_area: B) -> Self
    where
        B: Fn() -> Length + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Split::set_min_bar_area(&mut this, min_bar_area());
        })
    }

    fn draggable<D>(self, draggable: D) -> Self
    where
        D: Fn() -> bool + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Split::set_draggable(&mut this, draggable());
        })
    }

    fn bar_solid<B>(self, bar_solid: B) -> Self
    where
        B: Fn() -> bool + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Split::set_bar_solid(&mut this, bar_solid());
        })
    }
}
