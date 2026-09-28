//! Various [`Label`] implementations.
//!
//! The most important thing in the module is the [`NewLabelExt`]
//! which is implemented for [`NewWidget<Label>`].
//!
//! [`NewChildedLabelExt`] is just a [`NewLabelExt`] for [`W: TypedSingleChildWidget<Child = Label> + 'static`](TypedSingleChildWidget) (for ease of use)
//!
//! _See the [widget](Label) documentation for more information_.

use std::mem::Discriminant;

use crate::{
    utils::text_style::{apply_label_style_actions, get_style_opt_action},
    widgets::TypedSingleChildWidget,
};
use masonry::{
    TextAlign,
    core::{ArcStr, NewWidget, StyleProperty, Widget},
    widgets::Label,
};
use velona_core::{
    reactive::{graph::untrack, traits::SignalOrFn},
    widgets::UseWidgetValResult,
};
// use velona_core::widgets::TypedSingleChildWidget;

use super::NewWidgetExt;

/// A [`Label`] trait extention
#[must_use]
pub trait NewLabelExt {
    /// It is inefficient to call this function twice.
    fn text<S, T>(self, text: S) -> Self
    where
        S: SignalOrFn<Output = T> + 'static,
        T: Into<ArcStr>;
    /// Reactive text styles.
    fn style<S, T>(self, style: S) -> Self
    where
        S: SignalOrFn<Output = T> + 'static,
        T: Into<StyleProperty>;
    /// Reactive optional text styles.
    fn style_opt<S, T>(self, style: S) -> Self
    where
        S: SignalOrFn<Output = Option<T>> + 'static,
        T: Into<StyleProperty>;
    /// The reactive equivalent of [`with_hint`](Label::with_hint).
    fn hint<S>(self, hint: S) -> Self
    where
        S: SignalOrFn<Output = bool> + 'static;
    /// The reactive equivalent of [`with_text_alignment`](Label::with_text_alignment).
    fn text_alignment<S>(self, align: S) -> Self
    where
        S: SignalOrFn<Output = TextAlign> + 'static;
}

impl NewLabelExt for NewWidget<Label> {
    fn text<S, T>(self, text: S) -> Self
    where
        S: SignalOrFn<Output = T> + 'static,
        T: Into<ArcStr>,
    {
        self.use_widget_mut(
            move || text.run().into(),
            move |mut this, text| {
                Label::set_text(&mut this, text);
            },
        )
    }

    fn style_opt<S, T>(self, style: S) -> Self
    where
        S: SignalOrFn<Output = Option<T>> + 'static,
        T: Into<StyleProperty>,
    {
        self.use_widget_mut_val(
            move |old_style: Option<Discriminant<StyleProperty>>| {
                let new_style = style.run().map(Into::<StyleProperty>::into);
                get_style_opt_action(old_style, new_style)
            },
            apply_label_style_actions,
        )
    }
    fn style<S, T>(self, style: S) -> Self
    where
        S: SignalOrFn<Output = T> + 'static,
        T: Into<StyleProperty>,
    {
        self.style_opt(move || Some(style.run()))
    }

    fn hint<S>(self, hint: S) -> Self
    where
        S: SignalOrFn<Output = bool> + 'static,
    {
        self.use_widget_mut(hint, |mut this, hint| {
            Label::set_hint(&mut this, hint);
        })
    }

    fn text_alignment<S>(self, align: S) -> Self
    where
        S: SignalOrFn<Output = TextAlign> + 'static,
    {
        // {
        //     self.widget = Box::new(self.widget.with_text_alignment(untrack(&align)));
        // }
        self.use_widget_mut(align, |mut this, align| {
            Label::set_text_alignment(&mut this, align);
        })
    }
}

/// [`NewLabelExt`] knock-off
#[must_use]
pub trait NewChildedLabelExt {
    /// It is inefficient to call this function twice.
    fn text<S, T>(self, text: S) -> Self
    where
        S: SignalOrFn<Output = T> + 'static,
        T: Into<ArcStr>;
    /// Reactive text styles.
    fn style<S, T>(self, style: S) -> Self
    where
        S: SignalOrFn<Output = T> + 'static,
        T: Into<StyleProperty>;
    /// Reactive optional text styles.
    fn style_opt<S, T>(self, style: S) -> Self
    where
        S: SignalOrFn<Output = Option<T>> + 'static,
        T: Into<StyleProperty>;
    /// The reactive equivalent of [`with_hint`](Label::with_hint).
    fn hint<S>(self, hint: S) -> Self
    where
        S: SignalOrFn<Output = bool> + 'static;
    /// The reactive equivalent of [`with_text_alignment`](Label::with_text_alignment).
    fn text_alignment<S>(self, align: S) -> Self
    where
        S: SignalOrFn<Output = TextAlign> + 'static;
}

impl<W> NewChildedLabelExt for W
where
    W: TypedSingleChildWidget<Child = Label> + 'static,
{
    fn text<S, T>(self, text: S) -> Self
    where
        S: SignalOrFn<Output = T> + 'static,
        T: Into<ArcStr>,
    {
        self.use_child(
            move |_| UseWidgetValResult::to_edit_fn(text.run().into()),
            |mut this, text| {
                Label::set_text(&mut this, text);
            },
        )
    }

    fn style_opt<S, T>(self, style: S) -> Self
    where
        S: SignalOrFn<Output = Option<T>> + 'static,
        T: Into<StyleProperty>,
    {
        self.use_child(
            move |old_style: Option<Discriminant<StyleProperty>>| {
                let new_style = style.run().map(Into::<StyleProperty>::into);
                get_style_opt_action(old_style, new_style)
            },
            apply_label_style_actions,
        )
    }
    fn style<S, T>(self, style: S) -> Self
    where
        S: SignalOrFn<Output = T> + 'static,
        T: Into<StyleProperty>,
    {
        self.style_opt(move || Some(style.run()))
    }

    fn hint<S>(self, hint: S) -> Self
    where
        S: SignalOrFn<Output = bool> + 'static,
    {
        self.use_child(
            move |_| UseWidgetValResult::to_edit_fn(hint.run()),
            |mut this, hint| {
                Label::set_hint(&mut this, hint);
            },
        )
    }

    fn text_alignment<S>(self, align: S) -> Self
    where
        S: SignalOrFn<Output = TextAlign> + 'static,
    {
        // {
        //     self.widget = Box::new(self.widget.with_text_alignment(untrack(&align)));
        // }
        self.use_child(
            move |_| UseWidgetValResult::to_edit_fn(align.run()),
            |mut this, align| {
                Label::set_text_alignment(&mut this, align);
            },
        )
    }
}

#[must_use]
pub trait IntoLabel {
    fn into_label(self) -> Label;
}

impl<V> IntoLabel for V
where
    V: Into<ArcStr>,
{
    fn into_label(self) -> Label {
        Label::new(self)
    }
}

#[must_use]
pub trait IntoNewLabel {
    fn into_new_label(self) -> NewWidget<Label>;
}

impl<V, T> IntoNewLabel for V
where
    V: SignalOrFn<Output = T> + 'static,
    T: Into<ArcStr> + 'static,
{
    fn into_new_label(self) -> NewWidget<Label> {
        Label::new(untrack(|| self.run()))
            .prepare()
            .text(move || self.run().into())
    }
}
