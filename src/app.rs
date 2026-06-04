pub(crate) mod el_event;
mod executor;
use crate::window::runner as window;
mod handle;
mod run;
use anyrender::WindowRenderer;
pub(crate) use executor::AppTaskProxy;

use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, mpsc},
};

use crate::{app::executor::SpawnFn, window::builder::WindowBuilder};
use any_spawner::PinnedFuture;
use async_task::Runnable;
use copypasta::ClipboardContext;
use masonry::core::DefaultProperties;
use reactive_graph::owner::Owner;
use winit::{
    event_loop::{EventLoop, EventLoopBuilder},
    window::WindowId,
};

pub(crate) use el_event::{AppEventLoopProxy, EventLoopEvent};

pub struct Builder<W: WindowRenderer> {
    event_loop_builder: EventLoopBuilder<EventLoopEvent>,
    window_render_factory: Box<dyn FnMut(&AppHandle) -> W>,
    default_properties: DefaultProperties,
    spawn_fn: Option<SpawnFn>,
    windows: Vec<WindowBuilder>,
    owner: Owner,
}

impl<W: WindowRenderer> Builder<W> {
    pub fn default_properties(mut self, default_properties: DefaultProperties) -> Self {
        self.default_properties = default_properties;
        self
    }
    pub fn spawn_fn<F>(mut self, spawn_fn: F) -> Self
    where
        F: Fn(PinnedFuture<()>) + Send + Sync + 'static,
    {
        self.spawn_fn = Some(Box::new(spawn_fn));
        self
    }
    pub fn window(mut self, window_builder: WindowBuilder) -> Self {
        self.windows.push(window_builder);
        self
    }
    pub fn new<F>(factory: F) -> Self
    where
        F: FnMut(&AppHandle) -> W + 'static,
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
    pub fn provide_context<T: Send + Sync + 'static>(self, data: T) -> Self {
        self.owner.with(|| {
            reactive_graph::owner::provide_context(data);
        });
        self
    }
}

impl<W: WindowRenderer> Builder<W> {
    pub fn run(mut self) -> Result<(), crate::error::Error> {
        let spawn_fn = self
            .spawn_fn
            .unwrap_or_else(|| Box::new(|_| panic!("No spawn_fn provided")));
        let event_loop = self.event_loop_builder.build()?;
        let proxy = event_loop.create_proxy();
        let (runables_sender, runable_receiver) = mpsc::channel::<Runnable>();

        let proxy = AppTaskProxy {
            task_sender: runables_sender,
            proxy,
        };

        match any_spawner::Executor::init_custom_executor(executor::AppExecutor::new(
            spawn_fn,
            proxy.clone(),
        )) {
            Ok(_) => {}
            Err(_) => return Err(crate::error::Error::ExecutorAlreadyBeenSet),
        }
        let (signal_sender, signal_receiver) =
            mpsc::channel::<(WindowId, masonry::app::RenderRootSignal)>();

        let mut app = run::AppRunner {
            app_handle: AppHandle::new(proxy),
            windows: Default::default(),
            window_renderer_factory: self.window_render_factory,
            default_properties: Arc::new(self.default_properties),
            builder_windows: Some(self.windows),
            owner: self.owner,
            signal_receiver,
            signal_sender,
            clipboard_context: Rc::new(RefCell::new(ClipboardContext::new().unwrap())),
            tasks: runable_receiver,
            suspended: true,
        };
        // event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
        event_loop.run_app(&mut app)?;
        Ok(())
    }
}

// TODO add an Manager trait

pub use handle::{AppHandle, use_app_handle};
