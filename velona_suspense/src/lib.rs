use std::sync::{Arc, RwLock};

use masonry_raw_box::RawBox;
use reactive_graph::{
    signal::Trigger,
    spawn_local_scoped_with_cancellation,
    traits::{Notify, Track},
    transition::AsyncTransition,
};
use velona_core::{NewWidgetExt, masonry_core::core::Widget, widgets::View};

#[derive(Debug)]
pub struct Suspsense;

impl Suspsense {
    pub fn from_async_fn<C, V>(comp: C) -> impl View
    where
        C: AsyncFnOnce() -> V + 'static,
        V: View + 'static,
    {
        let notify = Trigger::new();
        let value = Arc::new(RwLock::new(None::<V>));
        {
            let value = value.clone();
            spawn_local_scoped_with_cancellation(async move {
                let view = AsyncTransition::run(|| comp()).await;
                value.write().unwrap().replace(view);
                notify.notify();
            });
        }
        RawBox::empty().prepare().use_widget_mut(
            move || {
                notify.track();
                value
                    .write()
                    .ok()
                    .and_then(|mut d| d.take())
                    .map(|v| v.into_new_widget())
            },
            |mut this, maybe_view| {
                if let Some(new) = maybe_view {
                    RawBox::set_child(&mut this, new);
                } else {
                    RawBox::remove_child(&mut this);
                }
            },
        )
    }
    pub fn from_sync_fn<C, V>(comp: C) -> impl View
    where
        C: FnOnce() -> V + 'static,
        V: View + 'static,
    {
        Suspsense::from_async_fn(async move || comp())
    }
}
