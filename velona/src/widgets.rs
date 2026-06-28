//! Various [`widgets`](masonry::widgets) extenstions
//!
//! TODO **custom implementation**
//!
//! - [x] [`Align`](masonry::widgets::Align)
//! - [x] [`Badge`](masonry::widgets::Badge)
//! - [x] [`Badged`](masonry_widgets::Badged)
//! - [x] [`Button`](masonry::widgets::Button)
//! - [x] [`Canvas`](masonry::widgets::Canvas)
//! - [x] [`Checkbox`](masonry::widgets::Checkbox) in [`checkbox`]
//! - [x] [`CollapsePanel`](masonry::widgets::CollapsePanel)
//! - [x] [`DisclosureButton`](masonry::widgets::DisclosureButton)
//! - [x] [`Divider`](masonry::widgets::Divider)
//! - [x] [`Flex`](masonry::widgets::Flex)
//! - [x] [`Grid`](masonry::widgets::Grid)
//! - [x] [`Image`](masonry::widgets::Image) in [`image`]
//! - [x] [`IndexedStack`](masonry::widgets::IndexedStack)
//! - [x] [`Label`](masonry::widgets::Label) in [`label`]
//! - [x] [`Pagination`](masonry::widgets::Pagination)
//! - [x] [`Passthrough`](masonry::widgets::Passthrough)
//! - [x] [`Portal`](masonry::widgets::Portal)
//! - [x] [`ProgressBar`](masonry::widgets::ProgressBar)
//! - [x] [`Prose`](masonry::widgets::Prose)
//! - [x] [`Radio`](masonry::widgets::RadioButton)
//! - [x] [`ResizeObserver`](masonry::widgets::ResizeObserver)
//! - [x] [`ScrollBar`](masonry::widgets::ScrollBar)
//! - [x] [`Selector`](masonry::widgets::Selector)
//! - [x] [`SelectorItem`](masonry::widgets::SelectorItem)
//! - [x] [`SizedBox`](masonry::widgets::SizedBox)
//! - [x] [`Slider`](masonry::widgets::Slider)
//! - [x] [`Spinner`](masonry::widgets::Spinner)
//! - [x] [`Split`](masonry::widgets::Split)
//! - [x] [`StepInput`](masonry::widgets::StepInput)
//! - [ ] [`Svg`](masonry::widgets::Svg)
//! - [ ] [`Switch`](masonry::widgets::Switch)
//! - [ ] [`TextArea`](masonry::widgets::TextArea)
//! - [ ] [`TextInput`](masonry::widgets::TextInput)
//! - [ ] [`VariableLabel`](masonry::widgets::VariableLabel)
//! - [ ] [`VirtualScroll`](masonry::widgets::VirtualScroll)
//! - [ ] [`ZStack`](masonry::widgets::ZStack)
// TODO add [new](NewWidget) for `New*Ext` widget trait doc comments
// TODO add doc comment for each module
// TODO aaa
pub mod align;
pub mod badge;
pub mod badged;
pub mod button;
pub mod canvas;
pub mod checkbox;
pub mod collapse_panel;
pub mod disclosure_button;
pub mod divider;
pub mod flex;
pub mod grid;
pub mod image;
pub mod indexed_stack;
pub mod label;
pub mod pagination;
pub mod portal;
pub mod progress;
pub mod prose;
pub mod radio;
pub mod resize_observer;
pub mod scrollbar;
pub mod selector;
pub mod selector_item;
pub mod sized_box;
pub mod slider;
pub mod split;
pub mod step_input;

use std::{any::type_name, marker::PhantomData, thread};

use log::warn;
use masonry::core::{NewWidget, Property, UsesProperty as HasProperty, Widget, WidgetMut};
use reactive_graph::{effect::Effect, graph::untrack};

use crate::{
    AnyNewWidget, widget_ref::VelonaWidgetRef, window::use_window,
    window_event_handler::register_window_event_handler,
};

