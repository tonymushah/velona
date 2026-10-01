use std::mem::{Discriminant, discriminant};

use masonry::{
    core::{StyleProperty, WidgetMut},
    widgets::{Label, TextArea},
};
use velona_core::widgets::UseWidgetValResult;

pub(crate) enum LabelStyleAction {
    Add(Box<StyleProperty>),
    Remove(Discriminant<StyleProperty>),
}

pub(crate) fn apply_label_style_actions(
    mut this: WidgetMut<'_, Label>,
    actions: Box<[LabelStyleAction]>,
) {
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

pub(crate) fn apply_text_style_actions<const USER_EDITABLE: bool>(
    mut this: WidgetMut<'_, TextArea<USER_EDITABLE>>,
    actions: Box<[LabelStyleAction]>,
) {
    for action in actions {
        match action {
            LabelStyleAction::Add(style_property) => {
                TextArea::insert_style(&mut this, *style_property);
            }
            LabelStyleAction::Remove(discriminant) => {
                TextArea::remove_style(&mut this, discriminant);
            }
        }
    }
}

pub(crate) fn get_style_opt_action(
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
