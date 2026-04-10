use any_spawner::{CustomExecutor, PinnedFuture};

use crate::app::{AppEventLoopProxy, EventLoopEvent};

pub(crate) type SpawnFn = Box<dyn Fn(PinnedFuture<()>) + Send + Sync>;

pub struct AppExecutor {
    spawn_fn: SpawnFn,
    proxy: super::AppEventLoopProxy,
}

impl CustomExecutor for AppExecutor {
    fn spawn(&self, fut: any_spawner::PinnedFuture<()>) {
        (self.spawn_fn)(fut);
    }

    fn spawn_local(&self, fut: any_spawner::PinnedLocalFuture<()>) {
        let proxy = self.proxy.clone();
        let (run, task) = async_task::spawn_local(fut, move |run| {
            log::trace!("");
            let res = proxy.send_event(EventLoopEvent::RunTask(Box::new(run)));
            if res.is_err() {
                log::warn!("the event loop is already closed!");
            }
        });
        task.detach();
        run.schedule();
    }

    fn poll_local(&self) {}
}

impl AppExecutor {
    pub fn new(spawn_fn: SpawnFn, proxy: AppEventLoopProxy) -> Self {
        Self { spawn_fn, proxy }
    }
}
