use std::mem::{Discriminant, discriminant};

use masonry::{
    TextAlign,
    core::{ArcStr, NewWidget, StyleProperty, Widget},
    widgets::Label,
};
use reactive_graph::graph::untrack;

use super::NewWidgetExt;

pub trait NewLabelExt {
    /// It is inefficient to call this function twice
    fn text<S, T>(self, text: S) -> Self
    where
        S: Fn() -> T + 'static,
        T: Into<ArcStr>;
    /// Reactive styles
    fn style<S, T>(self, style: S) -> Self
    where
        S: Fn() -> T + 'static,
        T: Into<StyleProperty>;
    /// Reactive option styles
    fn style_opt<S, T>(self, style: S) -> Self
    where
        S: Fn() -> Option<T> + 'static,
        T: Into<StyleProperty>;
    /// It is inefficient to call this function twice
    fn hint<S>(self, hint: S) -> Self
    where
        S: Fn() -> bool + 'static;
    /// It is inefficient to call this function twice
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

    fn style_opt<S, T>(mut self, style: S) -> Self
    where
        S: Fn() -> Option<T> + 'static,
        T: Into<StyleProperty>,
    {
        {
            if let Some(style) = untrack(&style) {
                self.widget = Box::new(self.widget.with_style(style));
            }
        }
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

    fn hint<S>(mut self, hint: S) -> Self
    where
        S: Fn() -> bool + 'static,
    {
        {
            self.widget = Box::new(self.widget.with_hint(untrack(&hint)));
        }
        self.use_reactive_widget_mut(move |mut this| {
            Label::set_hint(&mut this, hint());
        })
    }

    fn text_alignment<S>(mut self, align: S) -> Self
    where
        S: Fn() -> TextAlign + 'static,
    {
        {
            self.widget = Box::new(self.widget.with_text_alignment(untrack(&align)));
        }
        self.use_reactive_widget_mut(move |mut this| {
            Label::set_text_alignment(&mut this, align());
        })
    }
}

pub fn label<S, T>(text: S) -> NewWidget<Label>
where
    S: Fn() -> T + 'static,
    T: Into<ArcStr>,
{
    Label::new(untrack(&text)).with_auto_id().text(text)
}
