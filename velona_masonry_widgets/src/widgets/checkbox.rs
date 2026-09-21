//! Various [`Checkbox`] trait implementation.
//!
//! The most important thing in the module is the [`NewCheckboxExt`]
//! which is implemented for [`NewWidget<Checkbox>`].
//!
//! _See the [widget](Checkbox) documentation for more information_.

use masonry::{
    core::{ArcStr, NewWidget},
    widgets::Checkbox,
};
use velona_core::reactive::{computed::Memo, traits::Get};

use crate::NewWidgetExt;

/// A [`Checkbox`] trait extension
pub trait NewCheckboxExt {
    /// Make the `checked` value reactive
    fn checked<C>(self, checked: C) -> Self
    where
        C: Fn() -> bool + 'static;
    /// Make the `checked` value reactive that warps `checked` with a [`Memo`].
    fn checked_memozied<C>(self, checked: C) -> Self
    where
        C: Fn() -> bool + Send + 'static + Sync,
        Self: std::marker::Sized,
    {
        let checked_memo = Memo::new(move |_| checked());
        self.checked(move || checked_memo.get())
    }
    /// Make the `text` value reactive
    fn text<Tf, T>(self, text: Tf) -> Self
    where
        Tf: Fn() -> T + 'static,
        T: Into<ArcStr>;
}

impl NewCheckboxExt for NewWidget<Checkbox> {
    fn checked<C>(self, checked: C) -> Self
    where
        C: Fn() -> bool + 'static,
    {
        self.use_reactive_widget_mut(move |mut widget_mut| {
            Checkbox::set_checked(&mut widget_mut, checked());
        })
    }

    fn text<Tf, T>(self, text: Tf) -> Self
    where
        Tf: Fn() -> T + 'static,
        T: Into<ArcStr>,
    {
        self.use_reactive_widget_mut(move |mut widget_mut| {
            Checkbox::set_text(&mut widget_mut, text().into());
        })
    }
}
