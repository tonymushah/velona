//! Various [`Prose`] implementations.
//!
//! The most important thing in the module is the [`NewProseExt`]
//! which is implemented for [`NewWidget<Prose>`].
//!
//! [`NewWidget<Prose>`] also implement the [`NewTextAreaExt`] trait.
//!
//! _See the [widget](Prose) documentation for more information_.

use std::mem::{Discriminant, discriminant};

use masonry::{
    TextAlign,
    core::{NewWidget, StyleProperty, WidgetMut},
    widgets::{InsertNewline, Prose, TextArea},
};

#[cfg(doc)]
use reactive_graph::effect::Effect;

use crate::{NewWidgetExt, widgets::text_area::NewTextAreaExt};

/// A [new](NewWidget) [`Prose`] trait extension.
pub trait NewProseExt {
    /// Whether to clip the text to the available space.
    ///
    /// Reactive variant of [`Prose::set_clip`]
    fn clip<C>(self, clip: C) -> Self
    where
        C: Fn() -> bool + 'static;
    /// Use the underlying text area.
    ///
    /// It is worth noting that the `use_fn` will run inside an [`Effect`].
    ///
    /// Used to modify most properties of the text.
    fn use_text_mut<T>(self, use_fn: T) -> Self
    where
        T: FnMut(WidgetMut<TextArea<false>>) + 'static;
}

impl NewProseExt for NewWidget<Prose> {
    fn clip<C>(self, clip: C) -> Self
    where
        C: Fn() -> bool + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Prose::set_clip(&mut this, clip());
        })
    }

    fn use_text_mut<T>(self, mut use_fn: T) -> Self
    where
        T: FnMut(WidgetMut<TextArea<false>>) + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            use_fn(Prose::text_mut(&mut this));
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
        self.use_reactive_widget_mut_with_effect_val::<_, Discriminant<StyleProperty>>(
            move |mut this, old_style| {
                let mut this = Prose::text_mut(&mut this);
                if let Some(old_style) = old_style {
                    TextArea::remove_style(&mut this, old_style);
                }
                if let Some(style) = style() {
                    TextArea::insert_style(&mut this, style)
                        .as_ref()
                        .map(discriminant)
                } else {
                    None
                }
            },
        )
    }

    fn hint<S>(self, hint: S) -> Self
    where
        S: Fn() -> bool + 'static,
    {
        self.use_text_mut(move |mut this| {
            TextArea::set_hint(&mut this, hint());
        })
    }

    fn text_alignment<S>(self, align: S) -> Self
    where
        S: Fn() -> TextAlign + 'static,
    {
        self.use_text_mut(move |mut this| {
            TextArea::set_text_alignment(&mut this, align());
        })
    }

    fn word_wrap<W>(self, wrap_words: W) -> Self
    where
        W: Fn() -> bool + 'static,
    {
        self.use_text_mut(move |mut this| {
            TextArea::set_word_wrap(&mut this, wrap_words());
        })
    }

    fn insert_newline<I>(self, insert_newline: I) -> Self
    where
        I: Fn() -> InsertNewline + 'static,
    {
        self.use_text_mut(move |mut this| {
            TextArea::set_insert_newline(&mut this, insert_newline());
        })
    }

    fn text<T>(self, text: T) -> Self
    where
        T: Fn() -> String + 'static,
    {
        self.use_text_mut(move |mut this| {
            TextArea::reset_text(&mut this, text().as_str());
        })
    }
}
