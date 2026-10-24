use std::sync::{Arc, RwLock};

use futures_util::future::{AbortHandle, Abortable};
use masonry_raw_box::RawBox;
use reactive_graph::{
    computed::ScopedFuture,
    owner::on_cleanup,
    signal::Trigger,
    traits::{Notify, Track},
    transition::AsyncTransition,
};
use velona_core::{NewWidgetExt, masonry_core::core::Widget, task::spawn_local, widgets::View};

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
            let transition = AsyncTransition::run(|| comp());

            let (abort_handle, abort_registration) = AbortHandle::new_pair();
            on_cleanup(move || abort_handle.abort());

            let task = Abortable::new(
                async move {
                    let view = transition.await;
                    value.write().unwrap().replace(view);
                    notify.notify();
                },
                abort_registration,
            );

            let task = ScopedFuture::new_untracked(task);
            spawn_local(async move {
                let _ = task.await;
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
