use masonry::{
    core::{ArcStr, NewWidget, Widget},
    widgets::Checkbox,
};
use reactive_graph::{computed::Memo, graph::untrack, traits::Get};

use crate::NewWidgetExt;

/// A [`Checkbox`] trait extension
pub trait NewCheckboxExt {
    /// create a new checkbox with reactive text and checked value
    fn new<Cf, Tf, T>(checked: Cf, text: Tf) -> Self
    where
        Cf: Fn() -> bool + 'static,
        Tf: Fn() -> T + 'static,
        T: Into<ArcStr>;
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
    fn new<Cf, Tf, T>(checked: Cf, text: Tf) -> Self
    where
        Cf: Fn() -> bool + 'static,
        Tf: Fn() -> T + 'static,
        T: Into<ArcStr>,
    {
        Checkbox::new(untrack(&checked), untrack(&text))
            .with_auto_id()
            .checked(checked)
            .text(text)
    }
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

pub fn _checkbox<Cf, Tf, T>(checked: Cf, text: Tf) -> NewWidget<Checkbox>
where
    Cf: Fn() -> bool + 'static,
    Tf: Fn() -> T + 'static,
    T: Into<ArcStr>,
{
    <NewWidget<Checkbox> as NewCheckboxExt>::new(checked, text)
}
