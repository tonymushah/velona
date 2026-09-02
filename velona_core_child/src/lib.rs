use std::any::type_name;

use masonry::core::FromDynWidget;
use velona_core::{
    AnyNewWidget,
    masonry_core::core::{NewWidget, Widget, WidgetMut},
};

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
    use super::SingleChildWidget;
    use masonry::widgets::*;
    use masonry_core::core::{NewWidget, Widget, WidgetMut};
    #[cfg(doc)]
    use velona_core::reactive::effect::Effect;
    use velona_core::{masonry_core, widgets::NewWidgetExt};

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
    type Child: Widget + FromDynWidget + ?Sized;
    fn use_child<C>(self, use_child_fn: C) -> Self
    where
        C: FnMut(WidgetMut<'_, Self::Child>) + 'static;
}

#[cfg(feature = "masonry_widget_impls")]
mod typed_single_child_widget_impl {
    use masonry::{
        core::{FromDynWidget, NewWidget, Widget, WidgetMut},
        widgets::*,
    };
    use velona_core::NewWidgetExt;

    use crate::TypedSingleChildWidget;

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
        W: Widget + FromDynWidget + ?Sized,
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
    use super::ReactiveSingleChildExt;
    use masonry::widgets::*;
    use masonry_core::core::NewWidget;
    use std::any::type_name;
    use velona_core::{
        AnyNewWidget, NewWidgetExt, masonry_core, reactive::effect::Effect, utils::ConsumeResult,
    };

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
    use velona_core::{NewWidgetExt, reactive::effect::Effect, utils::ConsumeResult};

    use crate::ReactiveSingleTypedChildExt;

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
