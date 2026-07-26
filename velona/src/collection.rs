use masonry::core::{CollectionWidget, NewWidget};
use reactive_graph::effect::Effect;

use crate::{NewWidgetExt, utils::ConsumeResult};

pub trait NewCollectionWidgetExt<P> {
    fn collect_reactive_iter<I, Ifn, C>(self, iter_fn: Ifn, convert_fn: C) -> Self
    where
        I: Iterator + 'static,
        I::Item: 'static,
        Ifn: Fn() -> I + 'static,
        C: FnMut(I::Item) -> (crate::AnyNewWidget, P) + 'static;
}

impl<W, P> NewCollectionWidgetExt<P> for NewWidget<W>
where
    W: CollectionWidget<P> + 'static,
    P: 'static,
{
    fn collect_reactive_iter<I, Ifn, C>(self, iter_fn: Ifn, mut convert_fn: C) -> Self
    where
        I: Iterator + 'static,
        I::Item: 'static,
        Ifn: Fn() -> I + 'static,
        C: FnMut(I::Item) -> (crate::AnyNewWidget, P) + 'static,
    {
        let self_ref = self.create_velona_ref();
        Effect::new(move || {
            self_ref
                .edit_local_now(|mut this| {
                    CollectionWidget::<P>::clear(&mut this);
                })
                .consume_with_log_err();
            let iter = iter_fn().map(&mut convert_fn);
            self_ref
                .edit_local_now(|mut this| {
                    for (child, params) in iter {
                        CollectionWidget::<P>::add(&mut this, child, params);
                    }
                })
                .consume_with_log_err();
        });
        self
    }
}
