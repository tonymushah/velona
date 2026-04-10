use std::any::TypeId;

use log::warn;
use masonry::core::{NewWidget, Property, Widget, WidgetMut};
use reactive_graph::{effect::Effect, graph::untrack};

use crate::{
    render_root::use_weak_render_root, window_event_handler::register_window_event_handler,
};

pub trait NewWidgetExt<W>
where
    W: Widget + 'static,
{
    fn use_reactive_widget_mut<F>(self, fun: F) -> Self
    where
        F: FnMut(WidgetMut<'_, W>) + 'static;
    fn register_handler<F>(self, fun: F) -> Self
    where
        F: Fn(&W::Action) + 'static;
    fn property<F, P>(self, prop: F) -> Self
    where
        F: Fn() -> P + 'static,
        P: Property;
}

impl<W> NewWidgetExt<W> for NewWidget<W>
where
    W: Widget + 'static,
{
    fn use_reactive_widget_mut<F>(self, mut fun: F) -> Self
    where
        F: FnMut(WidgetMut<'_, W>) + 'static,
    {
        let widget_id = self.id();
        Effect::new(move || {
            let Some(weak_render_root) = use_weak_render_root() else {
                warn!("No render root found");
                return;
            };
            weak_render_root.use_inner_render_root_mut(|rr| {
                if rr.tree.has_widget(widget_id) {
                    rr.tree.edit_widget::<()>(widget_id, |mut widget_mut| {
                        let Some(widget_mut) = widget_mut.try_downcast::<W>() else {
                            warn!("The {:?} is not {:?}", widget_id, TypeId::of::<W>());
                            return;
                        };
                        fun(widget_mut);
                    });
                } else {
                    warn!("No {:?} widget found", widget_id)
                }
            });
        });
        self
    }

    fn register_handler<F>(self, fun: F) -> Self
    where
        F: Fn(&<W as Widget>::Action) + 'static,
    {
        register_window_event_handler(
            self.id(),
            Box::new(move |ev| {
                let Some(ev) = ev.downcast_ref::<W::Action>() else {
                    warn!("Added s");
                    return;
                };
                fun(ev);
            }),
        );
        self
    }
    fn property<F, P>(mut self, prop: F) -> Self
    where
        F: Fn() -> P + 'static,
        P: Property,
    {
        self.properties.insert(untrack(&prop));
        self.use_reactive_widget_mut(move |mut this| {
            this.insert_prop::<P>(prop());
        })
    }
}
