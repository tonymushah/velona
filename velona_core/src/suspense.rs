use std::sync::{Arc, RwLock};

use crate::task::tick;

use super::{NewWidgetExt, masonry_core::core::Widget, task::spawn_local, widgets::View};
use futures_util::future::{AbortHandle, Abortable};
use masonry_raw_box::RawBox;
use reactive_graph::{
    computed::ScopedFuture,
    owner::{on_cleanup, provide_context, use_context},
    signal::{ReadSignal, Trigger, signal},
    traits::{Notify, Track},
    transition::AsyncTransition,
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct SuspenseContext {
    pub is_loaded: ReadSignal<bool>,
    _n: (),
}

#[derive(Debug)]
pub struct Suspsense<F: View> {
    pub fallback: F,
}

impl Default for Suspsense<()> {
    fn default() -> Self {
        Self { fallback: () }
    }
}

impl<F> Suspsense<F>
where
    F: View + 'static,
{
    pub fn from_async_fn<C, V>(self, comp: C) -> impl View
    where
        C: AsyncFnOnce() -> V + 'static,
        V: View + 'static,
    {
        let (is_loaded, set_loaded) = signal(false);

        let notify = Trigger::new();
        let value = Arc::new(RwLock::new(None::<Box<V>>));
        {
            let value = value.clone();
            let transition = AsyncTransition::run(move || {
                provide_context(SuspenseContext { is_loaded, _n: () });
                comp()
            });

            let (abort_handle, abort_registration) = AbortHandle::new_pair();
            on_cleanup(move || abort_handle.abort());

            let task = Abortable::new(
                async move {
                    let view = transition.await;
                    value.write().unwrap().replace(Box::new(view));
                    notify.notify();
                },
                abort_registration,
            );

            let task = ScopedFuture::new_untracked(task);
            spawn_local(async move {
                let _ = task.await;
                tick().await;
                set_loaded(true);
            });
        }

        debug_assert!(
            use_context::<SuspenseContext>().is_none(),
            "There should be no `SuspenseContext` available"
        );

        RawBox::new(self.fallback.into_new_widget())
            .prepare()
            .use_widget_mut(
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
    pub fn from_sync_fn<C, V>(self, comp: C) -> impl View
    where
        C: FnOnce() -> V + 'static,
        V: View + 'static,
    {
        Suspsense::from_async_fn(self, async move || comp())
    }
}
