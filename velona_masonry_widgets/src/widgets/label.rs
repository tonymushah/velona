//! Various [`Label`] implementations.
//!
//! The most important thing in the module is the [`NewLabelExt`]
//! which is implemented for [`NewWidget<Label>`].
//!
//! [`NewChildedLabelExt`] is just a [`NewLabelExt`] for [`W: TypedSingleChildWidget<Child = Label> + 'static`](TypedSingleChildWidget) (for ease of use)
//!
//! _See the [widget](Label) documentation for more information_.

use std::mem::{Discriminant, discriminant};

use crate::widgets::TypedSingleChildWidget;
use masonry::{
    TextAlign,
    core::{ArcStr, NewWidget, StyleProperty, WidgetMut},
    widgets::Label,
};
use velona_core::widgets::UseWidgetValResult;
// use velona_core::widgets::TypedSingleChildWidget;

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
        self.use_widget_mut(
            move || text().into(),
            move |mut this, text| {
                Label::set_text(&mut this, text);
            },
        )
    }

    fn style_opt<S, T>(self, style: S) -> Self
    where
        S: Fn() -> Option<T> + 'static,
        T: Into<StyleProperty>,
    {
        self.use_widget_mut_val(
            move |old_style: Option<Discriminant<StyleProperty>>| {
                let new_style = style().map(Into::<StyleProperty>::into);
                get_style_opt_action(old_style, new_style)
            },
            apply_label_style_actions,
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
        self.use_widget_mut(hint, |mut this, hint| {
            Label::set_hint(&mut this, hint);
        })
    }

    fn text_alignment<S>(self, align: S) -> Self
    where
        S: Fn() -> TextAlign + 'static,
    {
        // {
        //     self.widget = Box::new(self.widget.with_text_alignment(untrack(&align)));
        // }
        self.use_widget_mut(align, |mut this, align| {
            Label::set_text_alignment(&mut this, align);
        })
    }
}

fn get_style_opt_action(
    old_style: Option<
        Discriminant<masonry::parley::StyleProperty<'static, masonry::core::BrushIndex>>,
    >,
    new_style: Option<masonry::parley::StyleProperty<'static, masonry::core::BrushIndex>>,
) -> UseWidgetValResult<
    Box<[LabelStyleAction]>,
    Discriminant<masonry::parley::StyleProperty<'static, masonry::core::BrushIndex>>,
> {
    let new_style_discrimant = new_style.as_ref().map(discriminant);
    let mut instructions = Vec::<LabelStyleAction>::with_capacity(2);
    match (new_style, old_style) {
        (None, None) => {}
        (None, Some(old)) => {
            instructions.push(LabelStyleAction::Remove(old));
        }
        (Some(new), None) => {
            instructions.push(LabelStyleAction::Add(Box::new(new)));
        }
        (Some(new), Some(old)) => {
            if discriminant(&new) == old {
                instructions.push(LabelStyleAction::Add(Box::new(new)));
            } else {
                instructions.push(LabelStyleAction::Remove(old));
                instructions.push(LabelStyleAction::Add(Box::new(new)));
            }
        }
    }
    UseWidgetValResult {
        to_edit_fn: instructions.into_boxed_slice(),
        to_next_effect_run: new_style_discrimant,
    }
}

/// [`NewLabelExt`] knock-off
pub trait NewChildedLabelExt {
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

impl<W> NewChildedLabelExt for W
where
    W: TypedSingleChildWidget<Child = Label> + 'static,
{
    fn text<S, T>(self, text: S) -> Self
    where
        S: Fn() -> T + 'static,
        T: Into<ArcStr>,
    {
        self.use_child(
            move |_| UseWidgetValResult::to_edit_fn(text().into()),
            |mut this, text| {
                Label::set_text(&mut this, text);
            },
        )
    }

    fn style_opt<S, T>(self, style: S) -> Self
    where
        S: Fn() -> Option<T> + 'static,
        T: Into<StyleProperty>,
    {
        self.use_child(
            move |old_style: Option<Discriminant<StyleProperty>>| {
                let new_style = style().map(Into::<StyleProperty>::into);
                get_style_opt_action(old_style, new_style)
            },
            apply_label_style_actions,
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
        self.use_child(
            move |_| UseWidgetValResult::to_edit_fn(hint()),
            |mut this, hint| {
                Label::set_hint(&mut this, hint);
            },
        )
    }

    fn text_alignment<S>(self, align: S) -> Self
    where
        S: Fn() -> TextAlign + 'static,
    {
        // {
        //     self.widget = Box::new(self.widget.with_text_alignment(untrack(&align)));
        // }
        self.use_child(
            move |_| UseWidgetValResult::to_edit_fn(align()),
            |mut this, align| {
                Label::set_text_alignment(&mut this, align);
            },
        )
    }
}

enum LabelStyleAction {
    Add(Box<StyleProperty>),
    Remove(Discriminant<StyleProperty>),
}

fn apply_label_style_actions(mut this: WidgetMut<'_, Label>, actions: Box<[LabelStyleAction]>) {
    for action in actions {
        match action {
            LabelStyleAction::Add(style_property) => {
                Label::insert_style(&mut this, *style_property);
            }
            LabelStyleAction::Remove(discriminant) => {
                Label::remove_style(&mut this, discriminant);
            }
        }
    }
}
