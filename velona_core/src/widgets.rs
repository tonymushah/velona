use std::{any::type_name, marker::PhantomData, thread};

#[cfg(doc)]
use masonry_core::core::MutateCtx;
use masonry_core::{
    core::{NewWidget, Property, PropertyStackId, UsesProperty as HasProperty, Widget, WidgetMut},
    kurbo::Affine,
};
use reactive_graph::{effect::Effect, graph::untrack};

use crate::{
    AnyNewWidget, widget_ref::VelonaWidgetRef, window::use_window,
    window_event_handler::register_typed_widget_action_handler,
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

    /// Very similar to [`on`](Self::on_action) but uses a [`&self`](self) instead of [`self`].
    /// _You get the idea._
    fn on_action_ref_self<F>(&self, fun: F)
    where
        F: Fn(&W::Action) + Send + 'static;
    /// Listen to the [`Widget::Action`]
    fn on_action<F>(self, fun: F) -> Self
    where
        F: Fn(&W::Action) + Send + 'static;
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
    /// Set class the new widget class reactively.
    ///
    /// When the value changes, the old one will be [removed](MutateCtx::remove_class).
    ///
    /// See [`MutateCtx::add_class`] and [`MutateCtx::remove_class`].
    fn class<C>(self, class: C) -> Self
    where
        C: Fn() -> String + 'static;
    /// Similar to [`NewWidgetExt::class`] but uses a [`Option<String>`] instead of [`String`].
    ///
    /// See [`MutateCtx::add_class`] and [`MutateCtx::remove_class`].
    fn class_opt<C>(self, class: C) -> Self
    where
        C: Fn() -> Option<String> + 'static;
    /// Similar to [`NewWidgetExt::class`] and [`NewWidgetExt::class_opt`] but uses a [`Vec<String>`] (aka a list of classes).
    ///
    /// When the values changes, the old classes with be [removed](MutateCtx::remove_class).
    ///
    /// See [`MutateCtx::add_class`] and [`MutateCtx::remove_class`].
    fn classes<C>(self, classes: C) -> Self
    where
        C: Fn() -> Vec<String> + 'static;
    /// Sets the disabled state for this widget.
    ///
    /// Setting this to `false` does not mean a widget is not still disabled;
    /// for instance it may still be disabled by an ancestor.
    /// See [`MutateCtx::is_disabled`] for more information.
    ///
    /// _Reactive version of [`MutateCtx::set_disabled`]_.
    fn disabled<D>(self, disabled: D) -> Self
    where
        D: Fn() -> bool + 'static;
    /// Sets the disabled state for this widget.
    ///
    /// Unlike the [`disabled`](Self::disabled), the function of this one have a `bool` param with it
    /// which is the [`MutateCtx::is_disabled`] return value.
    fn disabled_with_current<D>(self, disabled: D) -> Self
    where
        D: Fn(bool) -> bool + 'static;
    /// Sets the local transform for this widget.
    ///
    /// This maps this widget’s border-box coordinate space to the parent’s border-box coordinate space.
    ///
    /// It behaves similarly as CSS transforms.
    ///
    /// _Reactive version of [`MutateCtx::set_transform`]_.
    fn transform<T>(self, transform: T) -> Self
    where
        T: Fn() -> Affine + 'static;
    /// Sets which property stack this widget uses for property resolution.
    ///
    /// _Reactive version of [`MutateCtx::set_property_stack`]_.
    fn property_stack_id<P>(self, property_stack_id: P) -> Self
    where
        P: Fn() -> PropertyStackId + 'static;
    /// Queues a callback that will be called with a [`WidgetMut`] for this widget.
    ///
    /// The callbacks will be run in the order they were submitted during the mutate pass.
    ///
    /// You might never use this thing, _since [`use_reactive_widget_mut`](Self::use_reactive_widget_mut) is what you use most of the time_
    /// but who knows?
    ///
    /// PS: *your `mutate_fn` will not run inside the current context!!*.
    fn mutate_later<Fn>(self, mutate_fn: Fn) -> Self
    where
        Fn: FnOnce(WidgetMut<'_, W>) + Send + 'static;
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
    fn on_action_ref_self<F>(&self, fun: F)
    where
        F: Fn(&<W as Widget>::Action) + Send + 'static,
    {
        register_typed_widget_action_handler::<W, _>(self.id(), fun);
    }
    fn on_action<F>(self, fun: F) -> Self
    where
        F: Fn(&<W as Widget>::Action) + Send + 'static,
    {
        self.on_action_ref_self(fun);
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

    fn class<C>(self, class: C) -> Self
    where
        C: Fn() -> String + 'static,
    {
        self.class_opt(move || Some(class()))
    }

    fn class_opt<C>(self, class: C) -> Self
    where
        C: Fn() -> Option<String> + 'static,
    {
        self.use_reactive_widget_mut_with_effect_val::<_, String>(move |mut widget, old_class| {
            if let Some(old_class) = old_class {
                widget.ctx.remove_class(&old_class);
            }
            let new_value = class();
            if let Some(new_class) = new_value.as_ref() {
                widget.ctx.add_class(new_class);
            }
            new_value
        })
    }

    fn classes<C>(self, classes: C) -> Self
    where
        C: Fn() -> Vec<String> + 'static,
    {
        self.use_reactive_widget_mut_with_effect_val::<_, Vec<String>>(
            move |mut widget, old_classes| {
                if let Some(old_classes) = old_classes {
                    for old_class in old_classes {
                        widget.ctx.remove_class(&old_class);
                    }
                }
                let new_value = classes();
                if new_value.is_empty() {
                    None
                } else {
                    for new_class in &new_value {
                        widget.ctx.add_class(new_class);
                    }
                    Some(new_value)
                }
            },
        )
    }

    fn disabled<D>(self, disabled: D) -> Self
    where
        D: Fn() -> bool + 'static,
    {
        self.disabled_with_current(move |_| disabled())
    }

    fn transform<T>(self, transform: T) -> Self
    where
        T: Fn() -> Affine + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            this.ctx.set_transform(transform());
        })
    }

    fn property_stack_id<P>(self, property_stack_id: P) -> Self
    where
        P: Fn() -> PropertyStackId + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            this.ctx.set_property_stack(property_stack_id());
        })
    }
    fn mutate_later<Fn>(self, mutate_fn: Fn) -> Self
    where
        Fn: FnOnce(WidgetMut<'_, W>) + Send + 'static,
    {
        if let Err(err) = self.create_velona_ref().mutate_later(mutate_fn) {
            log::error!("{err}");
        }
        self
    }

    fn disabled_with_current<D>(self, disabled: D) -> Self
    where
        D: Fn(bool) -> bool + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            let disabled = disabled(this.ctx.is_disabled());
            this.ctx.set_disabled(disabled);
        })
    }
}

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

