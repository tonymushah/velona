use masonry_core::core::{CollectionWidget, NewWidget};
use reactive_graph::owner::ArenaItem;

use crate::{NewWidgetExt, reactive::effect::Effect, utils::ConsumeResult};

pub type CollectIterItem<P> = (crate::AnyNewWidget, P);

pub trait NewCollectionWidgetExt<P> {
    fn collect_reactive_iter<I, Ifn>(self, iter_fn: Ifn) -> Self
    where
        I: IntoIterator<Item = CollectIterItem<P>> + 'static,
        Ifn: Fn() -> I + 'static;
}

impl<W, P> NewCollectionWidgetExt<P> for NewWidget<W>
where
    W: CollectionWidget<P> + 'static,
    P: 'static,
{
    #[track_caller]
    fn collect_reactive_iter<I, Ifn>(self, iter_fn: Ifn) -> Self
    where
        I: IntoIterator<Item = CollectIterItem<P>> + 'static,
        Ifn: Fn() -> I + 'static,
    {
        let self_ref = self.create_velona_ref();
        Effect::new(move || {
            let elements = ArenaItem::new_local(Some(Box::new(iter_fn())));

            self_ref
                .edit(move |mut this| {
                    let Some(elements) = elements.try_update_value(|inner| inner.take()).flatten()
                    else {
                        return;
                    };
                    CollectionWidget::<P>::clear(&mut this);
                    for (child, param) in *elements {
                        CollectionWidget::<P>::add(&mut this, child, param);
                    }
                })
                .consume_with_log_err();
        });
        self
    }
}
