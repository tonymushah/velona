use any_spawner::{CustomExecutor, PinnedFuture};
use async_task::Task;

use crate::app::{EventLoopEvent, proxy::AppEventLoopProxy};

pub(crate) type SpawnFn = Box<dyn Fn(PinnedFuture<()>) + Send + Sync>;

impl AppEventLoopProxy {
    pub fn create_task<F>(&self, fut: F) -> Task<F::Output>
    where
        F: Future + 'static,
        F::Output: 'static,
    {
        let proxy = self.clone();
        #[cfg(feature = "hotpath")]
        let fut = hotpath::future!(fut);
        let (run, task) = async_task::spawn_local(fut, move |run| {
            let res = proxy.send_event(EventLoopEvent::RunTask(run));
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
    proxy: AppEventLoopProxy,
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
    pub fn new(spawn_fn: SpawnFn, proxy: AppEventLoopProxy) -> Self {
        Self { spawn_fn, proxy }
    }
}
