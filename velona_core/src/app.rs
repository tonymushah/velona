pub(crate) mod el_event;
mod executor;
use crate::{
    app::proxy::AppEventLoopProxy,
    utils,
    window::{renderer::WindowRendererFactory, runner as window},
};
mod handle;
mod run;
use velona_renderer::WindowRenderer;
pub(crate) mod proxy;

use std::{cell::RefCell, rc::Rc, sync::Arc};

use crate::{app::executor::SpawnFn, window::builder::WindowBuilder};
use any_spawner::PinnedFuture;
use copypasta::ClipboardContext;
use masonry_core::core::DefaultProperties;
use reactive_graph::owner::Owner;
use winit::event_loop::{EventLoop, EventLoopBuilder};

pub(crate) use el_event::EventLoopEvent;

pub struct Builder<W: WindowRenderer> {
    event_loop_builder: EventLoopBuilder<()>,
    window_render_factory: Box<dyn WindowRendererFactory<WindowRenderer = W>>,
    default_properties: DefaultProperties,
    spawn_fn: Option<SpawnFn>,
    windows: Vec<WindowBuilder>,
    owner: Owner,
}

impl<W: WindowRenderer> Builder<W> {
    /// Default values that properties will have if not defined per-widget.
    ///
    /// This one is app global, Windows can changes their properties with [`WindowBuilder::with_default_properties`](crate::window::WindowBuilder::with_default_properties).
    pub fn with_default_properties(mut self, default_properties: DefaultProperties) -> Self {
        self.default_properties = default_properties;
        self
    }
    /// Sets the [`any_spawner::Executor::spawn`] function
    pub fn with_spawn_fn<F>(mut self, spawn_fn: F) -> Self
    where
        F: Fn(PinnedFuture<()>) + Send + Sync + 'static,
    {
        self.spawn_fn = Some(Box::new(spawn_fn));
        self
    }
    /// Run this app with a window.
    pub fn with_window(mut self, window_builder: WindowBuilder) -> Self {
        self.windows.push(window_builder);
        self
    }
    /// Create a builder with a custom renderer factory.
    pub fn new_with_renderer_factory<F>(factory: F) -> Self
    where
        F: WindowRendererFactory<WindowRenderer = W> + 'static,
    {
        Self {
            event_loop_builder: EventLoop::with_user_event(),
            window_render_factory: Box::new(factory),
            default_properties: Default::default(),
            spawn_fn: None,
            windows: Vec::with_capacity(1),
            owner: Owner::new(),
        }
    }
    pub fn new<F>(factory: F) -> Self
    where
        F: FnMut(&AppHandle) -> W + 'static,
    {
        Self::new_with_renderer_factory(factory)
    }
    /// Provide global context data.
    pub fn provide_context<T: Send + Sync + 'static>(self, data: T) -> Self {
        self.owner.with(|| {
            reactive_graph::owner::provide_context(data);
        });
        self
    }
}

impl<W: WindowRenderer> Builder<W> {
    /// Run the app.
    pub fn run(mut self) -> Result<(), crate::error::Error> {
        let spawn_fn = self
            .spawn_fn
            .unwrap_or_else(|| Box::new(|_| panic!("No spawn_fn provided")));
        let event_loop = self.event_loop_builder.build()?;
        let proxy = event_loop.create_proxy();

        let (send, receiver) = utils::flume_channel::<EventLoopEvent>();

        let proxy = AppEventLoopProxy::new(proxy, send);

        match any_spawner::Executor::init_local_custom_executor(executor::AppExecutor::new(
            spawn_fn,
            proxy.clone(),
        )) {
            Ok(_) => {}
            Err(_) => return Err(crate::error::Error::ExecutorAlreadyBeenSet),
        }

        let mut app = run::AppRunner {
            app_handle: AppHandle::new(proxy),
            windows: Default::default(),
            window_renderer_factory: self.window_render_factory,
            default_properties: Arc::new(self.default_properties),
            builder_windows: Some(self.windows),
            owner: self.owner,
            clipboard_context: Rc::new(RefCell::new(ClipboardContext::new().unwrap())),
            suspended: true,
            receiver,
        };
        // event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
        event_loop.run_app(&mut app)?;
        Ok(())
    }
}

// TODO add an Manager trait

pub use handle::{AppHandle, use_app_handle};
