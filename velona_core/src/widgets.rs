use std::marker::PhantomData;

use masonry_core::core::ErasedAction;
use masonry_core::core::FromDynWidget;
#[cfg(doc)]
use masonry_core::core::MutateCtx;
use masonry_core::core::WidgetRef;
use masonry_core::{
    core::{NewWidget, Property, PropertyStackId, UsesProperty as HasProperty, Widget, WidgetMut},
    kurbo::Affine,
};
use reactive_graph::effect::Effect;

use reactive_graph::owner::ArenaItem;
use send_wrapper::SendWrapper;

use crate::utils::ConsumeResult;
use crate::{
    utils::register_widget_action_listener,
    widget_ref::VelonaWidgetRef,
    window::{event_listener::register_typed_widget_action_listener, use_window},
};

pub trait View {
    type Widget: Widget + FromDynWidget + ?Sized;

    fn into_new_widget(self) -> NewWidget<Self::Widget>;

    fn into_erased(self) -> NewWidget<dyn Widget>;
}

trait IsNewWidget {}

impl<W> IsNewWidget for NewWidget<W> where W: ?Sized {}

pub struct UseWidgetValResult<A, B = ()> {
    pub to_edit_fn: A,
    pub to_next_effect_run: Option<B>,
}

impl<A> UseWidgetValResult<A, ()> {
    pub const fn to_edit_fn(value: A) -> Self {
        Self {
            to_edit_fn: value,
            to_next_effect_run: None,
        }
    }
}

