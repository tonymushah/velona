use any_spawner::PinnedFuture;
use tokio::{
    runtime::{Handle, Runtime},
    task::JoinHandle,
};
use velona_core::reactive::owner::expect_context;

pub struct RuntimeWraper {
    tokio_runtime: Runtime,
}

impl RuntimeWraper {
    pub fn new(runtime: Runtime) -> Self {
        Self {
            tokio_runtime: runtime,
        }
    }
}

impl RuntimeWraper {
    pub fn boxed_spawn_fn(&self) -> Box<dyn Fn(PinnedFuture<()>) + Send + Sync> {
        let handle = self.tokio_runtime.handle().clone();
        Box::new(move |fut| {
            handle.spawn(fut);
        })
    }
    pub fn handle(&self) -> Handle {
        self.tokio_runtime.handle().clone()
    }
}

pub fn expect_tokio_handle() -> Handle {
    expect_context()
}

pub fn tokio_spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    let handle = expect_tokio_handle();
    handle.spawn(future)
}

pub fn tokio_spawn_blocking<F, T>(task: F) -> JoinHandle<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let handle = expect_tokio_handle();
    handle.spawn_blocking(task)
}
