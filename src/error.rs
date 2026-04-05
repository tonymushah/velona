use masonry::vello::{self, wgpu};

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum Error {
    EventLoop(#[from] winit::error::EventLoopError),
    RequestAdapter(#[from] wgpu::RequestAdapterError),
    Surface(#[from] wgpu::SurfaceError),
    RequestDevice(#[from] wgpu::RequestDeviceError),
    Vello(#[from] vello::Error),
    #[error("Unsupported surface format")]
    UnsupportedSurfaceFormat,
    #[error("The `any_spawner::Executor` was already been set")]
    ExecutorAlreadyBeenSet,
}
