#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum Error {
    EventLoop(#[from] winit::error::EventLoopError),
    #[error("The `any_spawner::Executor` was already been set")]
    ExecutorAlreadyBeenSet,
    #[error("The vello render context is used somewhere")]
    RenderContextUsedSomewhere,
}
