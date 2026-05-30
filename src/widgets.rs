pub mod checkbox;
pub mod label;

use std::{marker::PhantomData, thread};

use log::warn;
use masonry::core::{HasProperty, NewWidget, Property, Widget, WidgetMut};
use reactive_graph::{effect::Effect, graph::untrack};

use crate::{
    widget_ref::VelonaWidgetRef, window::use_window,
    window_event_handler::register_window_event_handler,
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
    /// Use [`property`](Self::property) for reactive values
    fn append_static_propeperty<P>(self, prop: P) -> Self
    where
        P: Property,
        W: HasProperty<P>;
    /// Update the internal [`NewWidget::widget`].
    // **NOTE: Please be smart and always use [`untrack`](reactive_graph::graph::untrack) if you use decide to bring a reactive closure on using this.**
    // Weird thing might happen if you do that.
    fn update_inner_widget<T>(self, update_fn: T) -> Self
    where
        T: FnOnce(W) -> W;
    /// Create a [`WidgetRef`](VelonaWidgetRef) that you can send safely between thread.
    fn create_velona_ref(&self) -> VelonaWidgetRef<W>;
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
        let widget_ref = self.create_velona_ref().disarm();
        Effect::new(move |v: Option<Option<V>>| {
            let v = v.flatten();
            match widget_ref.edit_local_now(|widget_mut| (fun)(widget_mut, v)) {
                Ok(val) => val,
                Err(err) => {
                    log::warn!("{err}");
                    None
                }
            }
            // weak_render_root
            //     .use_inner_render_root_mut::<_, Option<V>>(|rr| {
            //         if rr.tree.has_widget(widget_id) {
            //             rr.tree.edit_widget(widget_id, |mut widget_mut| {
            //                 let Some(widget_mut) = widget_mut.try_downcast::<W>() else {
            //                     warn!("The {:?} is not {:?}", widget_id, TypeId::of::<W>());
            //                     return None::<V>;
            //                 };
            //                 fun(widget_mut, v)
            //             })
            //         } else {
            //             warn!("No {:?} widget found", widget_id);
            //             None
            //         }
            //     })
            //     .flatten()
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

    fn update_inner_widget<T>(mut self, update_fn: T) -> Self
    where
        T: FnOnce(W) -> W,
    {
        self.widget = Box::new(update_fn(*self.widget));
        self
    }

    fn create_velona_ref(&self) -> VelonaWidgetRef<W> {
        VelonaWidgetRef {
            id: self.id(),
            window: use_window().map(Box::new),
            phantom: PhantomData::<W>,
            thread_id: thread::current().id(),
        }
    }
}

pub use masonry::widgets as masonry_widgets;
