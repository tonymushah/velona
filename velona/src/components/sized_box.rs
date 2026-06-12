//! A reactive widget

use masonry::{
    core::{NewWidget, Widget},
    widgets::SizedBox,
};

use crate::{
    AnyNewWidget,
    widgets::{ReactiveSingleChildExt, sized_box::NewSizedBoxExt},
};

/// The function signature and the signature might sound dumb,
/// but this allows you to show widgets based on conditional logic, ect...
pub fn sized_box<F>(widget_fn: F) -> NewWidget<SizedBox>
where
    F: Fn() -> AnyNewWidget + 'static,
{
    SizedBox::empty()
        // .with_child(untrack(&mut widget_fn))
        .prepare()
        .child(widget_fn)
}

/// Similar to [`sized_box`] but with a [`Option<AnyNewWidget>`]
/// for the `widget_fn` return value.
pub fn sized_box_opt_child<F>(widget_fn: F) -> NewWidget<SizedBox>
where
    F: Fn() -> Option<AnyNewWidget> + 'static,
{
    SizedBox::empty()
        // .with_child(untrack(&mut widget_fn))
        .prepare()
        .child_opt(widget_fn)
}
