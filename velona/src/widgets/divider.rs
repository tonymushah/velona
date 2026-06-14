use imaging::kurbo::{Axis, Cap};
use masonry::{
    core::{NewWidget, Widget, WidgetMut},
    layout::Length,
    widgets::{DashFit, Divider, Placement},
};
use reactive_graph::effect::Effect;

use crate::{
    AnyNewWidget, NewWidgetExt,
    utils::ConsumeResult,
    widgets::{ReactiveSingleChildExt, SingleChildWidget},
};

/// A [new](NewWidget) [`Divider`] trait extension.
// TODO add example
pub trait NewDividerExt {
    /// Sets the [divider direction](Divider::set_direction) reactively.
    fn direction<A>(self, axis: A) -> Self
    where
        A: Fn() -> Axis + 'static;
    /// Sets the [divider thickness](Divider::set_thickness) reactively.
    ///
    /// If the `thickness` function return [`Some`] [`Length`], it will use [`Divider::set_thickness`],
    /// if [`None`], it will call [`Divider::set_hairline`] instead.
    fn thickness<T>(self, thickness: T) -> Self
    where
        T: Fn() -> Option<Length> + 'static;
    /// Sets the [divider `dash_fit`](Divider::set_dash_fit) reactively.
    fn dash_fit<D>(self, dash_fit: D) -> Self
    where
        D: Fn() -> DashFit + 'static;
    /// Sets the [divider `dash_pattern`](Divider::set_dash_pattern) reactively.
    ///
    /// See [`Divider::dash_pattern`] for more details.
    ///
    /// **Panics**
    ///
    /// Panics if `dash_pattern` contains an uneven number of entries of 3 or more and debug assertions are enabled.
    fn dash_pattern<D>(self, dash_pattern: D) -> Self
    where
        D: Fn() -> Vec<Length> + 'static;
    /// Sets the `cap` used both for start and end _reactively_.
    ///
    /// Use [`start_cap`](Self::start_cap) or [`ending_cap`](Self::ending_cap) to set different edge caps.
    ///
    /// Defaults to [`Cap::Butt`].
    ///
    /// It is not recommended to use [`cap`](Self::cap)
    /// and [`start_cap`](Self::start_cap)/[`end_cap`](Self::ending_cap) together.
    fn cap<C>(self, cap: C) -> Self
    where
        C: Fn() -> Cap + 'static;
    /// Sets the starting `cap`.
    ///
    /// Use [`cap`](Self::cap) to set the cap for both the start and the end.
    ///
    /// Defaults to [`Cap::Butt`].
    ///
    /// It is not recommended to use [`start_cap`](Self::start_cap) and [`cap`](Self::cap) together.
    fn start_cap<C>(self, cap: C) -> Self
    where
        C: Fn() -> Cap + 'static;
    /// Sets the ending `cap`.
    ///
    /// Use [`cap`](Self::cap) to set the cap for both the start and the end.
    ///
    /// Defaults to [`Cap::Butt`].
    ///
    /// It is not recommended to use [`ending_cap`](Self::ending_cap) and [`cap`](Self::cap) together.
    fn ending_cap<C>(self, cap: C) -> Self
    where
        C: Fn() -> Cap + 'static;
    /// Sets the content `placement` _reactively_.
    ///
    /// Defaults to [`Placement::Center`].
    fn placement<P>(self, placement: P) -> Self
    where
        P: Fn() -> Placement + 'static;
    /// Sets the [divider `content`](Divider::set_content) _reactively_.
    ///
    /// If the `content` function returns `None`, it will [clear the current content](Divider::clear_content).
    fn content<C>(self, content: C) -> Self
    where
        C: Fn() -> Option<AnyNewWidget> + 'static;
    /// Sets the [`pad`](Divider::set_pad) _reactively_.
    ///
    /// This `pad` determines the amount of space between the divider line and the content.
    /// It does nothing when there is no content.
    ///
    /// The default value is `5px`.
    fn pad<P>(self, pad: P) -> Self
    where
        P: Fn() -> Length + 'static;
    /// Use the [divider `content`](Divider::content_mut).
    ///
    /// It worth noting that the `use_content_fn` function will run inside an [`Effect`].
    fn use_content<C>(self, use_content_fn: C) -> Self
    where
        C: FnMut(Option<WidgetMut<'_, dyn Widget>>) + 'static;
}

impl NewDividerExt for NewWidget<Divider> {
    fn direction<A>(self, axis: A) -> Self
    where
        A: Fn() -> Axis + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Divider::set_direction(&mut this, axis());
        })
    }

    fn thickness<T>(self, thickness: T) -> Self
    where
        T: Fn() -> Option<Length> + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            if let Some(thickness) = thickness() {
                Divider::set_thickness(&mut this, thickness);
            } else {
                Divider::set_hairline(&mut this);
            }
        })
    }

    fn dash_fit<D>(self, dash_fit: D) -> Self
    where
        D: Fn() -> DashFit + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Divider::set_dash_fit(&mut this, dash_fit());
        })
    }

    fn dash_pattern<D>(self, dash_pattern: D) -> Self
    where
        D: Fn() -> Vec<Length> + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Divider::set_dash_pattern(&mut this, &dash_pattern());
        })
    }

    fn cap<C>(self, cap: C) -> Self
    where
        C: Fn() -> Cap + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Divider::set_cap(&mut this, cap());
        })
    }

    fn start_cap<C>(self, cap: C) -> Self
    where
        C: Fn() -> Cap + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Divider::set_start_cap(&mut this, cap());
        })
    }

    fn ending_cap<C>(self, cap: C) -> Self
    where
        C: Fn() -> Cap + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Divider::set_end_cap(&mut this, cap());
        })
    }

    fn placement<P>(self, placement: P) -> Self
    where
        P: Fn() -> Placement + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Divider::set_placement(&mut this, placement());
        })
    }

    fn content<C>(self, content: C) -> Self
    where
        C: Fn() -> Option<AnyNewWidget> + 'static,
    {
        let this_ref = self.create_velona_ref();
        Effect::new(move || {
            let content = content();
            this_ref
                .edit_local_now(move |mut this| {
                    if let Some(content) = content {
                        Divider::set_content(&mut this, content);
                    } else {
                        Divider::clear_content(&mut this);
                    }
                })
                .consume_with_log_err();
        });
        self
    }

    fn pad<P>(self, pad: P) -> Self
    where
        P: Fn() -> Length + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Divider::set_pad(&mut this, pad());
        })
    }

    fn use_content<C>(self, mut use_content_fn: C) -> Self
    where
        C: FnMut(Option<WidgetMut<'_, dyn Widget>>) + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            use_content_fn(Divider::content_mut(&mut this));
        })
    }
}

impl SingleChildWidget for NewWidget<Divider> {
    /// It worth noting that the `use_child_fn` might not re-run properly
    /// if there are no `content` inside the [`Divider`].
    ///
    /// It is recommended to use [`NewDividerExt::use_content`], instead of this.
    fn use_child_erased<C>(self, mut use_child_fn: C) -> Self
    where
        C: FnMut(WidgetMut<'_, dyn Widget>) + 'static,
    {
        self.use_content(move |content| {
            if let Some(content) = content {
                use_child_fn(content);
            } else {
                log::warn!("No content found of this `Divider`");
            }
        })
    }
}

impl ReactiveSingleChildExt for NewWidget<Divider> {
    /// A [`Option`]less version of [`NewDividerExt::content`].
    fn child<Cf>(self, child_fn: Cf) -> Self
    where
        Cf: Fn() -> AnyNewWidget + 'static,
    {
        self.content(move || Some(child_fn()))
    }
}