#[allow(private_bounds)]
pub trait NewWidgetExt: View + IsNewWidget {
    // TODO add docs on how it works
    fn use_widget_mut_val<Vfn, Efn, V, O>(
        self,
        val_fn: Vfn,
        edit_fn: Efn,
    ) -> NewWidget<Self::Widget>
    where
        Efn: FnMut(WidgetMut<'_, Self::Widget>, V) + 'static,
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static;
    // TODO add docs
    fn use_widget_mut<Vfn, Efn, V>(self, val_fn: Vfn, edit_fn: Efn) -> NewWidget<Self::Widget>
    where
        Efn: FnMut(WidgetMut<'_, Self::Widget>, V) + 'static,
        V: 'static,
        Vfn: Fn() -> V + 'static;

    fn use_widget_ref_val<Vfn, Efn, V, O>(
        self,
        val_fn: Vfn,
        use_fn: Efn,
    ) -> NewWidget<Self::Widget>
    where
        Efn: FnMut(WidgetRef<'_, Self::Widget>, V) + 'static,
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static;
    // TODO add docs
    fn use_widget_ref<Vfn, Efn, V>(self, val_fn: Vfn, use_fn: Efn) -> NewWidget<Self::Widget>
    where
        Efn: FnMut(WidgetRef<'_, Self::Widget>, V) + 'static,
        V: 'static,
        Vfn: Fn() -> V + 'static;

    /// Very similar to [`on`](Self::on_action) but uses a [`&self`](self) instead of [`self`].
    /// _You get the idea._
    fn on_action_ref_self<F>(&self, fun: F)
    where
        F: Fn(&<Self::Widget as Widget>::Action) + Send + 'static,
        Self::Widget: Sized;
    /// Listen to the [`Widget::Action`]
    fn on_action<F>(self, fun: F) -> NewWidget<Self::Widget>
    where
        F: Fn(&<Self::Widget as Widget>::Action) + Send + 'static,
        Self::Widget: Sized;
    fn with_props_opt_reactive<F, P>(self, prop: F) -> NewWidget<Self::Widget>
    where
        F: Fn() -> Option<P> + 'static,
        P: Property;
    /// Set a [widget](Widget) [property](Property) reactively.
    fn with_props_reactive<F, P>(self, prop: F) -> NewWidget<Self::Widget>
    where
        F: Fn() -> P + 'static,
        P: Property,
        Self::Widget: HasProperty<P> + 'static;
    /// Update the internal [`NewWidget::widget`].
    // **NOTE: Please be smart and always use [`untrack`](reactive_graph::graph::untrack) if you use decide to bring a reactive closure on using this.**
    // Weird thing might happen if you do that.
    fn update_inner_widget<T>(self, update_fn: T) -> NewWidget<Self::Widget>
    where
        T: FnOnce(Self::Widget) -> Self::Widget,
        Self::Widget: Sized;
    /// Create a [`WidgetRef`](VelonaWidgetRef) that you can send safely between thread.
    fn create_velona_ref(&self) -> VelonaWidgetRef<Self::Widget>;
    /// Queues a callback that will be called with a [`WidgetMut`] for this widget.
    ///
    /// The callbacks will be run in the order they were submitted during the mutate pass.
    ///
    /// You might never use this thing, _since [`use_reactive_widget_mut`](Self::use_reactive_widget_mut) is what you use most of the time_
    /// but who knows?
    ///
    /// PS: *your `mutate_fn` will not run inside the current context!!*.
    fn mutate_later<Fn>(self, mutate_fn: Fn) -> NewWidget<Self::Widget>
    where
        Fn: FnOnce(WidgetMut<'_, Self::Widget>) + Send + 'static;
    /// Very similar to [`on`](Self::on_erased_action) but uses a [`&self`](self) instead of [`self`].
    /// _You get the idea._
    fn on_erased_action_ref_self<F>(&self, fun: F)
    where
        F: Fn(&ErasedAction) + Send + 'static;
    /// Listen to the [`Widget::Action`] but it is [erased](ErasedAction).
    fn on_erased_action<F>(self, fun: F) -> Self
    where
        F: Fn(&ErasedAction) + Send + 'static;

    /// Set class the new widget class reactively.
    ///
    /// When the value changes, the old one will be [removed](MutateCtx::remove_class).
    ///
    /// See [`MutateCtx::add_class`] and [`MutateCtx::remove_class`].
    fn class<C>(self, class: C) -> Self
    where
        C: Fn() -> String + 'static;
    /// Similar to [`class`](Self::class) but uses a [`Option<String>`] instead of [`String`].
    ///
    /// See [`MutateCtx::add_class`] and [`MutateCtx::remove_class`].
    fn class_opt<C>(self, class: C) -> Self
    where
        C: Fn() -> Option<String> + 'static;
    /// Similar to [`class`](Self::class) and [`class_opt`](Self::class_opt) but uses a [`Vec<String>`] (aka a list of classes).
    ///
    /// When the values changes, the old classes with be [removed](MutateCtx::remove_class).
    ///
    /// See [`MutateCtx::add_class`] and [`MutateCtx::remove_class`].
    fn classes<C>(self, classes: C) -> Self
    where
        C: Fn() -> Box<[String]> + 'static;
    /// Sets the disabled state for this widget.
    ///
    /// Setting this to `false` does not mean a widget is not still disabled;
    /// for instance it may still be disabled by an ancestor.
    /// See [`MutateCtx::is_disabled`] for more information.
    ///
    /// _Reactive version of [`MutateCtx::set_disabled`]_.
    fn disabled_reactive<D>(self, disabled: D) -> Self
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
}

impl<W> View for NewWidget<W>
where
    W: Widget + FromDynWidget + ?Sized,
{
    type Widget = W;
    fn into_new_widget(self) -> NewWidget<Self::Widget> {
        self
    }
    fn into_erased(self) -> NewWidget<dyn Widget> {
        self.erased()
    }
}

impl<W> NewWidgetExt for NewWidget<W>
where
    W: Widget + FromDynWidget + ?Sized,
{
    #[track_caller]
    fn use_widget_mut_val<Vfn, Efn, V, O>(
        self,
        val_fn: Vfn,
        edit_fn: Efn,
    ) -> NewWidget<Self::Widget>
    where
        Efn: FnMut(WidgetMut<'_, Self::Widget>, V) + 'static,
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static,
    {
        let widget_ref = self.create_velona_ref();
        let edit_fn = ArenaItem::new_local(edit_fn);
        Effect::new(move |effect_param: Option<Option<O>>| {
            let res = val_fn(effect_param.flatten());

            let to_send = SendWrapper::new(res.to_edit_fn);

            widget_ref
                .edit(move |widget_mut| {
                    edit_fn.try_update_value(|efn| {
                        efn(widget_mut, to_send.take());
                    });
                })
                .consume_with_log_err();
            res.to_next_effect_run
        });
        self
    }

    #[track_caller]
    fn use_widget_mut<Vfn, Efn, V>(self, val_fn: Vfn, edit_fn: Efn) -> NewWidget<Self::Widget>
    where
        Efn: FnMut(WidgetMut<'_, Self::Widget>, V) + 'static,
        V: 'static,
        Vfn: Fn() -> V + 'static,
    {
        self.use_widget_mut_val(move |_| UseWidgetValResult::to_edit_fn(val_fn()), edit_fn)
    }

    #[track_caller]
    fn on_action_ref_self<F>(&self, fun: F)
    where
        F: Fn(&<Self::Widget as Widget>::Action) + Send + 'static,
        Self::Widget: Sized,
    {
        register_typed_widget_action_listener::<Self::Widget, F>(self.id(), fun);
    }

    #[track_caller]
    fn on_action<F>(self, fun: F) -> NewWidget<Self::Widget>
    where
        F: Fn(&<Self::Widget as Widget>::Action) + Send + 'static,
        Self::Widget: Sized,
    {
        register_typed_widget_action_listener::<Self::Widget, F>(self.id(), fun);
        self
    }

    #[track_caller]
    fn with_props_opt_reactive<F, P>(self, prop: F) -> NewWidget<Self::Widget>
    where
        F: Fn() -> Option<P> + 'static,
        P: Property,
    {
        self.use_widget_mut(prop, move |mut widget_mut, prop| {
            if let Some(prop) = prop {
                widget_mut.insert_prop(prop);
            } else {
                widget_mut.remove_prop::<P>();
            }
        })
    }

    #[track_caller]
    fn with_props_reactive<F, P>(self, prop: F) -> NewWidget<Self::Widget>
    where
        F: Fn() -> P + 'static,
        P: Property,
        Self::Widget: HasProperty<P> + 'static,
    {
        self.with_props_opt_reactive(move || Some(prop()))
    }

    #[track_caller]
    fn update_inner_widget<T>(mut self, update_fn: T) -> NewWidget<Self::Widget>
    where
        T: FnOnce(Self::Widget) -> Self::Widget,
        Self::Widget: Sized,
    {
        *self.widget = update_fn(*self.widget);
        self
    }

    fn create_velona_ref(&self) -> VelonaWidgetRef<Self::Widget> {
        VelonaWidgetRef {
            id: self.id(),
            window: use_window().map(Box::new),
            phantom: PhantomData::<Self::Widget>,
        }
    }

    #[track_caller]
    fn mutate_later<Fn>(self, mutate_fn: Fn) -> NewWidget<Self::Widget>
    where
        Fn: FnOnce(WidgetMut<'_, Self::Widget>) + Send + 'static,
    {
        self.create_velona_ref()
            .mutate_later(mutate_fn)
            .consume_with_log_err();
        self
    }

    #[track_caller]
    fn on_erased_action_ref_self<F>(&self, fun: F)
    where
        F: Fn(&ErasedAction) + Send + 'static,
    {
        register_widget_action_listener(self.id(), Box::new(fun));
    }

    #[track_caller]
    fn on_erased_action<F>(self, fun: F) -> Self
    where
        F: Fn(&ErasedAction) + Send + 'static,
    {
        register_widget_action_listener(self.id(), Box::new(fun));
        self
    }

    #[track_caller]
    fn class<C>(self, class: C) -> Self
    where
        C: Fn() -> String + 'static,
    {
        self.class_opt(move || Some(class()))
    }

    #[track_caller]
    fn class_opt<C>(self, class: C) -> Self
    where
        C: Fn() -> Option<String> + 'static,
    {
        self.use_widget_mut_val(
            move |old_class_maybe: Option<String>| {
                let mut instructions = Vec::<(String, ClassActionType)>::with_capacity(2);
                if let Some(old_class) = old_class_maybe {
                    instructions.push((old_class, ClassActionType::Remove));
                }
                let maybe_new_class = class();
                if let Some(new_class) = maybe_new_class.as_ref() {
                    instructions.push((new_class.clone(), ClassActionType::Add));
                }
                UseWidgetValResult {
                    to_edit_fn: instructions.into_boxed_slice(),
                    to_next_effect_run: maybe_new_class,
                }
            },
            exec_class_actions,
        )
    }

    #[track_caller]
    fn classes<C>(self, classes: C) -> Self
    where
        C: Fn() -> Box<[String]> + 'static,
    {
        self.use_widget_mut_val(
            move |old_classes_maybe: Option<Box<[String]>>| {
                let mut instructions = Vec::<(String, ClassActionType)>::new();
                if let Some(old_classes) = old_classes_maybe {
                    for old_class in old_classes {
                        instructions.push((old_class, ClassActionType::Remove));
                    }
                }
                let new_classes = classes();
                for new_class in &new_classes {
                    instructions.push((new_class.clone(), ClassActionType::Add));
                }
                UseWidgetValResult {
                    to_edit_fn: instructions.into_boxed_slice(),
                    to_next_effect_run: Some(new_classes),
                }
            },
            exec_class_actions,
        )
    }

    #[track_caller]
    fn disabled_reactive<D>(self, disabled: D) -> Self
    where
        D: Fn() -> bool + 'static,
    {
        self.disabled_with_current(move |_| disabled())
    }

    #[track_caller]
    fn disabled_with_current<D>(self, disabled: D) -> Self
    where
        D: Fn(bool) -> bool + 'static,
    {
        self.use_widget_mut_val(
            move |old_state: Option<bool>| {
                let new_state = disabled(old_state.unwrap_or_default());
                UseWidgetValResult {
                    to_edit_fn: new_state,
                    to_next_effect_run: Some(new_state),
                }
            },
            |mut widget_mut, disabled| {
                widget_mut.ctx.set_disabled(disabled);
            },
        )
    }

    #[track_caller]
    fn transform<T>(self, transform: T) -> Self
    where
        T: Fn() -> Affine + 'static,
    {
        self.use_widget_mut(transform, |mut widget_mut, transform| {
            widget_mut.ctx.set_transform(transform);
        })
    }

    #[track_caller]
    fn property_stack_id<P>(self, property_stack_id: P) -> Self
    where
        P: Fn() -> PropertyStackId + 'static,
    {
        self.use_widget_mut(property_stack_id, |mut widget_mut, stack_id| {
            widget_mut.ctx.set_property_stack(stack_id);
        })
    }

    fn use_widget_ref_val<Vfn, Efn, V, O>(self, val_fn: Vfn, use_fn: Efn) -> NewWidget<Self::Widget>
    where
        Efn: FnMut(WidgetRef<'_, Self::Widget>, V) + 'static,
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static,
    {
        let widget_ref = self.create_velona_ref();
        let use_fn = ArenaItem::new_local(use_fn);
        Effect::new(move |effect_param: Option<Option<O>>| {
            let res = val_fn(effect_param.flatten());

            let to_send = SendWrapper::new(res.to_edit_fn);

            widget_ref
                .use_widget(move |widget_ref| {
                    use_fn.try_update_value(|efn| {
                        efn(widget_ref, to_send.take());
                    });
                })
                .consume_with_log_err();
            res.to_next_effect_run
        });
        self
    }

    fn use_widget_ref<Vfn, Efn, V>(self, val_fn: Vfn, use_fn: Efn) -> NewWidget<Self::Widget>
    where
        Efn: FnMut(WidgetRef<'_, Self::Widget>, V) + 'static,
        V: 'static,
        Vfn: Fn() -> V + 'static,
    {
        self.use_widget_ref_val(move |_| UseWidgetValResult::to_edit_fn(val_fn()), use_fn)
    }
}

enum ClassActionType {
    Add,
    Remove,
}

fn exec_class_actions<W>(
    mut widget_mut: WidgetMut<'_, W>,
    actions: Box<[(String, ClassActionType)]>,
) where
    W: Widget + FromDynWidget + ?Sized,
{
    for (class, action_type) in actions {
        match action_type {
            ClassActionType::Add => {
                widget_mut.ctx.add_class(&class);
            }
            ClassActionType::Remove => {
                widget_mut.ctx.remove_class(&class);
            }
        }
    }
}
