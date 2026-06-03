//! A reactive widget

use masonry::{
    core::{NewWidget, Widget},
    widgets::SizedBox,
};

use crate::{AnyNewWidget, widgets::sized_box::NewSizedBoxExt};

/// The function signature and the signature might sound dumb,
/// but this allows you to show widgets based on conditional logic, ect...
pub fn sized_box<F>(widget_fn: F) -> NewWidget<SizedBox>
where
    F: FnMut() -> AnyNewWidget + 'static,
{
    SizedBox::empty()
        // .with_child(untrack(&mut widget_fn))
        .with_auto_id()
        .child(widget_fn)
}
