use std::range::Range;

use masonry::{
    core::{NewWidget, WidgetMut},
    widgets::Slider,
};

use crate::NewWidgetExt;

/// A utility struct for [`Slider::set_range`].
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct SliderRange {
    pub min: f64,
    pub max: f64,
}

impl SliderRange {
    pub fn apply(self, this: &mut WidgetMut<Slider>) {
        Slider::set_range(this, self.min, self.max);
    }
}

impl From<Range<f64>> for SliderRange {
    fn from(value: Range<f64>) -> Self {
        Self {
            min: value.start,
            max: value.end,
        }
    }
}

impl From<SliderRange> for Range<f64> {
    fn from(value: SliderRange) -> Self {
        Self {
            start: value.min,
            end: value.max,
        }
    }
}

/// A [new](NewWidget) [`Slider`] trait extension.
// TODO add example
pub trait NewSliderExt {
    /// Set the [slider value](Slider::set_value) reactively.
    fn value<V>(self, value: V) -> Self
    where
        V: Fn() -> f64 + 'static;
    /// Sets or removes the stepping interval of the slider.
    ///
    /// _A reactive version of [`Slider::set_step`]_.
    fn step<S>(self, step: S) -> Self
    where
        S: Fn() -> Option<f64> + 'static;
    /// Sets the [range (min and max) of the slider](Slider::set_range) reactively.
    fn range<R>(self, range: R) -> Self
    where
        R: Fn() -> SliderRange + 'static;
}

impl NewSliderExt for NewWidget<Slider> {
    fn value<V>(self, value: V) -> Self
    where
        V: Fn() -> f64 + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Slider::set_value(&mut this, value());
        })
    }

    fn step<S>(self, step: S) -> Self
    where
        S: Fn() -> Option<f64> + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Slider::set_step(&mut this, step());
        })
    }

    fn range<R>(self, range: R) -> Self
    where
        R: Fn() -> SliderRange + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            range().apply(&mut this);
        })
    }
}
