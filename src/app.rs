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
    app::RenderRoot,
    core::{
        DefaultProperties, WindowEvent as MasonryWindowEvent,
        keyboard::{Key, KeyState},
    },
    vello::wgpu::{self, InstanceDescriptor},
};
use reactive_graph::owner::Owner;
use ui_events_winit::WindowEventTranslation;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{EventLoop, EventLoopBuilder},
    window::WindowId,
};

use window::Window;

use crate::{
    app::{executor::SpawnFn, window::WindowNew},
    convert_winit_event::winit_ime_to_masonry,
    utils::todo_warn_of_something,
    window::WindowBuilder,
};

pub(crate) use el_event::{AppEventLoopProxy, EventLoopEvent};

struct App {
    event_loop_proxy: AppEventLoopProxy,
    windows: HashMap<WindowId, Box<Window>>,
    instance: wgpu::Instance,
    default_properties: Arc<DefaultProperties>,
    builder_windows: Option<Vec<WindowBuilder>>,
    owner: Owner,
}

pub struct Builder {
    event_loop_builder: EventLoopBuilder<EventLoopEvent>,
    instance_descriptor: Option<InstanceDescriptor>,
    default_properties: DefaultProperties,
    spawn_fn: Option<SpawnFn>,
    windows: Vec<WindowBuilder>,
    owner: Owner,
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
            owner: Owner::new(),
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
            owner: self.owner,
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
    fn use_window_ref<F, R>(&mut self, window_id: WindowId, fun: F) -> Option<R>
    where
        F: FnOnce(&Window) -> R,
    {
        if let Some(window) = self.windows.get(&window_id) {
            Some(fun(window))
        } else {
            warn!("No matching window state found for {:?}", window_id);
            None
        }
    }
    fn use_window_render_root<F, R>(&mut self, window_id: WindowId, fun: F) -> Option<R>
    where
        F: FnOnce(&mut RenderRoot) -> R,
    {
        self.use_window(window_id, |window| {
            window
                .render_root
                .use_inner_render_root_mut(|r| fun(&mut r.tree))
        })
        .flatten()
    }
    fn handle_redraw_request(&mut self, window_id: WindowId) {
        self.use_window(window_id, |win| match win.render() {
            Ok(_) => {}
            Err(crate::error::Error::Surface(
                wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
            )) => {
                let size = win.winit_window.inner_size();
                win.render_root.use_inner_render_root_mut(|inner| {
                    inner
                        .tree
                        .handle_window_event(MasonryWindowEvent::Resize(size));
                });
            }
            Err(e) => {
                log::error!("Unable to render {}", e);
            }
        });
    }
    fn handle_resize_event(&mut self, window_id: WindowId, size: PhysicalSize<u32>) {
        self.use_window_render_root(window_id, |render_root| {
            render_root.handle_window_event(MasonryWindowEvent::Resize(size));
        });
    }
}

