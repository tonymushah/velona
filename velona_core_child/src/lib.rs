use std::any::type_name;

use velona_core::masonry_core::core::FromDynWidget;
use velona_core::widgets::UseWidgetValResult;
use velona_core::{
    AnyNewWidget,
    masonry_core::core::{NewWidget, Widget, WidgetMut},
};

/// Some widget has a single child with them. (like [button](masonry::widgets::Button), [align](masonry::widgets::Align))
///
/// This trait will unify all of those single child widgets "mutations" (aka `child_mut`) _instead of making duplicates method for those_.
#[must_use]
pub trait SingleChildWidget {
    #[track_caller]
    fn use_child_erased<Vfn, Cfn, V, O>(self, val_fn: Vfn, edit_child_fn: Cfn) -> Self
    where
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static,
        Cfn: FnMut(WidgetMut<'_, dyn Widget>, V) + 'static;
    #[track_caller]
    fn use_child_casted<Vfn, Cfn, V, O, W>(self, val_fn: Vfn, mut edit_child_fn: Cfn) -> Self
    where
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static,
        Cfn: FnMut(WidgetMut<'_, W>, V) + 'static,
        W: Widget + 'static,
        Self: Sized,
    {
        self.use_child_erased(val_fn, move |mut child, val| {
            if let Some(child) = child.try_downcast::<W>() {
                edit_child_fn(child, val);
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
    use velona_core::{
        masonry_core,
        widgets::{NewWidgetExt, UseWidgetValResult},
    };

    macro_rules! impl_single_widget {
        ($($widget:ty,)*) => {
            $(
                #[cfg_attr(docsrs, doc(feature = "masonry_child_widget_impls"))]
                impl SingleChildWidget for NewWidget<$widget> {
                    /// It is worth mentioning that the `use_child_fn` will run inside an [`Effect`].
                    fn use_child_erased<Vfn, Cfn, V, O>(self, val_fn: Vfn, mut edit_child_fn: Cfn) -> Self
                    where
                        V: 'static,
                        O: 'static,
                        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static,
                        Cfn: FnMut(WidgetMut<'_, dyn Widget>, V) + 'static
                    {
                        self.use_widget_mut_val(val_fn, move|mut this, val| {
                            let child = <$widget>::child_mut(&mut this);
                            edit_child_fn(child, val);
                        })
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
        fn use_child_erased<Vfn, Cfn, V, O>(self, val_fn: Vfn, mut edit_child_fn: Cfn) -> Self
        where
            V: 'static,
            O: 'static,
            Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static,
            Cfn: FnMut(WidgetMut<'_, dyn Widget>, V) + 'static,
        {
            self.use_widget_mut_val(val_fn, move |mut this, val| {
                if let Some(child) = SizedBox::child_mut(&mut this) {
                    edit_child_fn(child, val);
                }
            })
        }
    }
}

/// Similar to [`SingleChildWidget`] but the child is typed instead of erased.
// TODO implement for [`Portal`](masonry::widgets::Portal)
#[must_use]
pub trait TypedSingleChildWidget {
    type Child: Widget + FromDynWidget + ?Sized;

    fn use_child<Vfn, Cfn, V, O>(self, val_fn: Vfn, edit_child_fn: Cfn) -> Self
    where
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static,
        Cfn: FnMut(WidgetMut<'_, Self::Child>, V) + 'static;
}

#[cfg(feature = "masonry_widget_impls")]
mod typed_single_child_widget_impl {
    use masonry::{
        core::{FromDynWidget, NewWidget, Widget, WidgetMut},
        widgets::*,
    };
    use velona_core::{NewWidgetExt, widgets::UseWidgetValResult};

    use crate::TypedSingleChildWidget;

    #[cfg_attr(docsrs, doc(feature = "masonry_child_widget_impls"))]
    impl TypedSingleChildWidget for NewWidget<Selector> {
        type Child = Label;

        fn use_child<Vfn, Cfn, V, O>(self, val_fn: Vfn, mut edit_child_fn: Cfn) -> Self
        where
            V: 'static,
            O: 'static,
            Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static,
            Cfn: FnMut(WidgetMut<'_, Self::Child>, V) + 'static,
        {
            self.use_widget_mut_val(val_fn, move |mut this, val| {
                edit_child_fn(Selector::child_mut(&mut this), val);
            })
        }
    }

    #[cfg_attr(docsrs, doc(feature = "masonry_child_widget_impls"))]
    impl TypedSingleChildWidget for NewWidget<SelectorItem> {
        type Child = Label;

        fn use_child<Vfn, Cfn, V, O>(self, val_fn: Vfn, mut edit_child_fn: Cfn) -> Self
        where
            V: 'static,
            O: 'static,
            Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static,
            Cfn: FnMut(WidgetMut<'_, Self::Child>, V) + 'static,
        {
            self.use_widget_mut_val(val_fn, move |mut this, val| {
                edit_child_fn(SelectorItem::child_mut(&mut this), val);
            })
        }
    }

    #[cfg_attr(docsrs, doc(feature = "masonry_child_widget_impls"))]
    impl<W> TypedSingleChildWidget for NewWidget<Portal<W>>
    where
        W: Widget + FromDynWidget + ?Sized,
    {
        type Child = W;

        fn use_child<Vfn, Cfn, V, O>(self, val_fn: Vfn, mut edit_child_fn: Cfn) -> Self
        where
            V: 'static,
            O: 'static,
            Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static,
            Cfn: FnMut(WidgetMut<'_, Self::Child>, V) + 'static,
        {
            self.use_widget_mut_val(val_fn, move |mut this, val| {
                edit_child_fn(Portal::child_mut(&mut this), val);
            })
        }
    }
}

impl<T> SingleChildWidget for T
where
    T: TypedSingleChildWidget,
{
    fn use_child_erased<Vfn, Cfn, V, O>(self, val_fn: Vfn, mut edit_child_fn: Cfn) -> Self
    where
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static,
        Cfn: FnMut(WidgetMut<'_, dyn Widget>, V) + 'static,
    {
        <Self as TypedSingleChildWidget>::use_child(self, val_fn, move |mut child, val| {
            edit_child_fn(child.downcast::<dyn Widget>(), val);
        })
    }
}

/// Allows you to [`Widget`] `set_child` reactively.
///
/// This is only implemented for [`Widget`]s that has an erashed `set_child`
#[must_use]
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
    use velona_core::{AnyNewWidget, NewWidgetExt, masonry_core};

    macro_rules! impl_reactive_child {
        ($($widget:ty,)*) => {
            $(
                #[cfg_attr(docsrs, doc(feature = "masonry_widget_impls"))]
                impl ReactiveSingleChildExt for NewWidget<$widget> {
                    fn child<Cf>(self, child_fn: Cf) -> Self
                    where
                        Cf: Fn() -> AnyNewWidget + 'static
                    {
                        self.use_widget_mut(child_fn, |mut this, new_widget| {
                            <$widget>::set_child(&mut this, new_widget);
                        })
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
            self.use_widget_mut(child_fn, |mut this, child| {
                SizedBox::set_child(&mut this, child);
            })
        }
    }
}

/// Allows you to [`Widget`] `set_child` reactively.
///
/// Unlike [`ReactiveSingleChildExt`], this trait is only implemented for [`Widget`]s that has a **typed** `set_child`.
#[must_use]
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
    use velona_core::NewWidgetExt;

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
            self.use_widget_mut(child_fn, |mut this, child| {
                Portal::set_child(&mut this, child);
            })
        }
    }
}
