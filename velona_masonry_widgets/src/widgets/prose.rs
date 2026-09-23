//! Various [`Prose`] implementations.
//!
//! The most important thing in the module is the [`NewProseExt`]
//! which is implemented for [`NewWidget<Prose>`].
//!
//! [`NewWidget<Prose>`] also implement the [`NewTextAreaExt`] trait.
//!
//! _See the [widget](Prose) documentation for more information_.

use std::{fmt::Display, mem::Discriminant};

use masonry::{
    TextAlign,
    core::{NewWidget, StyleProperty, Widget, WidgetMut},
    widgets::{InsertNewline, Prose, TextArea},
};

#[cfg(doc)]
use velona_core::reactive::effect::Effect;
use velona_core::{
    reactive::{graph::untrack, traits::SignalOrFn},
    widgets::UseWidgetValResult,
};

use crate::{
    NewWidgetExt,
    utils::text_style::{apply_text_style_actions, get_style_opt_action},
    widgets::text_area::NewTextAreaExt,
};

/// A [new](NewWidget) [`Prose`] trait extension.
#[must_use]
pub trait NewProseExt {
    /// Whether to clip the text to the available space.
    ///
    /// Reactive variant of [`Prose::set_clip`]
    fn clip<C>(self, clip: C) -> Self
    where
        C: Fn() -> bool + 'static;
    /// Use the underlying text area.
    ///
    /// It is worth noting that only the `use_fn` will run inside an [`Effect`].
    ///
    /// Used to modify most properties of the text.
    fn use_text_mut<Vfn, Efn, V, O>(self, val_fn: Vfn, edit_fn: Efn) -> Self
    where
        Efn: FnMut(WidgetMut<'_, TextArea<false>>, V) + 'static,
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static;
}

impl NewProseExt for NewWidget<Prose> {
    fn clip<C>(self, clip: C) -> Self
    where
        C: Fn() -> bool + 'static,
    {
        self.use_widget_mut(clip, move |mut this, clip| {
            Prose::set_clip(&mut this, clip);
        })
    }

    fn use_text_mut<Vfn, Efn, V, O>(self, val_fn: Vfn, mut edit_fn: Efn) -> Self
    where
        Efn: FnMut(WidgetMut<'_, TextArea<false>>, V) + 'static,
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static,
    {
        self.use_widget_mut_val(val_fn, move |mut this, val| {
            edit_fn(Prose::text_mut(&mut this), val);
        })
    }
}

impl NewTextAreaExt<false> for NewWidget<Prose> {
    fn style<S, T>(self, style: S) -> Self
    where
        S: Fn() -> T + 'static,
        T: Into<StyleProperty>,
    {
        self.style_opt(move || Some(style()))
    }

    fn style_opt<S, T>(self, style: S) -> Self
    where
        S: Fn() -> Option<T> + 'static,
        T: Into<StyleProperty>,
    {
        self.use_text_mut(
            move |old_style: Option<Discriminant<StyleProperty>>| {
                let new_style = style().map(Into::<StyleProperty>::into);
                get_style_opt_action(old_style, new_style)
            },
            apply_text_style_actions,
        )
    }

    fn hint<S>(self, hint: S) -> Self
    where
        S: Fn() -> bool + 'static,
    {
        self.use_text_mut(
            move |_| UseWidgetValResult::to_edit_fn(hint()),
            |mut this, hint| {
                TextArea::set_hint(&mut this, hint);
            },
        )
    }

    fn text_alignment<S>(self, align: S) -> Self
    where
        S: Fn() -> TextAlign + 'static,
    {
        self.use_text_mut(
            move |_| UseWidgetValResult::to_edit_fn(align()),
            |mut this, align| {
                TextArea::set_text_alignment(&mut this, align);
            },
        )
    }

    fn word_wrap<W>(self, wrap_words: W) -> Self
    where
        W: Fn() -> bool + 'static,
    {
        self.use_text_mut(
            move |_| UseWidgetValResult::to_edit_fn(wrap_words()),
            |mut this, wrap_words| {
                TextArea::set_word_wrap(&mut this, wrap_words);
            },
        )
    }

    fn insert_newline<I>(self, insert_newline: I) -> Self
    where
        I: Fn() -> InsertNewline + 'static,
    {
        self.use_text_mut(
            move |_| UseWidgetValResult::to_edit_fn(insert_newline()),
            |mut this, insert_newline| {
                TextArea::set_insert_newline(&mut this, insert_newline);
            },
        )
    }

    fn text<Tfn, T>(self, text: Tfn) -> Self
    where
        Tfn: Fn() -> T + 'static,
        T: AsRef<str> + 'static,
    {
        self.use_text_mut(
            move |_| UseWidgetValResult::to_edit_fn(text()),
            |mut this, text| {
                TextArea::reset_text(&mut this, text.as_ref());
            },
        )
    }
}

#[must_use]
pub trait IntoProse {
    fn into_prose(self) -> Prose;
}

impl<V> IntoProse for V
where
    V: Display,
{
    fn into_prose(self) -> Prose {
        Prose::new(&self.to_string())
    }
}

#[must_use]
pub trait IntoNewProse {
    fn into_new_prose(self) -> NewWidget<Prose>;
}

impl<Vfn, V> IntoNewProse for Vfn
where
    Vfn: SignalOrFn<Output = V> + 'static,
    V: AsRef<str> + 'static,
{
    fn into_new_prose(self) -> NewWidget<Prose> {
        Prose::new(untrack(|| self.run()).as_ref())
            .prepare()
            .text(move || self.run())
    }
}
