mod executor;
mod window;

use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
    thread,
};

use any_spawner::PinnedFuture;
use async_executor::Executor;
use log::warn;
use masonry::{
    core::{DefaultProperties, Properties, Widget, WindowEvent as MasonryWindowEvent},
    palette::css,
    properties::ContentColor,
    vello::wgpu::{self, InstanceDescriptor},
    widgets::{ChildAlignment, ZStack},
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{EventLoop, EventLoopBuilder, EventLoopProxy},
    window::{WindowAttributes, WindowId},
};

use window::Window;

use crate::{app::executor::SpawnFn, utils::todo_warn};

pub(crate) enum EventLoopEvent {
    AccessKitAction(Box<accesskit_winit::Event>),
    RunTask(async_task::Runnable),
}

pub(crate) type AppEventLoopProxy = EventLoopProxy<EventLoopEvent>;

impl From<accesskit_winit::Event> for EventLoopEvent {
    fn from(value: accesskit_winit::Event) -> Self {
        Self::AccessKitAction(Box::new(value))
    }
}

struct App {
    event_loop_proxy: AppEventLoopProxy,
    windows: HashMap<WindowId, Window>,
    instance: wgpu::Instance,
    default_properties: Arc<DefaultProperties>,
}

pub struct Builder {
    event_loop_builder: EventLoopBuilder<EventLoopEvent>,
    instance_descriptor: Option<InstanceDescriptor>,
    default_properties: DefaultProperties,
    spawn_fn: Option<SpawnFn>,
}

impl Builder {
    pub fn instance_descriptor(mut self, instance_descriptor: InstanceDescriptor) -> Self {
        self.instance_descriptor = Some(instance_descriptor);
        self
    }
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
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            event_loop_builder: EventLoop::with_user_event(),
            instance_descriptor: None,
            default_properties: Default::default(),
            spawn_fn: None,
        }
    }
}

impl Builder {
    pub fn run(mut self) -> Result<(), crate::error::Error> {
        let spawn_fn = self.spawn_fn.unwrap_or_else(|| {
            static EXECUTOR: OnceLock<async_executor::Executor<'static>> = OnceLock::new();
            Box::new(|fut| {
                EXECUTOR.get_or_init(Executor::new).spawn(fut).detach();
            })
        });
        let event_loop = self.event_loop_builder.build()?;
        let proxy = event_loop.create_proxy();
        match any_spawner::Executor::init_custom_executor(executor::AppExecutor::new(
            spawn_fn,
            proxy.clone(),
        )) {
            Ok(_) => {}
            Err(_) => return Err(crate::error::Error::ExecutorAlreadyBeenSet),
        }
        let instance_descriptor = self.instance_descriptor.unwrap_or(InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY.union(wgpu::Backends::SECONDARY),
            ..Default::default()
        });

        let mut app = App {
            event_loop_proxy: proxy,
            windows: Default::default(),
            instance: wgpu::Instance::new(&instance_descriptor),
            default_properties: Arc::new(self.default_properties),
        };
        event_loop.run_app(&mut app)?;
        Ok(())
    }
}

impl App {
    fn use_window<F, R>(&mut self, window_id: WindowId, fun: F) -> Option<R>
    where
        F: FnOnce(&mut Window) -> R,
    {
        if let Some(window) = self.windows.get_mut(&window_id) {
            Some(fun(window))
        } else {
            warn!("No matching window state found for {:?}", window_id);
            None
        }
    }
    fn handle_redraw_request(&mut self, window_id: WindowId) {
        self.use_window(window_id, |win| match win.render() {
            Ok(_) => {}
            Err(crate::error::Error::Surface(
                wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
            )) => {
                let size = win.winit_window.inner_size();
                win.render_root
                    .handle_window_event(MasonryWindowEvent::Resize(size));
            }
            Err(e) => {
                log::error!("Unable to render {}", e);
            }
        });
    }
    fn handle_resize_event(&mut self, window_id: WindowId, size: PhysicalSize<u32>) {
        self.use_window(window_id, |window| {
            window
                .render_root
                .handle_window_event(MasonryWindowEvent::Resize(size));
        });
    }
}

impl ApplicationHandler<EventLoopEvent> for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window_attributes = non_exhaustive::non_exhaustive!(WindowAttributes {
            visible: true,
            inner_size: Some(winit::dpi::Size::Physical(PhysicalSize {
                width: 500,
                height: 300,
            })),
        });
        match event_loop.create_window(window_attributes) {
            Ok(window) => {
                let access_kit = accesskit_winit::Adapter::with_event_loop_proxy(
                    event_loop,
                    &window,
                    self.event_loop_proxy.clone(),
                );
                match pollster::block_on(Window::new(
                    window,
                    &self.instance,
                    || {
                        ZStack::new()
                            .with_child(
                                masonry::widgets::Label::new("aaaaaaa")
                                    .with_props(Properties::one(ContentColor::new(css::BEIGE))),
                                ChildAlignment::ParentAligned,
                            )
                            .with_auto_id()
                            .erased()
                    },
                    self.default_properties.clone(),
                    access_kit,
                )) {
                    Ok(new_instance) => {
                        self.windows
                            .insert(new_instance.winit_window.id(), new_instance);
                    }
                    Err(err) => {
                        log::error!("Cannot create new window ({err})")
                    }
                }
            }
            Err(err) => {
                log::error!("Os error on creating new window {err}");
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: WindowId,
        event: winit::event::WindowEvent,
    ) {
        self.use_window(window_id, |window| {
            window
                .access_kit
                .process_event(&window.winit_window, &event);
        });
        match event {
            WindowEvent::RedrawRequested => {
                self.handle_redraw_request(window_id);
            }
            WindowEvent::Resized(size) => {
                self.handle_resize_event(window_id, size);
            }
            WindowEvent::CloseRequested => {
                self.windows.remove(&window_id);
                if self.windows.is_empty() {
                    event_loop.exit();
                }
            }
            _ => {
                todo_warn();
            }
        }
    }
    fn memory_warning(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        self.windows.shrink_to_fit();
    }
    fn exiting(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        thread::sleep(std::time::Duration::from_secs(1));
    }
    fn user_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        event: EventLoopEvent,
    ) {
        match event {
            EventLoopEvent::AccessKitAction(event) => {
                self.use_window(event.window_id, |window| match event.window_event {
                    accesskit_winit::WindowEvent::InitialTreeRequested => {
                        window.winit_window.request_redraw();
                    }
                    accesskit_winit::WindowEvent::ActionRequested(action_request) => {
                        window.render_root.handle_access_event(action_request);
                    }
                    accesskit_winit::WindowEvent::AccessibilityDeactivated => {
                        window.winit_window.request_redraw();
                    }
                });
            }
            EventLoopEvent::RunTask(runnable) => {
                runnable.run();
            }
        }
    }
}
