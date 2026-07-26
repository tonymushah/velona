//! Various [`TextInput`] implementations.
//!
//! The most important thing in the module is the [`NewTextInputExt`]
//! which is implemented for [`NewWidget<TextInput>`].
//!
//! There is also the [`NewTextInputActionExt`] to handle the internal [`TextArea`] actions.
//!
//! [`NewWidget<TextInput>`] also implements the [`NewTextAreaExt<true>`] trait.
//!
//! _See the [widget](TextInput) documentation for more information_.

use std::mem::{Discriminant, discriminant};

use masonry::{
    TextAlign,
    core::{ArcStr, NewWidget, StyleProperty, WidgetMut},
    parley,
    widgets::{InsertNewline, Label, TextAction, TextArea, TextInput},
};

use reactive_graph::actions;
#[cfg(doc)]
use reactive_graph::effect::Effect;

use crate::{
    NewWidgetExt, utils::register_typed_widget_action_handler, widgets::text_area::NewTextAreaExt,
};

/// A [new](NewWidget) [`TextInput`] trait extension.
// TODO add example
pub trait NewTextInputExt {
    /// Edits the underlying text area.
    ///
    /// Used to modify most properties of the text.
    ///
    /// It is worth noting that the `use_fn` runs inside an [`Effect`].
    fn use_text_mut<U>(self, use_fn: U) -> Self
    where
        U: FnMut(WidgetMut<TextArea<true>>) + 'static;
    /// Edits the child label representing the placeholder text.
    ///
    /// It is worth noting that the `use_fn` runs inside an [`Effect`].
    fn use_placeholder_mut<U>(self, use_fn: U) -> Self
    where
        U: FnMut(WidgetMut<Label>) + 'static;
    /// The text that will be displayed when this input is empty.
    ///
    /// The reactive equivalent of [`with_placeholder`](TextInput::with_placeholder).
    fn placeholder<P, T>(self, placeholder_text: P) -> Self
    where
        P: Fn() -> T + 'static,
        T: Into<ArcStr> + 'static;
    /// Whether to clip the text to the drawn boundaries.
    ///
    /// If this is set to true, it is recommended, but not required, that this
    /// wraps a text area with [word wrapping](TextArea::set_word_wrap) enabled.
    ///
    /// The reactive equivalent of [`with_clip`](TextInput::with_clip).
    fn clip<C>(self, clip: C) -> Self
    where
        C: Fn() -> bool + 'static;
    /// Sets the text alignment for both the input text and placeholder.
    ///
    /// The reactive equivalent of [`with_text_alignment`](TextInput::text_alignment).
    fn text_alignment<A>(self, text_alignment: A) -> Self
    where
        A: Fn() -> parley::Alignment + 'static;
}

impl NewTextInputExt for NewWidget<TextInput> {
    fn use_text_mut<U>(self, mut use_fn: U) -> Self
    where
        U: FnMut(WidgetMut<TextArea<true>>) + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            use_fn(TextInput::text_mut(&mut this));
        })
    }

    fn use_placeholder_mut<U>(self, mut use_fn: U) -> Self
    where
        U: FnMut(WidgetMut<Label>) + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            use_fn(TextInput::placeholder_mut(&mut this));
        })
    }

    fn placeholder<P, T>(self, placeholder_text: P) -> Self
    where
        P: Fn() -> T + 'static,
        T: Into<ArcStr> + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            TextInput::set_placeholder(&mut this, placeholder_text());
        })
    }

    fn clip<C>(self, clip: C) -> Self
    where
        C: Fn() -> bool + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            TextInput::set_clip(&mut this, clip());
        })
    }

    fn text_alignment<A>(self, text_alignment: A) -> Self
    where
        A: Fn() -> parley::Alignment + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            TextInput::set_text_alignment(&mut this, text_alignment());
        })
    }
}

impl NewTextAreaExt<true> for NewWidget<TextInput> {
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
                let mut this = TextInput::text_mut(&mut this);
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

/// Since a [`TextInput`] is a [`TextArea`] wrapper,
/// it might be quite complex to handle event _via [`NewTextInputExt::use_text_mut`]_.
///
/// This trait provides a [`on_text_action`](Self::on_text_action) to listen to the internal [`TextArea`] action.
pub trait NewTextInputActionExt {
    /// Handle the internal [`TextArea`] [`TextAction`]s.
    fn on_text_action<H>(self, on_action: H) -> Self
    where
        H: Fn(&TextAction) + 'static;
    /// Handle the internal [`TextArea`] [`TextAction::Changed`]s.
    fn on_text_action_changed<H>(self, on_changed: H) -> Self
    where
        H: Fn(&String) + 'static,
        Self: Sized,
    {
        self.on_text_action(move |action| {
            if let TextAction::Changed(changes) = action {
                on_changed(changes);
            }
        })
    }
    /// Handle the internal [`TextArea`] [`TextAction::Entered`]s.
    fn on_text_action_entered<H>(self, on_entered: H) -> Self
    where
        H: Fn(&String) + 'static,
        Self: Sized,
    {
        self.on_text_action(move |action| {
            if let TextAction::Entered(changes) = action {
                on_entered(changes);
            }
        })
    }
}

impl NewTextInputActionExt for NewWidget<TextInput> {
    fn on_text_action<H>(self, on_action: H) -> Self
    where
        H: Fn(&TextAction) + 'static,
    {
        register_typed_widget_action_handler::<TextArea<false>, _>(
            self.widget.area_pod().id(),
            on_action,
        );
        self
    }
}
