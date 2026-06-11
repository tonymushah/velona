use masonry::{core::NewWidget, widgets::Button};
use reactive_graph::effect::Effect;

use crate::{AnyNewWidget, NewWidgetExt, utils::ConsumeResult};

/// A [new](NewWidget) [`Button`] exenstion trait
pub trait NewButton {
    /// Make the button [child](Button::set_child) reactive.
    fn child<C>(self, child: C) -> Self
    where
        C: Fn() -> AnyNewWidget + 'static;
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
}
