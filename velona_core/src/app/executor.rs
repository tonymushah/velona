use any_spawner::{CustomExecutor, PinnedFuture, PinnedLocalFuture};
use send_wrapper::SendWrapper;

use crate::app::{EventLoopEvent, proxy::AppEventLoopProxy};

pub(crate) type SpawnFn = Box<dyn Fn(PinnedFuture<()>) + Send + Sync>;

impl AppEventLoopProxy {
    pub fn create_task(&self, fut: PinnedLocalFuture<()>) {
        #[cfg(feature = "hotpath")]
        let fut = hotpath::future!(fut);
        if let Err(err) = self.send_event(EventLoopEvent::SpawnTaskLocal(SendWrapper::new(fut))) {
            log::error!("{err}")
        }
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
        self.proxy.create_task(fut);
    }

    fn poll_local(&self) {}
}

impl AppExecutor {
    pub fn new(spawn_fn: SpawnFn, proxy: AppEventLoopProxy) -> Self {
        Self { spawn_fn, proxy }
    }
}
