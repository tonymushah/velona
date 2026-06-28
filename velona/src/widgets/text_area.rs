use std::mem::{Discriminant, discriminant};

use masonry::{
    TextAlign,
    core::{NewWidget, StyleProperty},
    widgets::{InsertNewline, TextArea},
};

use crate::NewWidgetExt;

/// A [new](NewWidget) [`TextArea`] trait extension.
pub trait NewTextAreaExt<const USER_EDITABLE: bool> {
    /// Reactive text styles.
    fn style<S, T>(self, style: S) -> Self
    where
        S: Fn() -> T + 'static,
        T: Into<StyleProperty>;
    /// Reactive option text styles
    fn style_opt<S, T>(self, style: S) -> Self
    where
        S: Fn() -> Option<T> + 'static,
        T: Into<StyleProperty>;
    /// Sets whether hinting will be used for this text area.
    ///
    /// The runtime equivalent of [`with_hint`](TextArea::with_hint).
    /// For full documentation, see that method.
    fn hint<S>(self, hint: S) -> Self
    where
        S: Fn() -> bool + 'static;
    /// Sets the [text alignment](https://en.wikipedia.org/wiki/Typographic_alignment) of the text.
    ///
    /// The reactive equivalent of [`with_text_alignment`](TextArea::with_text_alignment).
    fn text_alignment<S>(self, align: S) -> Self
    where
        S: Fn() -> TextAlign + 'static;
    /// Sets [word wrapping](https://en.wikipedia.org/wiki/Line_wrap_and_word_wrap) for the text area.
    ///
    /// When enabled, the text will be laid out to fit within the available width.
    /// If word wrapping is disabled, the text will likely flow past the available area.
    /// Note that parent widgets will often clip this, so the overflow will not be visible.
    ///
    /// This widget does not currently support scrolling to the cursor,
    /// so it is recommended to leave word wrapping enabled.
    ///
    /// The reactive equivalent of [`with_word_wrap`](TextArea::with_word_wrap).
    fn word_wrap<W>(self, wrap_words: W) -> Self
    where
        W: Fn() -> bool + 'static;
    /// Configures how this text area handles the user pressing Enter <kbd>↵</kbd>.
    ///
    /// The reactive equivalent for [`with_insert_newline`](TextArea::with_insert_newline).
    fn insert_newline<I>(self, insert_newline: I) -> Self
    where
        I: Fn() -> InsertNewline + 'static;
}

impl<const USER_EDITABLE: bool> NewTextAreaExt<USER_EDITABLE>
    for NewWidget<TextArea<USER_EDITABLE>>
{
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
        self.use_reactive_widget_mut(move |mut this| {
            TextArea::set_hint(&mut this, hint());
        })
    }

    fn text_alignment<S>(self, align: S) -> Self
    where
        S: Fn() -> TextAlign + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            TextArea::set_text_alignment(&mut this, align());
        })
    }

    fn word_wrap<W>(self, wrap_words: W) -> Self
    where
        W: Fn() -> bool + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            TextArea::set_word_wrap(&mut this, wrap_words());
        })
    }

    fn insert_newline<I>(self, insert_newline: I) -> Self
    where
        I: Fn() -> InsertNewline + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            TextArea::set_insert_newline(&mut this, insert_newline());
        })
    }
}
