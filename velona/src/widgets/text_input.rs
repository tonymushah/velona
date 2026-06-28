use masonry::{
    core::{ArcStr, NewWidget, WidgetMut},
    parley,
    widgets::{Label, TextAction, TextArea, TextInput},
};

#[cfg(doc)]
use reactive_graph::effect::Effect;

use crate::{NewWidgetExt, utils::register_typed_widget_action_handler};

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

/// Since a [`TextInput`] is a [`TextArea`] wrapper,
/// it might be quite complex to handle event _via [`NewTextInputExt::use_text_mut`]_.
///
/// This trait provides a [`on_text_action`](Self::on_text_action) to listen to the internal [`TextArea`] action.
pub trait NewTextInputActionExt {
    /// Handle the internal [`TextArea`] [`TextAction`]s.
    fn on_text_action<H>(self, on_action: H) -> Self
    where
        H: Fn(&TextAction) + 'static;
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
