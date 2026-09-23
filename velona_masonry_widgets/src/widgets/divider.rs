//! Various [`Divider`] implementations.
//!
//! The most important thing in the module is the [`NewDividerExt`]
//! which is implemented for [`NewWidget<Divider>`].
//!
//! _See the [widget](Divider) documentation for more information_.
use masonry::imaging::kurbo::{Axis, Cap};
use masonry::{
    core::{NewWidget, Widget, WidgetMut},
    layout::Length,
    widgets::{DashFit, Divider, Placement},
};
use velona_core::AnyNewWidget;
#[cfg(doc)]
use velona_core::reactive::effect::Effect;
use velona_core::widgets::UseWidgetValResult;

use crate::NewWidgetExt;

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
        D: Fn() -> Box<[Length]> + 'static;
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
    /// It worth noting that the only `val_fn` function will run inside an [`Effect`].
    fn use_content<Vfn, Efn, V, O>(self, val_fn: Vfn, edit_fn: Efn) -> Self
    where
        Efn: FnMut(Option<WidgetMut<'_, dyn Widget>>, V) + 'static,
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static;
}

impl NewDividerExt for NewWidget<Divider> {
    fn direction<A>(self, axis: A) -> Self
    where
        A: Fn() -> Axis + 'static,
    {
        self.use_widget_mut(axis, |mut this, axis| {
            Divider::set_direction(&mut this, axis);
        })
    }

    fn thickness<T>(self, thickness: T) -> Self
    where
        T: Fn() -> Option<Length> + 'static,
    {
        self.use_widget_mut(thickness, |mut this, maybe_thickness| {
            if let Some(thickness) = maybe_thickness {
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
        self.use_widget_mut(dash_fit, |mut this, dash_fit| {
            Divider::set_dash_fit(&mut this, dash_fit);
        })
    }

    fn dash_pattern<D>(self, dash_pattern: D) -> Self
    where
        D: Fn() -> Box<[Length]> + 'static,
    {
        self.use_widget_mut(dash_pattern, |mut this, dash_pattern| {
            Divider::set_dash_pattern(&mut this, &dash_pattern);
        })
    }

    fn cap<C>(self, cap: C) -> Self
    where
        C: Fn() -> Cap + 'static,
    {
        self.use_widget_mut(cap, |mut this, cap| {
            Divider::set_cap(&mut this, cap);
        })
    }

    fn start_cap<C>(self, cap: C) -> Self
    where
        C: Fn() -> Cap + 'static,
    {
        self.use_widget_mut(cap, |mut this, cap| {
            Divider::set_start_cap(&mut this, cap);
        })
    }

    fn ending_cap<C>(self, cap: C) -> Self
    where
        C: Fn() -> Cap + 'static,
    {
        self.use_widget_mut(cap, |mut this, cap| {
            Divider::set_end_cap(&mut this, cap);
        })
    }

    fn placement<P>(self, placement: P) -> Self
    where
        P: Fn() -> Placement + 'static,
    {
        self.use_widget_mut(placement, |mut this, placement| {
            Divider::set_placement(&mut this, placement);
        })
    }

    fn content<C>(self, content: C) -> Self
    where
        C: Fn() -> Option<AnyNewWidget> + 'static,
    {
        self.use_widget_mut(content, |mut this, maybe_content| {
            if let Some(content) = maybe_content {
                Divider::set_content(&mut this, content);
            } else {
                Divider::clear_content(&mut this);
            }
        })
    }

    fn pad<P>(self, pad: P) -> Self
    where
        P: Fn() -> Length + 'static,
    {
        self.use_widget_mut(pad, |mut this, pad| {
            Divider::set_pad(&mut this, pad);
        })
    }

    fn use_content<Vfn, Efn, V, O>(self, val_fn: Vfn, mut edit_fn: Efn) -> Self
    where
        Efn: FnMut(Option<WidgetMut<'_, dyn Widget>>, V) + 'static,
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static,
    {
        self.use_widget_mut_val(val_fn, move |mut this, val| {
            edit_fn(Divider::content_mut(&mut this), val);
        })
    }
}

// impl SingleChildWidget for NewWidget<Divider> {
//     /// It worth noting that the `use_child_fn` might not re-run properly
//     /// if there are no `content` inside the [`Divider`].
//     ///
//     /// It is recommended to use [`NewDividerExt::use_content`], instead of this.
//     fn use_child_erased<C>(self, mut use_child_fn: C) -> Self
//     where
//         C: FnMut(WidgetMut<'_, dyn Widget>) + 'static,
//     {
//         self.use_content(move |content| {
//             if let Some(content) = content {
//                 use_child_fn(content);
//             } else {
//                 log::warn!("No content found of this `Divider`");
//             }
//         })
//     }
// }

// impl ReactiveSingleChildExt for NewWidget<Divider> {
//     /// A [`Option`]less version of [`NewDividerExt::content`].
//     fn child<Cf>(self, child_fn: Cf) -> Self
//     where
//         Cf: Fn() -> AnyNewWidget + 'static,
//     {
//         self.content(move || Some(child_fn()))
//     }
// }
