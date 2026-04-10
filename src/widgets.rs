pub mod label;

use std::any::TypeId;

use log::warn;
use masonry::core::{HasProperty, NewWidget, Property, Widget, WidgetMut};
use reactive_graph::{effect::Effect, graph::untrack};

use crate::{
    render_root::use_weak_render_root, window_event_handler::register_window_event_handler,
};

pub trait NewWidgetExt<W>
where
    W: Widget + 'static,
{
    fn use_reactive_widget_mut_with_effect_val<F, V>(self, fun: F) -> Self
    where
        F: FnMut(WidgetMut<'_, W>, Option<V>) -> Option<V> + 'static,
        V: 'static;
    fn use_reactive_widget_mut<F>(self, fun: F) -> Self
    where
        F: FnMut(WidgetMut<'_, W>) + 'static;

    fn register_handler<F>(self, fun: F) -> Self
    where
        F: Fn(&W::Action) + 'static;
    fn property<F, P>(self, prop: F) -> Self
    where
        F: Fn() -> P + 'static,
        P: Property,
        W: HasProperty<P>;
    /// Use [`property`] for reactive values
    fn append_static_propeperty<P>(self, prop: P) -> Self
    where
        P: Property,
        W: HasProperty<P>;
}

impl<W> NewWidgetExt<W> for NewWidget<W>
where
    W: Widget + 'static,
{
    fn use_reactive_widget_mut_with_effect_val<F, V>(self, mut fun: F) -> Self
    where
        F: FnMut(WidgetMut<'_, W>, Option<V>) -> Option<V> + 'static,
        V: 'static,
    {
        let widget_id = self.id();
        Effect::new(move |v: Option<Option<V>>| {
            let v = v.flatten();
            let Some(weak_render_root) = use_weak_render_root() else {
                warn!("No render root found");
                return None;
            };
            weak_render_root
                .use_inner_render_root_mut::<_, Option<V>>(|rr| {
                    if rr.tree.has_widget(widget_id) {
                        rr.tree.edit_widget(widget_id, |mut widget_mut| {
                            let Some(widget_mut) = widget_mut.try_downcast::<W>() else {
                                warn!("The {:?} is not {:?}", widget_id, TypeId::of::<W>());
                                return None::<V>;
                            };
                            fun(widget_mut, v)
                        })
                    } else {
                        warn!("No {:?} widget found", widget_id);
                        None
                    }
                })
                .flatten()
        });
        self
    }
    fn use_reactive_widget_mut<F>(self, mut fun: F) -> Self
    where
        F: FnMut(WidgetMut<'_, W>) + 'static,
    {
        self.use_reactive_widget_mut_with_effect_val::<_, ()>(move |this, _| {
            fun(this);
            None
        })
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
        W: HasProperty<P>,
    {
        self.properties.insert(untrack(&prop));
        self.use_reactive_widget_mut(move |mut this| {
            this.insert_prop::<P>(prop());
        })
    }

    fn append_static_propeperty<P>(mut self, prop: P) -> Self
    where
        P: Property,
        W: HasProperty<P>,
    {
        self.properties.insert(prop);

        self
    }
}
