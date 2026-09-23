//! Various [`Align`] trait implementations.
//!
//! The most important thing in the module is the [`NewAlignExt`]
//! which is implemented for [`NewWidget<Align>`].
//!
//! The [`NewWidget<Align>`] also implements the [`ReactiveSingleChildExt`][reactive-child] trait
//! and the [`SingleChildWidget`][single-widget] trait.
//!
//! _See the [widget](Align) documentation for more information_.
//!
//! There is also the [`IntoAlign`] and [`IntoNewAlign`] trait for quickly building a [`Align`] widget from any [`View`].
//!
//! [single-widget]: super::SingleChildWidget
//! [reactive-child]: super::ReactiveSingleChildExt

use masonry::{
    core::{NewWidget, Widget},
    layout::UnitPoint,
    widgets::Align,
};
use masonry_raw_box::RawBox;
use velona_core::widgets::View;

use crate::NewWidgetExt;

/// A [new](NewWidget) [`Align`] trait extension
#[must_use]
pub trait NewAlignExt {
    /// Make the [`Align::set_alignment`] reactive
    fn alignment<A>(self, alignment: A) -> Self
    where
        A: Fn() -> UnitPoint + 'static;
}

impl NewAlignExt for NewWidget<Align> {
    fn alignment<A>(self, alignment: A) -> Self
    where
        A: Fn() -> UnitPoint + 'static,
    {
        self.use_widget_mut(alignment, |mut this, alignment| {
            Align::set_alignment(&mut this, alignment);
        })
    }
}

#[must_use]
pub trait IntoAlign {
    fn align_centered(self) -> Align;
    fn align_right(self) -> Align;
    fn align_left(self) -> Align;
    fn align_horizontal(self, align: UnitPoint) -> Align;
    fn align_vertical(self, align: UnitPoint) -> Align;
}

impl<V> IntoAlign for V
where
    V: View + 'static,
{
    fn align_centered(self) -> Align {
        Align::centered(self.into_new_widget())
    }

    fn align_right(self) -> Align {
        Align::right(self.into_new_widget())
    }

    fn align_left(self) -> Align {
        Align::left(self.into_new_widget())
    }

    fn align_horizontal(self, align: UnitPoint) -> Align {
        Align::horizontal(align, self.into_new_widget())
    }

    fn align_vertical(self, align: UnitPoint) -> Align {
        Align::vertical(align, self.into_new_widget())
    }
}

#[must_use]
pub trait IntoNewAlign {
    fn new_align_centered(self) -> NewWidget<Align>;
    fn new_align_right(self) -> NewWidget<Align>;
    fn new_align_left(self) -> NewWidget<Align>;
    fn new_align_horizontal(self, align: UnitPoint) -> NewWidget<Align>;
    fn new_align_vertical(self, align: UnitPoint) -> NewWidget<Align>;
}

impl<V, Vfn> IntoNewAlign for Vfn
where
    V: View + 'static,
    Vfn: Fn() -> V + 'static,
{
    fn new_align_centered(self) -> NewWidget<Align> {
        Align::centered(RawBox::empty().prepare())
            .prepare()
            .use_widget_mut(
                move || (self)().into_erased(),
                |mut this, child| {
                    Align::set_child(&mut this, child);
                },
            )
    }

    fn new_align_right(self) -> NewWidget<Align> {
        Align::right(RawBox::empty().prepare())
            .prepare()
            .use_widget_mut(
                move || (self)().into_erased(),
                |mut this, child| {
                    Align::set_child(&mut this, child);
                },
            )
    }

    fn new_align_left(self) -> NewWidget<Align> {
        Align::left(RawBox::empty().prepare())
            .prepare()
            .use_widget_mut(
                move || (self)().into_erased(),
                |mut this, child| {
                    Align::set_child(&mut this, child);
                },
            )
    }

    fn new_align_horizontal(self, align: UnitPoint) -> NewWidget<Align> {
        Align::horizontal(align, RawBox::empty().prepare())
            .prepare()
            .use_widget_mut(
                move || (self)().into_erased(),
                |mut this, child| {
                    Align::set_child(&mut this, child);
                },
            )
    }

    fn new_align_vertical(self, align: UnitPoint) -> NewWidget<Align> {
        Align::vertical(align, RawBox::empty().prepare())
            .prepare()
            .use_widget_mut(
                move || (self)().into_erased(),
                |mut this, child| {
                    Align::set_child(&mut this, child);
                },
            )
    }
}
