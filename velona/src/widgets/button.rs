use std::any::type_name;

use masonry::{
    core::{NewWidget, Widget, WidgetMut},
    widgets::Button,
};
use reactive_graph::effect::Effect;

use crate::{AnyNewWidget, NewWidgetExt, utils::ConsumeResult};

/// A [new](NewWidget) [`Button`] exenstion trait
pub trait NewButton {
    /// Make the button [child](Button::set_child) reactive.
    fn child<C>(self, child: C) -> Self
    where
        C: Fn() -> AnyNewWidget + 'static;
    fn use_child_erased<C>(self, use_child_fn: C) -> Self
    where
        C: FnMut(WidgetMut<'_, dyn Widget>) + 'static;
    fn use_child<C, W>(self, mut use_child_fn: C) -> Self
    where
        C: FnMut(WidgetMut<'_, W>) + 'static,
        W: Widget + 'static,
        Self: Sized,
    {
        self.use_child_erased(move |mut child| {
            if let Some(child) = child.try_downcast::<W>() {
                use_child_fn(child);
            } else {
                log::warn!(
                    "Invalid downcast. (expected {}, found {:?})",
                    type_name::<W>(),
                    child.widget.type_id()
                );
            }
        })
    }
}

impl NewButton for NewWidget<Button> {
    fn child<C>(self, child: C) -> Self
    where
        C: Fn() -> AnyNewWidget + 'static,
    {
        let v_ref = self.create_velona_ref();
        Effect::new(move || {
            let new_widget = child();
            v_ref
                .edit_local_now(|mut this| {
                    Button::set_child(&mut this, new_widget);
                })
                .consume_with_log_err();
        });
        self
    }

    fn use_child_erased<C>(self, mut use_child_fn: C) -> Self
    where
        C: FnMut(WidgetMut<'_, dyn Widget>) + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| use_child_fn(Button::child_mut(&mut this)))
    }
}
