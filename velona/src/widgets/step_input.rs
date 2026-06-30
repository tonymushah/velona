use masonry::{
    core::{NewWidget, WidgetMut},
    widgets::{StepInput, StepState, Steppable},
};

#[cfg(doc)]
use masonry::widgets::Step;
#[cfg(doc)]
use reactive_graph::owner::Owner;

use crate::NewWidgetExt;

/// An utility struct for [`StepInput`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct StepInputBounds<T> {
    pub min: T,
    pub max: T,
}

impl<T> StepInputBounds<T>
where
    T: Steppable + 'static,
{
    /// Apply this bounds to a [`StepInput`].
    pub fn apply(self, this: &mut WidgetMut<StepInput<T>>) {
        StepInput::set_bounds(this, self.min, self.max);
    }
}

/// A [new](NewWidget) [`StepInput`] trait extension.
// TODO add example
pub trait NewStepInputExt<T>
where
    T: Steppable + 'static,
{
    /// Set a new `base` value
    /// reactively.
    ///
    /// The new `base` value must be greater or equal
    /// to `min` and less than or equal to `max`.
    /// Otherwise it will be ignored.
    ///
    /// **Never update the value in response to a [`Step`] action as it may cause precision issues.**
    /// Only call this when the base value has changed due to reasons other than what [`StepInput`] already knows about.
    ///
    /// # Panics
    ///
    /// Panics if `base` is less than `min` or greater than `max` and debug assertions are enabled.
    fn base<B>(self, base: B) -> Self
    where
        B: Fn() -> T + 'static;
    /// Set a new `step` value
    /// reactively.
    ///
    /// The new `step` value must be greater than zero.
    /// Invalid values will be ignored.
    ///
    /// # Panics
    ///
    /// Panics if `step` is zero or less and debug assertions are enabled.
    fn step<S>(self, step: S) -> Self
    where
        S: Fn() -> T + 'static;
    /// Set a new `snap` value
    /// reactively.
    ///
    /// The new `snap` value must be greater than zero.
    /// Invalid values will be ignored.
    /// `None` disables the snap functionality.
    ///
    /// # Panics
    ///
    /// Panics if `snap` is zero or less and debug assertions are enabled.
    fn snap<S>(self, snap: S) -> Self
    where
        S: Fn() -> Option<T> + 'static;
    /// Set new `min` and `max` bounds
    /// reactively.
    ///
    /// [`min`](StepInputBounds::min) must be less or equal to [`max`](StepInputBounds::max).
    /// Invalid values will be ignored.
    ///
    /// # Panics
    ///
    /// Panics if [`min`](StepInputBounds::min) is greater than [`max`](StepInputBounds::max) and debug assertions are enabled.
    fn bounds<B>(self, bounds: B) -> Self
    where
        B: Fn() -> StepInputBounds<T> + 'static;
    /// Set a new wrap value
    /// reactively.
    ///
    /// Wrap determines whether the values should form an infinite circle,
    /// where increasing above `max` results in `min` and decreasing below `min` results in `max`.
    fn wrap<W>(self, wrap: W) -> Self
    where
        W: Fn() -> bool + 'static;
    /// Set a new custom display function
    /// reactively.
    ///
    /// _I know that you will hate why this function is like this_.
    ///
    /// The display function will receive a [`StepState`] and it will need to return its [`String`] representation.
    ///
    /// **DON'T PUT ANY REACTIVE VALUE INSIDE THE DISPLAY FUNCTION as it will be called outside of the [`Owner`] it is currently in.**
    ///
    /// This is useful for showing units (1 ft, 2 GB) or for doing visual-only rounding, etc.
    fn display<D, F>(self, display_fn_factory: D) -> Self
    where
        D: Fn() -> F + 'static,
        F: Fn(StepState<T>) -> String + 'static;
}

impl<T> NewStepInputExt<T> for NewWidget<StepInput<T>>
where
    T: Steppable + 'static,
{
    fn base<B>(self, base: B) -> Self
    where
        B: Fn() -> T + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            StepInput::set_base(&mut this, base());
        })
    }

    fn step<S>(self, step: S) -> Self
    where
        S: Fn() -> T + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            StepInput::set_step(&mut this, step());
        })
    }

    fn snap<S>(self, snap: S) -> Self
    where
        S: Fn() -> Option<T> + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            StepInput::set_snap(&mut this, snap());
        })
    }

    fn bounds<B>(self, bounds: B) -> Self
    where
        B: Fn() -> StepInputBounds<T> + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            bounds().apply(&mut this);
        })
    }

    fn wrap<W>(self, wrap: W) -> Self
    where
        W: Fn() -> bool + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            StepInput::set_wrap(&mut this, wrap());
        })
    }

    fn display<D, F>(self, display: D) -> Self
    where
        D: Fn() -> F + 'static,
        F: Fn(StepState<T>) -> String + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            StepInput::set_display(&mut this, display());
        })
    }
}
