use std::{ops::Deref, sync::mpsc};

use any_spawner::{CustomExecutor, PinnedFuture};
use async_task::{Runnable, Task};

use crate::app::EventLoopEvent;

pub(crate) type SpawnFn = Box<dyn Fn(PinnedFuture<()>) + Send + Sync>;

#[derive(Debug, Clone)]
pub(crate) struct AppTaskProxy {
    pub proxy: super::AppEventLoopProxy,
    pub task_sender: mpsc::Sender<Runnable>,
}

impl Deref for AppTaskProxy {
    type Target = super::AppEventLoopProxy;
    fn deref(&self) -> &Self::Target {
        &self.proxy
    }
}

impl AppTaskProxy {
    pub fn create_task<F>(&self, fut: F) -> Task<F::Output>
    where
        F: Future + 'static,
        F::Output: 'static,
    {
        let proxy = self.clone();
        #[cfg(feature = "hotpath")]
        let fut = hotpath::future!(fut);
        let (run, task) = async_task::spawn_local(fut, move |run| {
            // log::trace!("");
            if proxy.task_sender.send(run).is_err() {
                log::warn!("the event loop is already closed!");
            }
            let res = proxy.proxy.send_event(EventLoopEvent::RunTasks);
            if res.is_err() {
                log::warn!("the event loop is already closed!");
            }
        });
        run.schedule();
        task
    }
}

pub struct AppExecutor {
    spawn_fn: SpawnFn,
    // TODO Use [`Arc`]
    proxy: AppTaskProxy,
}

impl CustomExecutor for AppExecutor {
    fn spawn(&self, fut: any_spawner::PinnedFuture<()>) {
        #[cfg(feature = "hotpath")]
        let fut = Box::pin(hotpath::future!(fut));
        (self.spawn_fn)(fut);
    }

    fn spawn_local(&self, fut: any_spawner::PinnedLocalFuture<()>) {
        let task = self.proxy.create_task(fut);
        task.detach();
    }

    fn poll_local(&self) {}
}

impl AppExecutor {
    pub fn new(spawn_fn: SpawnFn, proxy: AppTaskProxy) -> Self {
        Self { spawn_fn, proxy }
    }
}