impl ApplicationHandler<EventLoopEvent> for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(builder_windows) = self.builder_windows.take() {
            if builder_windows.is_empty() {
                log::warn!("No window provided! Exiting...");
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
        self.use_window(window_id, |window| {
            if !matches!(
                event,
                WindowEvent::KeyboardInput {
                    is_synthetic: true,
                    ..
                }
            ) && let Some(wet) = window
                .event_reducer
                .reduce(window.winit_window.scale_factor(), &event)
            {
                match wet {
                    WindowEventTranslation::Keyboard(k) => {
                        // TODO - Detect in Masonry code instead
                        let action_mod = if cfg!(target_os = "macos") {
                            k.modifiers.meta()
                        } else {
                            k.modifiers.ctrl()
                        };
                        if let Key::Character(c) = &k.key
                            && c.as_str().eq_ignore_ascii_case("v")
                            && action_mod
                            && k.state == KeyState::Down
                        {
                            window.render_root.use_inner_render_root_mut(|_rr| {
                                todo_warn_of_something("Clipboard Paste");
                                /*
                                rr.tree.handle_text_event(TextEvent::ClipboardPaste(
                                    self.clipboard_cx.get_contents().unwrap(),
                                ));*/
                            });
                        } else {
                            window.render_root.use_inner_render_root_mut(|rr| {
                                rr.tree
                                    .handle_text_event(masonry::core::TextEvent::Keyboard(k));
                            });
                        }
                    }
                    WindowEventTranslation::Pointer(p) => {
                        window.render_root.use_inner_render_root_mut(|rr| {
                            rr.tree.handle_pointer_event(p);
                        });
                    }
                }
            }
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
            WindowEvent::Ime(ime) => {
                let ime = winit_ime_to_masonry(ime);
                self.use_window_render_root(window_id, |render_root| {
                    render_root.handle_text_event(masonry::core::TextEvent::Ime(ime));
                });
            }
            _e => {
                log::trace!("event {:#?} handling is not implemented yet", _e);
            }
        }
    }
    fn memory_warning(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        self.windows.shrink_to_fit();
        self.windows
            .values_mut()
            .for_each(|w| w.on_memory_warning());
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
                        window.render_root.use_inner_render_root_mut(|render_root| {
                            render_root
                                .tree
                                .handle_window_event(MasonryWindowEvent::EnableAccessTree);
                        });
                    }
                    accesskit_winit::WindowEvent::ActionRequested(action_request) => {
                        window.render_root.use_inner_render_root_mut(|inner| {
                            inner.tree.handle_access_event(action_request);
                        });
                    }
                    accesskit_winit::WindowEvent::AccessibilityDeactivated => {
                        window.render_root.use_inner_render_root_mut(|render_root| {
                            render_root
                                .tree
                                .handle_window_event(MasonryWindowEvent::DisableAccessTree);
                        });
                    }
                });
            }
            EventLoopEvent::RunTask(runnable) => {
                log::trace!("running task");
                runnable.run();
            }
            EventLoopEvent::NewLayer(new_layer) => {
                self.use_window_render_root(new_layer.window_id, |render_root| {
                    render_root.add_layer(new_layer.layer.0.take(), new_layer.point)
                });
            }
            EventLoopEvent::RemoveLayer(render_root_remove_layer) => {
                self.use_window_render_root(render_root_remove_layer.window_id, |render_root| {
                    render_root.remove_layer(render_root_remove_layer.widget_id);
                });
            }
            EventLoopEvent::RepositionLayer(render_root_reposition_layer) => {
                self.use_window_render_root(
                    render_root_reposition_layer.window_id,
                    |render_root| {
                        render_root.reposition_layer(
                            render_root_reposition_layer.widget_id,
                            render_root_reposition_layer.point,
                        );
                    },
                );
            }
            EventLoopEvent::NewWindow(builder) => {
                let window_attributes = builder.window_attributes;
                match event_loop.create_window(window_attributes) {
                    Ok(window) => {
                        let window = Arc::new(window);
                        let access_kit = accesskit_winit::Adapter::with_event_loop_proxy(
                            event_loop,
                            &window,
                            self.event_loop_proxy.clone(),
                        );
                        match pollster::block_on(Window::new(WindowNew {
                            window,
                            instance: &self.instance,
                            view: builder.view,
                            default_properties: self.default_properties.clone(),
                            access_kit,
                            event_loop_proxy: self.event_loop_proxy.clone(),
                            parent_owner: &self.owner,
                            base_color: builder.base_color,
                        })) {
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
            EventLoopEvent::WidgetAction(widget_action) => {
                log::debug!("{:#?}", widget_action);
                self.use_window_ref(widget_action.window_id, |window| {
                    window
                        .window_event_handler
                        .read()
                        .handle_event(widget_action.widget_id, &widget_action.event);
                });
            }
        }
    }
}
