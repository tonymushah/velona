use masonry::{core::NewWidget, layout::UnitPoint, widgets::Align};
use reactive_graph::effect::Effect;

use crate::{AnyNewWidget, NewWidgetExt};

/// A [`Align`] trait extension
pub trait NewAlign {
    /// Make the [`Align::set_child`] reactive
    fn child<C>(self, child: C) -> Self
    where
        C: Fn() -> AnyNewWidget + 'static;
    /// Make the [`Align::set_alignment`] reactive
    fn alignment<A>(self, alignment: A) -> Self
    where
        A: Fn() -> UnitPoint + 'static;
}

impl NewAlign for NewWidget<Align> {
    fn child<C>(self, child: C) -> Self
    where
        C: Fn() -> AnyNewWidget + 'static,
    {
        let w_ref = self.create_velona_ref();
        Effect::new(move || {
            let child = child();
            let _ = w_ref
                .edit_local_now(|mut this| {
                    Align::set_child(&mut this, child);
                })
                .inspect_err(|err| log::error!("Cannot edit widget locally => {err}"));
        });
        self
    }

    fn alignment<A>(self, alignment: A) -> Self
    where
        A: Fn() -> UnitPoint + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Align::set_alignment(&mut this, alignment());
        })
    }
}
