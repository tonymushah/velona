//! Various [`widgets`](masonry::widgets) extenstions
//!
//! TODO **custom implementation**
//!
//! - [ ] [`Align`](masonry::widgets::Align)
//! - [ ] [`Badge`](masonry::widgets::Badge)
//! - [ ] [`Button`](masonry::widgets::Button)
//! - [ ] [`Canvas`](masonry::widgets::Canvas)
//! - [x] [`Checkbox`](masonry::widgets::Checkbox) in [`checkbox`]
//! - [ ] [`CollapsePanel`](masonry::widgets::CollapsePanel)
//! - [ ] [`DisclosureButton`](masonry::widgets::DisclosureButton)
//! - [ ] [`Divider`](masonry::widgets::Divider)
//! - [ ] [`Flex`](masonry::widgets::Flex)
//! - [ ] [`Grid`](masonry::widgets::Grid)
//! - [x] [`Image`](masonry::widgets::Image) in [`image`]
//! - [ ] [`IndexedStack`](masonry::widgets::IndexedStack)
//! - [x] [`Label`](masonry::widgets::Label) in [`label`]
//! - [ ] [`Pagination`](masonry::widgets::Pagination)
//! - [ ] [`Passthrough`](masonry::widgets::Passthrough)
//! - [ ] [`Portal`](masonry::widgets::Portal)
//! - [ ] [`ProgressBar`](masonry::widgets::ProgressBar)
//! - [ ] [`Prose`](masonry::widgets::Prose)
//! - [ ] [`Radio`](masonry::widgets::RadioButton)
//! - [ ] [`ResizeObserver`](masonry::widgets::ResizeObserver)
//! - [ ] [`ScrollBar`](masonry::widgets::ScrollBar)
//! - [ ] [`Selector`](masonry::widgets::Selector)
//! - [ ] [`SelectorItem`](masonry::widgets::Selector)
//! - [x] [`SizedBox`](masonry::widgets::SizedBox)
//! - [ ] [`Slider`](masonry::widgets::Slider)
//! - [ ] [`Spinner`](masonry::widgets::Spinner)
//! - [ ] [`Split`](masonry::widgets::Split)
//! - [ ] [`StepInput`](masonry::widgets::StepInput)
//! - [ ] [`Svg`](masonry::widgets::Svg)
//! - [ ] [`Switch`](masonry::widgets::Switch)
//! - [ ] [`TextArea`](masonry::widgets::TextArea)
//! - [ ] [`TextInput`](masonry::widgets::TextInput)
//! - [ ] [`VariableLabel`](masonry::widgets::VariableLabel)
//! - [ ] [`VirtualScroll`](masonry::widgets::VirtualScroll)
//! - [ ] [`ZStack`](masonry::widgets::ZStack)
pub mod checkbox;
pub mod image;
pub mod label;
pub mod sized_box;

use std::{marker::PhantomData, thread};

use log::warn;
use masonry::core::{NewWidget, Property, UsesProperty as HasProperty, Widget, WidgetMut};
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
