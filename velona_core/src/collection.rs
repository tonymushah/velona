use masonry_core::core::{CollectionWidget, NewWidget};

use crate::{
    NewWidgetExt,
    utils::{ConsumeResult, local_effect},
};

pub type CollectIterItem<P> = (crate::AnyNewWidget, P);

pub trait NewCollectionWidgetExt<P> {
    fn collect_reactive_iter<I, Ifn>(self, iter_fn: Ifn) -> Self
    where
        I: IntoIterator<Item = CollectIterItem<P>>,
        Ifn: Fn() -> I + 'static;
}

impl<W, P> NewCollectionWidgetExt<P> for NewWidget<W>
where
    W: CollectionWidget<P> + 'static,
    P: 'static,
{
    fn collect_reactive_iter<I, Ifn>(self, iter_fn: Ifn) -> Self
    where
        I: IntoIterator<Item = CollectIterItem<P>>,
        Ifn: Fn() -> I + 'static,
    {
        let self_ref = self.create_velona_ref();
        local_effect(move || {
            self_ref
                .edit_local_now(|mut this| {
                    CollectionWidget::<P>::clear(&mut this);
                })
                .consume_with_log_err();
            let elements = iter_fn();
            self_ref
                .edit_local_now(|mut this| {
                    for (child, param) in elements {
                        CollectionWidget::<P>::add(&mut this, child, param);
                    }
                })
                .consume_with_log_err();
        });
        self
    }
}
