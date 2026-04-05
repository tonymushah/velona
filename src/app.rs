pub(crate) mod el_event;
mod executor;
mod window;

use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use any_spawner::PinnedFuture;
use async_executor::Executor;
use log::warn;
use masonry::{
    core::{DefaultProperties, WindowEvent as MasonryWindowEvent},
    vello::wgpu::{self, InstanceDescriptor},
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{EventLoop, EventLoopBuilder},
    window::WindowId,
};

use window::Window;

use crate::{app::executor::SpawnFn, window::WindowBuilder};

pub(crate) use el_event::{AppEventLoopProxy, EventLoopEvent};

struct App {
    event_loop_proxy: AppEventLoopProxy,
    windows: HashMap<WindowId, Box<Window>>,
    instance: wgpu::Instance,
    default_properties: Arc<DefaultProperties>,
    builder_windows: Option<Vec<WindowBuilder>>,
}

pub struct Builder {
    event_loop_builder: EventLoopBuilder<EventLoopEvent>,
    instance_descriptor: Option<InstanceDescriptor>,
    default_properties: DefaultProperties,
    spawn_fn: Option<SpawnFn>,
    windows: Vec<WindowBuilder>,
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
    pub fn window(mut self, window_builder: WindowBuilder) -> Self {
        self.windows.push(window_builder);
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
            windows: Default::default(),
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
            builder_windows: Some(self.windows),
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
        if let Some(builder_windows) = self.builder_windows.take() {
            if builder_windows.is_empty() {
                event_loop.exit();
            } else {
                for window in builder_windows {
                    if self
                        .event_loop_proxy
                        .send_event(EventLoopEvent::NewWindow(Box::new(window)))
                        .is_err()
                    {
                        log::warn!("the event loop is already dead lol");
                    }
                }
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
            WindowEvent::Destroyed => {
                if self.windows.is_empty() {
                    event_loop.exit();
                }
            }
            WindowEvent::RedrawRequested => {
                self.handle_redraw_request(window_id);
            }
            WindowEvent::Resized(size) => {
                self.handle_resize_event(window_id, size);
            }
            WindowEvent::CloseRequested => {
                self.windows.remove(&window_id);
            }
            _e => {
                log::warn!("event {:#?} handling is not implemented yet", _e);
            }
        }
    }
    fn memory_warning(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        self.windows.shrink_to_fit();
    }
    fn user_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
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
            EventLoopEvent::NewLayer(new_layer) => {
                self.use_window(new_layer.window_id, |window| {
                    window
                        .render_root
                        .add_layer(new_layer.layer.0.take(), new_layer.point);
                });
            }
            EventLoopEvent::RemoveLayer(render_root_remove_layer) => {
                self.use_window(render_root_remove_layer.window_id, |window| {
                    window
                        .render_root
                        .remove_layer(render_root_remove_layer.widget_id);
                });
            }
            EventLoopEvent::RepositionLayer(render_root_reposition_layer) => {
                self.use_window(render_root_reposition_layer.window_id, |window| {
                    window.render_root.reposition_layer(
                        render_root_reposition_layer.widget_id,
                        render_root_reposition_layer.point,
                    );
                });
            }
            EventLoopEvent::NewWindow(builder) => {
                let window_attributes = builder.window_attributes;
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
                            builder.view,
                            self.default_properties.clone(),
                            access_kit,
                            self.event_loop_proxy.clone(),
                        )) {
                            Ok(new_instance) => {
                                self.windows
                                    .insert(new_instance.winit_window.id(), Box::new(new_instance));
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
        }
    }
}
