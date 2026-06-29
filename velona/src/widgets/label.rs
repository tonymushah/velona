use std::{
    mem::{Discriminant, discriminant},
    sync::{self, Mutex},
};

use masonry::{
    TextAlign,
    core::{ArcStr, NewWidget, StyleProperty},
    widgets::Label,
};

use crate::widgets::TypedSingleChildWidget;

use super::NewWidgetExt;

/// A [`Label`] trait extention
pub trait NewLabelExt {
    /// It is inefficient to call this function twice.
    fn text<S, T>(self, text: S) -> Self
    where
        S: Fn() -> T + 'static,
        T: Into<ArcStr>;
    /// Reactive text styles.
    fn style<S, T>(self, style: S) -> Self
    where
        S: Fn() -> T + 'static,
        T: Into<StyleProperty>;
    /// Reactive optional text styles.
    fn style_opt<S, T>(self, style: S) -> Self
    where
        S: Fn() -> Option<T> + 'static,
        T: Into<StyleProperty>;
    /// The reactive equivalent of [`with_hint`](Label::with_hint).
    fn hint<S>(self, hint: S) -> Self
    where
        S: Fn() -> bool + 'static;
    /// The reactive equivalent of [`with_text_alignment`](Label::with_text_alignment).
    fn text_alignment<S>(self, align: S) -> Self
    where
        S: Fn() -> TextAlign + 'static;
}

impl NewLabelExt for NewWidget<Label> {
    fn text<S, T>(self, text: S) -> Self
    where
        S: Fn() -> T + 'static,
        T: Into<ArcStr>,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Label::set_text(&mut this, text());
        })
    }

    fn style_opt<S, T>(self, style: S) -> Self
    where
        S: Fn() -> Option<T> + 'static,
        T: Into<StyleProperty>,
    {
        self.use_reactive_widget_mut_with_effect_val::<_, Discriminant<StyleProperty>>(
            move |mut this, old_style| {
                if let Some(old_style) = old_style {
                    Label::remove_style(&mut this, old_style);
                }
                if let Some(style) = style() {
                    Label::insert_style(&mut this, style)
                        .as_ref()
                        .map(discriminant)
                } else {
                    None
                }
            },
        )
    }
    fn style<S, T>(self, style: S) -> Self
    where
        S: Fn() -> T + 'static,
        T: Into<StyleProperty>,
    {
        self.style_opt(move || Some(style()))
    }

    fn hint<S>(self, hint: S) -> Self
    where
        S: Fn() -> bool + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Label::set_hint(&mut this, hint());
        })
    }

    fn text_alignment<S>(self, align: S) -> Self
    where
        S: Fn() -> TextAlign + 'static,
    {
        // {
        //     self.widget = Box::new(self.widget.with_text_alignment(untrack(&align)));
        // }
        self.use_reactive_widget_mut(move |mut this| {
            Label::set_text_alignment(&mut this, align());
        })
    }
}

impl<W> NewLabelExt for W
where
    W: TypedSingleChildWidget<Child = Label> + 'static,
{
    fn text<S, T>(self, text: S) -> Self
    where
        S: Fn() -> T + 'static,
        T: Into<ArcStr>,
    {
        self.use_child(move |mut this| {
            Label::set_text(&mut this, text());
        })
    }

    fn style_opt<S, T>(self, style: S) -> Self
    where
        S: Fn() -> Option<T> + 'static,
        T: Into<StyleProperty>,
    {
        let old_style_data = sync::Arc::new(Mutex::new(None::<Discriminant<StyleProperty>>));
        self.use_child(move |mut this| {
            let mut old_style_lock = {
                if old_style_data.is_poisoned() {
                    old_style_data.clear_poison();
                }
                old_style_data.lock().unwrap()
            };
            if let Some(old_style) = *old_style_lock {
                Label::remove_style(&mut this, old_style);
            }
            if let Some(style) = style() {
                *old_style_lock = Label::insert_style(&mut this, style)
                    .as_ref()
                    .map(discriminant);
            } else {
                *old_style_lock = None;
            }
        })
    }
    fn style<S, T>(self, style: S) -> Self
    where
        S: Fn() -> T + 'static,
        T: Into<StyleProperty>,
    {
        self.style_opt(move || Some(style()))
    }

    fn hint<S>(self, hint: S) -> Self
    where
        S: Fn() -> bool + 'static,
    {
        self.use_child(move |mut this| {
            Label::set_hint(&mut this, hint());
        })
    }

    fn text_alignment<S>(self, align: S) -> Self
    where
        S: Fn() -> TextAlign + 'static,
    {
        // {
        //     self.widget = Box::new(self.widget.with_text_alignment(untrack(&align)));
        // }
        self.use_child(move |mut this| {
            Label::set_text_alignment(&mut this, align());
        })
    }
}
