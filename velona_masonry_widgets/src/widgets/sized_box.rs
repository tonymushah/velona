//! Various [`SizedBox`] implementations.
//!
//! The most important thing in the module is the [`NewSizedBoxExt`]
//! which is implemented for [`NewWidget<SizedBox>`].
//!
//! _See the [widget](SizedBox) documentation for more information_.

use masonry::{
    core::{NewWidget, Widget, WidgetMut},
    layout::Length,
    widgets::SizedBox,
};
use velona_core::{
    AnyNewWidget,
    widgets::{UseWidgetValResult, View},
};

#[cfg(doc)]
use velona_core::reactive::effect::Effect;

use crate::NewWidgetExt;

/// A [new](NewWidget) [`SizedBox`] extension trait.
pub trait NewSizedBoxExt {
    /// Set a "reactive" child for this [`SizedBox`].
    ///
    /// The `child_fn` will run inside a [`Effect::new`].
    ///
    /// If the function returns a [`NewWidget`], it will [update](SizedBox::set_child) the current child,
    /// if [`None`], the current child will be [removed](SizedBox::remove_child).
    ///
    /// If you want an non-[`Option`] version, use [`ReactiveSingleChildExt::child`].
    ///
    /// [`ReactiveSingleChildExt::child`]: crate::widgets::ReactiveSingleChildExt::child
    fn child_opt<Cf>(self, child_fn: Cf) -> Self
    where
        Cf: Fn() -> Option<AnyNewWidget> + 'static;
    /// Set a reactive width for this [`SizedBox`].
    ///
    /// The `width_fn` will run inside a [`Effect::new`],
    ///
    /// If the function returns a [`Length`], it will [update the current width](SizedBox::set_width) sized box,
    /// if [`None`], the current container width will be [unset](SizedBox::unset_width).
    fn raw_width<W>(self, width_fn: W) -> Self
    where
        W: Fn() -> Option<Length> + 'static;
    /// Similar to [`width_opt`](Self::raw_width)
    fn width<W>(self, width_fn: W) -> Self
    where
        W: Fn() -> Length + 'static,
        Self: Sized,
    {
        self.raw_width(move || Some(width_fn()))
    }
    /// Set a reactive height for this [`SizedBox`].
    ///
    /// The `height_fn` will run inside a [`Effect::new`],
    ///
    /// If the function returns a [`Length`], it will [update the current height](SizedBox::set_height) sized box,
    /// if [`None`], the current container height will be [unset](SizedBox::unset_height).
    fn raw_height<W>(self, height_fn: W) -> Self
    where
        W: Fn() -> Option<Length> + 'static;
    /// Similar to [`height_opt`](Self::raw_height)
    fn height<W>(self, height_fn: W) -> Self
    where
        W: Fn() -> Length + 'static,
        Self: Sized,
    {
        self.raw_height(move || Some(height_fn()))
    }
    fn use_child_opt<Vfn, Efn, V, O>(self, val_fn: Vfn, edit_fn: Efn) -> Self
    where
        Efn: FnMut(Option<WidgetMut<'_, dyn Widget>>, V) + 'static,
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static;
}

impl NewSizedBoxExt for NewWidget<SizedBox> {
    fn child_opt<Cf>(self, child_fn: Cf) -> Self
    where
        Cf: Fn() -> Option<AnyNewWidget> + 'static,
    {
        self.use_widget_mut(child_fn, |mut this, child| {
            if let Some(child) = child {
                SizedBox::set_child(&mut this, child);
            } else {
                SizedBox::remove_child(&mut this);
            }
        })
    }

    fn raw_width<W>(self, width_fn: W) -> Self
    where
        W: Fn() -> Option<Length> + 'static,
    {
        self.use_widget_mut(width_fn, |mut this, width| {
            SizedBox::set_raw_width(&mut this, width);
        })
    }

    fn raw_height<W>(self, height_fn: W) -> Self
    where
        W: Fn() -> Option<Length> + 'static,
    {
        self.use_widget_mut(height_fn, |mut this, height| {
            SizedBox::set_raw_height(&mut this, height);
        })
    }

    fn use_child_opt<Vfn, Efn, V, O>(self, val_fn: Vfn, mut edit_fn: Efn) -> Self
    where
        Efn: FnMut(Option<WidgetMut<'_, dyn Widget>>, V) + 'static,
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static,
    {
        self.use_widget_mut_val(val_fn, move |mut this, val| {
            edit_fn(SizedBox::child_mut(&mut this), val)
        })
    }
}

pub trait IntoSizedBox {
    fn into_sized_box(self) -> SizedBox;
}

impl<V> IntoSizedBox for V
where
    V: View + 'static,
{
    fn into_sized_box(self) -> SizedBox {
        SizedBox::new(self.into_new_widget())
    }
}

pub trait IntoNewSizedBox {
    fn into_new_sized_box(self) -> NewWidget<SizedBox>;
}

impl<Vfn, V> IntoNewSizedBox for Vfn
where
    Vfn: Fn() -> Option<V> + 'static,
    V: View + 'static,
{
    fn into_new_sized_box(self) -> NewWidget<SizedBox> {
        SizedBox::empty()
            .prepare()
            .child_opt(move || (self)().map(V::into_erased))
    }
}