// TODO add a `use_reactive_widget` with `WidgetRef` instead.
// TODO add a `use_reactive_widget_with_effect_val` with `WidgetRef` instead.
// TODO add documentation for this trait and its methods.
pub trait NewWidgetExt<W>
where
    W: Widget + 'static,
{
    /// Use [`WidgetMut`] inside an [`Effect`] with a value.
    ///
    /// Since its runs inside an [effect](Effect), any signal changes (subscription) will (re-)run the `fun`.
    ///
    /// The return value might useful if you want to track values between re-runs.
    fn use_reactive_widget_mut_with_effect_val<F, V>(self, fun: F) -> Self
    where
        F: FnMut(WidgetMut<'_, W>, Option<V>) -> Option<V> + 'static,
        V: 'static;
    /// Very similar to [`Self::use_reactive_widget_mut_with_effect_val`],
    /// but doesn't require a return value.
    fn use_reactive_widget_mut<F>(self, fun: F) -> Self
    where
        F: FnMut(WidgetMut<'_, W>) + 'static;

    /// Very similar to [`on`](Self::on) but uses a [`&self`](self) instead of [`self`].
    /// _You get the idea._
    fn on_ref_self<F>(&self, fun: F)
    where
        F: Fn(&W::Action) + 'static;
    /// Listen to the [`Widget::Action`]
    fn on<F>(self, fun: F) -> Self
    where
        F: Fn(&W::Action) + 'static;
    /// Set a [widget](Widget) [property](Property) reactively.
    fn property<F, P>(self, prop: F) -> Self
    where
        F: Fn() -> P + 'static,
        P: Property,
        W: HasProperty<P>;
    /// Use [`property`](Self::property) for reactive values
    fn static_propeperty<P>(self, prop: P) -> Self
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
                    log::warn!("cannot edit widget reactivelt => {err}");
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
    fn on_ref_self<F>(&self, fun: F)
    where
        F: Fn(&<W as Widget>::Action) + 'static,
    {
        register_window_event_handler(
            self.id(),
            Box::new(move |ev| {
                let Some(ev) = ev.downcast_ref::<W::Action>() else {
                    warn!("Cannot cast action");
                    return;
                };
                fun(ev);
            }),
        );
    }
    fn on<F>(self, fun: F) -> Self
    where
        F: Fn(&<W as Widget>::Action) + 'static,
    {
        self.on_ref_self(fun);
        self
    }
    /// It is worth mentioning that the `prop` function will be called immediately (inside an [`untrack`]) to set the property beforehand.
    /// After that, it will just passed inside a [`use_reactive_widget_mut`](Self::use_reactive_widget_mut).
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

    // TODO remove this
    fn static_propeperty<P>(mut self, prop: P) -> Self
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

/// Some widget has a single child with them. (like [button](masonry::widgets::Button), [align](masonry::widgets::Align))
///
/// This trait will unify all of those single child widgets "mutations" (aka `child_mut`) _instead of making duplicates method for those_.
pub trait SingleChildWidget {
    fn use_child_erased<C>(self, use_child_fn: C) -> Self
    where
        C: FnMut(WidgetMut<'_, dyn Widget>) + 'static;
    fn use_child_casted<C, W>(self, mut use_child_fn: C) -> Self
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

mod single_impl {
    use super::masonry_widgets::*;
    use super::{NewWidgetExt, SingleChildWidget};
    use masonry_core::core::{NewWidget, Widget, WidgetMut};
    #[cfg(doc)]
    use reactive_graph::effect::Effect;

    macro_rules! impl_single_widget {
        ($($widget:ty,)*) => {
            $(
                impl SingleChildWidget for NewWidget<$widget> {
                    /// It is worth mentioning that the `use_child_fn` will run inside an [`Effect`].
                    fn use_child_erased<C>(self, mut use_child_fn: C) -> Self
                    where
                        C: FnMut(WidgetMut<'_, dyn Widget>) + 'static
                    {
                        self.use_reactive_widget_mut(move |mut this| use_child_fn(<$widget>::child_mut(&mut this)))
                    }
                }
            )*
        };
    }

    impl_single_widget!(
        Align,
        Badge,
        Button,
        CollapsePanel,
        Passthrough,
        RadioGroup,
        ResizeObserver,
        // VirtualScroll,
    );
}

/// Similar to [`SingleChildWidget`] but the child is typed instead of erased.
// TODO implement for [`Portal`](masonry::widgets::Portal)
// TODO implement for [`Selector`](masonry::widgets::Selector)
// TODO implement for [`SelectorItem`](masonry::widgets::SelectorItem)
pub trait TypedSingleChildWidget {
    type Child: Widget + 'static;
    fn use_child<C>(self, use_child_fn: C) -> Self
    where
        C: FnMut(WidgetMut<'_, Self::Child>) + 'static;
}

impl<T> SingleChildWidget for T
where
    T: TypedSingleChildWidget,
{
    fn use_child_erased<C>(self, mut use_child_fn: C) -> Self
    where
        C: FnMut(WidgetMut<'_, dyn Widget>) + 'static,
    {
        <Self as TypedSingleChildWidget>::use_child(self, move |mut child| {
            if let Some(child) = child.try_downcast::<dyn Widget>() {
                use_child_fn(child);
            } else {
                log::warn!("Cannot cast to `dyn Widget`. (which is dumb)");
            }
        })
    }
}

/// Allows you to [`Widget`] `set_child` reactively.
///
/// This is only implemented for [`Widget`]s that has an erashed `set_child`
pub trait ReactiveSingleChildExt {
    fn child<Cf>(self, child_fn: Cf) -> Self
    where
        Cf: Fn() -> AnyNewWidget + 'static;
}

mod reactive_child_impl {
    use super::masonry_widgets::*;
    use super::{NewWidgetExt, ReactiveSingleChildExt};
    use crate::AnyNewWidget;
    use masonry_core::core::NewWidget;
    use reactive_graph::effect::Effect;
    use std::any::type_name;

    macro_rules! impl_reactive_child {
        ($($widget:ty,)*) => {
            $(
                impl ReactiveSingleChildExt for NewWidget<$widget> {
                    fn child<Cf>(self, child_fn: Cf) -> Self
                    where
                        Cf: Fn() -> AnyNewWidget + 'static
                    {
                        let w_ref = self.create_velona_ref();
                        Effect::new(move || {
                            let new_widget = child_fn();
                            let _ = w_ref
                                .edit_local_now(|mut this| {
                                    <$widget>::set_child(&mut this, new_widget);
                                })
                                .inspect_err(|err| {
                                    log::error!("Cannot set a new child for this widget {} => {err}", type_name::<$widget>());
                                });
                        });
                        self
                    }
                }
            )*
        };
    }

    impl_reactive_child!(
        Align,
        Badge,
        Button,
        CollapsePanel,
        Passthrough,
        ResizeObserver,
        // VirtualScroll,
    );
}

/// Allows you to [`Widget`] `set_child` reactively.
///
/// Unlike [`ReactiveSingleChildExt`], this trait is only implemented for [`Widget`]s that has a **typed** `set_child`.
// TODO implement for [`Portal`](masonry::widgets::Portal)
// TODO implement for [`Selector`](masonry::widgets::Selector)
// TODO implement for [`SelectorItem`](masonry::widgets::SelectorItem)
pub trait ReactiveSingleTypedChildExt {
    type Child: Widget + 'static;
    fn child<Cf>(self, child_fn: Cf) -> Self
    where
        Cf: Fn() -> NewWidget<Self::Child> + 'static;
}
