use masonry::core::{CollectionWidget, NewWidget};
use reactive_graph::effect::Effect;

use crate::{NewWidgetExt, utils::ConsumeResult};

pub type CollectIterItem<P> = (crate::AnyNewWidget, P);

pub trait NewCollectionWidgetExt<P> {
    fn collect_reactive_iter<I, Ifn>(self, iter_fn: Ifn) -> Self
    where
        I: Iterator<Item = CollectIterItem<P>>,
        Ifn: Fn() -> I + 'static;
}

impl<W, P> NewCollectionWidgetExt<P> for NewWidget<W>
where
    W: CollectionWidget<P> + 'static,
    P: 'static,
{
    fn collect_reactive_iter<I, Ifn>(self, iter_fn: Ifn) -> Self
    where
        I: Iterator<Item = CollectIterItem<P>>,
        Ifn: Fn() -> I + 'static,
    {
        let self_ref = self.create_velona_ref();
        Effect::new(move || {
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
