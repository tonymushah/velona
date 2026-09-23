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

use std::mem::Discriminant;

use masonry::{
    TextAlign,
    core::{ArcStr, NewWidget, StyleProperty, WidgetMut},
    parley,
    widgets::{InsertNewline, Label, TextAction, TextArea, TextInput},
};

#[cfg(doc)]
use velona_core::reactive::effect::Effect;
use velona_core::{utils::register_typed_widget_action_listener, widgets::UseWidgetValResult};

use crate::{
    NewWidgetExt,
    utils::text_style::{apply_text_style_actions, get_style_opt_action},
    widgets::text_area::NewTextAreaExt,
};

/// A [new](NewWidget) [`TextInput`] trait extension.
// TODO add example
pub trait NewTextInputExt {
    /// Edits the underlying text area.
    ///
    /// Used to modify most properties of the text.
    ///
    /// It is worth noting that only the `val_fn` runs inside an [`Effect`].
    fn use_text_mut<Vfn, Efn, V, O>(self, val_fn: Vfn, edit_fn: Efn) -> Self
    where
        Efn: FnMut(WidgetMut<'_, TextArea<true>>, V) + 'static,
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static;
    /// Edits the child label representing the placeholder text.
    ///
    /// It is worth noting that only the `val_fn` runs inside an [`Effect`].
    fn use_placeholder_mut<Vfn, Efn, V, O>(self, val_fn: Vfn, edit_fn: Efn) -> Self
    where
        Efn: FnMut(WidgetMut<'_, Label>, V) + 'static,
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static;
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
    fn placeholder<P, T>(self, placeholder_text: P) -> Self
    where
        P: Fn() -> T + 'static,
        T: Into<ArcStr> + 'static,
    {
        self.use_widget_mut(placeholder_text, |mut this, text| {
            TextInput::set_placeholder(&mut this, text);
        })
    }

    fn clip<C>(self, clip: C) -> Self
    where
        C: Fn() -> bool + 'static,
    {
        self.use_widget_mut(clip, |mut this, clip| {
            TextInput::set_clip(&mut this, clip);
        })
    }

    fn text_alignment<A>(self, text_alignment: A) -> Self
    where
        A: Fn() -> parley::Alignment + 'static,
    {
        self.use_widget_mut(text_alignment, |mut this, text_alignment| {
            TextInput::set_text_alignment(&mut this, text_alignment);
        })
    }

    fn use_text_mut<Vfn, Efn, V, O>(self, val_fn: Vfn, mut edit_fn: Efn) -> Self
    where
        Efn: FnMut(WidgetMut<'_, TextArea<true>>, V) + 'static,
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static,
    {
        self.use_widget_mut_val(val_fn, move |mut this, val| {
            edit_fn(TextInput::text_mut(&mut this), val);
        })
    }

    fn use_placeholder_mut<Vfn, Efn, V, O>(self, val_fn: Vfn, mut edit_fn: Efn) -> Self
    where
        Efn: FnMut(WidgetMut<'_, Label>, V) + 'static,
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static,
    {
        self.use_widget_mut_val(val_fn, move |mut this, val| {
            edit_fn(TextInput::placeholder_mut(&mut this), val);
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
            |mut this, wrap| {
                TextArea::set_word_wrap(&mut this, wrap);
            },
        )
    }

    fn insert_newline<I>(self, insert_newline: I) -> Self
    where
        I: Fn() -> InsertNewline + 'static,
    {
        self.use_text_mut(
            move |_| UseWidgetValResult::to_edit_fn(insert_newline()),
            |mut this, insert| {
                TextArea::set_insert_newline(&mut this, insert);
            },
        )
    }

    fn text<Tfn, T>(self, text: Tfn) -> Self
    where
        T: AsRef<str> + 'static,
        Tfn: Fn() -> T + 'static,
    {
        self.use_text_mut(
            move |_| UseWidgetValResult::to_edit_fn(text()),
            |mut this, text| {
                TextArea::reset_text(&mut this, text.as_ref());
            },
        )
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
        H: Fn(&TextAction) + Send + 'static;
    /// Handle the internal [`TextArea`] [`TextAction::Changed`]s.
    fn on_text_action_changed<H>(self, on_changed: H) -> Self
    where
        H: Fn(&String) + Send + 'static,
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
        H: Fn(&String) + Send + 'static,
        Self: Sized,
    {
        self.on_text_action(move |action| {
            if let TextAction::Entered(changes) = action {
                on_entered(changes);
            }
        })
    }
    /// Handle the internal [`TextArea`] [`TextAction::Cancelled`]s.
    fn on_text_action_cancelled<H>(self, on_entered: H) -> Self
    where
        H: Fn() + Send + 'static,
        Self: Sized,
    {
        self.on_text_action(move |action| {
            if let TextAction::Cancelled = action {
                on_entered();
            }
        })
    }
}

impl NewTextInputActionExt for NewWidget<TextInput> {
    fn on_text_action<H>(self, on_action: H) -> Self
    where
        H: Fn(&TextAction) + Send + 'static,
    {
        register_typed_widget_action_listener::<TextArea<false>, _>(
            self.widget.area_pod().id(),
            on_action,
        );
        self
    }
}