#[cfg(feature = "masonry_widget_impls")]
mod single_impl {
    use super::{NewWidgetExt, SingleChildWidget};
    use masonry::widgets::*;
    use masonry_core::core::{NewWidget, Widget, WidgetMut};
    #[cfg(doc)]
    use reactive_graph::effect::Effect;

    macro_rules! impl_single_widget {
        ($($widget:ty,)*) => {
            $(
                #[cfg_attr(docsrs, doc(feature = "masonry_child_widget_impls"))]
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

    #[cfg_attr(docsrs, doc(feature = "masonry_child_widget_impls"))]
    impl SingleChildWidget for NewWidget<SizedBox> {
        /// It worth noting that the `use_child_fn` might not re-run properly
        /// if there are no child inside the [`SizedBox`].
        ///
        /// It is recommended to use `velona::NewSizedBoxExt::use_child_opt`, instead of this.
        fn use_child_erased<C>(self, mut use_child_fn: C) -> Self
        where
            C: FnMut(masonry::core::WidgetMut<'_, dyn Widget>) + 'static,
        {
            self.use_reactive_widget_mut(move |mut this| {
                let maybe_child = SizedBox::child_mut(&mut this);
                if let Some(child) = maybe_child {
                    // This will fail to re-run hardly if there are no child inside.
                    use_child_fn(child);
                } else {
                    log::warn!("Not child for SizedBox");
                }
            })
        }
    }
}

/// Similar to [`SingleChildWidget`] but the child is typed instead of erased.
// TODO implement for [`Portal`](masonry::widgets::Portal)
pub trait TypedSingleChildWidget {
    type Child: Widget + 'static;
    fn use_child<C>(self, use_child_fn: C) -> Self
    where
        C: FnMut(WidgetMut<'_, Self::Child>) + 'static;
}

#[cfg(feature = "masonry_widget_impls")]
mod typed_single_child_widget_impl {
    use masonry::{
        core::{NewWidget, Widget, WidgetMut},
        widgets::*,
    };

    use crate::{NewWidgetExt, widgets::TypedSingleChildWidget};

    #[cfg_attr(docsrs, doc(feature = "masonry_child_widget_impls"))]
    impl TypedSingleChildWidget for NewWidget<Selector> {
        type Child = Label;

        fn use_child<C>(self, mut use_child_fn: C) -> Self
        where
            C: FnMut(masonry::core::WidgetMut<'_, Self::Child>) + 'static,
        {
            self.use_reactive_widget_mut(move |mut this| {
                use_child_fn(Selector::child_mut(&mut this));
            })
        }
    }

    #[cfg_attr(docsrs, doc(feature = "masonry_child_widget_impls"))]
    impl TypedSingleChildWidget for NewWidget<SelectorItem> {
        type Child = Label;

        fn use_child<C>(self, mut use_child_fn: C) -> Self
        where
            C: FnMut(masonry::core::WidgetMut<'_, Self::Child>) + 'static,
        {
            self.use_reactive_widget_mut(move |mut this| {
                use_child_fn(SelectorItem::child_mut(&mut this));
            })
        }
    }

    #[cfg_attr(docsrs, doc(feature = "masonry_child_widget_impls"))]
    impl<W> TypedSingleChildWidget for NewWidget<Portal<W>>
    where
        W: Widget + 'static,
    {
        type Child = W;

        fn use_child<C>(self, mut use_child_fn: C) -> Self
        where
            C: FnMut(WidgetMut<'_, Self::Child>) + 'static,
        {
            self.use_reactive_widget_mut(move |mut this| use_child_fn(Portal::child_mut(&mut this)))
        }
    }
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

#[cfg(feature = "masonry_widget_impls")]
mod reactive_child_impl {
    use super::{NewWidgetExt, ReactiveSingleChildExt};
    use crate::{AnyNewWidget, utils::ConsumeResult};
    use masonry::widgets::*;
    use masonry_core::core::NewWidget;
    use reactive_graph::effect::Effect;
    use std::any::type_name;

    macro_rules! impl_reactive_child {
        ($($widget:ty,)*) => {
            $(
                #[cfg_attr(docsrs, doc(feature = "masonry_widget_impls"))]
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
    impl ReactiveSingleChildExt for NewWidget<SizedBox> {
        fn child<Cf>(self, child_fn: Cf) -> Self
        where
            Cf: Fn() -> AnyNewWidget + 'static,
        {
            let velona_ref = self.create_velona_ref();
            Effect::new(move || {
                let child = child_fn();
                velona_ref
                    .edit_local_now(|mut this| {
                        SizedBox::set_child(&mut this, child);
                    })
                    .consume_with_log_err();
            });
            self
        }
    }
}

/// Allows you to [`Widget`] `set_child` reactively.
///
/// Unlike [`ReactiveSingleChildExt`], this trait is only implemented for [`Widget`]s that has a **typed** `set_child`.
pub trait ReactiveSingleTypedChildExt {
    type Child: Widget + 'static;
    fn child<Cf>(self, child_fn: Cf) -> Self
    where
        Cf: Fn() -> NewWidget<Self::Child> + 'static;
}

#[cfg(feature = "masonry_widget_impls")]
mod reactive_typed_single_child_ext {
    use masonry::{
        core::{NewWidget, Widget},
        widgets::Portal,
    };
    use reactive_graph::effect::Effect;

    use crate::{NewWidgetExt, utils::ConsumeResult, widgets::ReactiveSingleTypedChildExt};
    #[cfg_attr(docsrs, doc(feature = "masonry_widget_impls"))]
    impl<W> ReactiveSingleTypedChildExt for NewWidget<Portal<W>>
    where
        W: Widget + 'static,
    {
        type Child = W;
        fn child<Cf>(self, child_fn: Cf) -> Self
        where
            Cf: Fn() -> NewWidget<Self::Child> + 'static,
        {
            let w_ref = self.create_velona_ref();
            Effect::new(move || {
                let new_child = child_fn();
                w_ref
                    .edit_local_now(|mut this| {
                        Portal::set_child(&mut this, new_child);
                    })
                    .consume_with_log_err();
            });
            self
        }
    }
}
