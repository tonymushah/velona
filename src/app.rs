pub(crate) mod el_event;
mod executor;
mod window;

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, OnceLock, mpsc},
};

use any_spawner::PinnedFuture;
use async_executor::Executor;
use copypasta::{ClipboardContext, ClipboardProvider};
use log::warn;
use masonry::{
    app::RenderRoot,
    core::{
        DefaultProperties, WindowEvent as MasonryWindowEvent,
        keyboard::{Key, KeyState},
    },
    vello::{
        util::RenderContext,
        wgpu::{self, InstanceDescriptor},
    },
};
use parking_lot::RwLock;
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
    convert_winit_event::{masonry_resize_direction_to_winit, winit_ime_to_masonry},
    utils::todo_warn_of_something,
    window::WindowBuilder,
};

pub(crate) use el_event::{AppEventLoopProxy, EventLoopEvent};

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
    pub fn instance_descriptor_from_env(mut self) -> Self {
        let backends = wgpu::Backends::from_env().unwrap_or_default();
        let flags = wgpu::InstanceFlags::from_build_config().with_env();
        let memory_budget_thresholds = wgpu::MemoryBudgetThresholds::default();
        let backend_options = wgpu::BackendOptions::from_env_or_default();
        let desc = wgpu::InstanceDescriptor {
            backends,
            flags,
            memory_budget_thresholds,
            backend_options,
        };
        self.instance_descriptor = Some(desc);
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
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        let wgpu_instance = wgpu::Instance::new(&instance_descriptor);
        let (signal_sender, signal_receiver) =
            mpsc::channel::<(WindowId, masonry::app::RenderRootSignal)>();
        let mut app = App {
            event_loop_proxy: proxy,
            windows: Default::default(),
            render_context: Arc::new(RwLock::new(RenderContext {
                instance: wgpu_instance,
                devices: Default::default(),
            })),
            default_properties: Arc::new(self.default_properties),
            builder_windows: Some(self.windows),
            owner: self.owner,
            signal_receiver,
            signal_sender,
            clipboard_context: ClipboardContext::new().unwrap(),
        };
        event_loop.run_app(&mut app)?;
        Ok(())
    }
}

struct App {
    event_loop_proxy: AppEventLoopProxy,
    windows: HashMap<WindowId, Box<Window>>,
    default_properties: Arc<DefaultProperties>,
    builder_windows: Option<Vec<WindowBuilder>>,
    owner: Owner,
    render_context: Arc<RwLock<RenderContext>>,
    signal_receiver: mpsc::Receiver<(WindowId, masonry::app::RenderRootSignal)>,
    signal_sender: mpsc::Sender<(WindowId, masonry::app::RenderRootSignal)>,
    clipboard_context: ClipboardContext,
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
            window.render_root.use_render_root_mut(|r| fun(r))
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
    fn handle_signals(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        let mut need_redraw = HashSet::<WindowId>::new();
        loop {
            let Some((window_id, signal)) = self.signal_receiver.try_iter().next() else {
                break;
            };
            let event_loop_proxy = self.event_loop_proxy.clone();
            self.use_window(window_id, |window| {
                match signal {
                    masonry::app::RenderRootSignal::Action(any_debug, widget_id) => {
                        window
                            .window_event_handler
                            .read()
                            .handle_event(widget_id, &any_debug);
                    }
                    masonry::app::RenderRootSignal::StartIme => {
                        window.winit_window.set_ime_allowed(true);
                    }
                    masonry::app::RenderRootSignal::EndIme => {
                        window.winit_window.set_ime_allowed(false);
                    }
                    masonry::app::RenderRootSignal::ImeMoved(logical_position, logical_size) => {
                        window
                            .winit_window
                            .set_ime_cursor_area(logical_position, logical_size);
                    }
                    masonry::app::RenderRootSignal::ClipboardStore(text) => {
                        let _ =
                            event_loop_proxy.send_event(EventLoopEvent::SetClipboardContent(text));
                    }
                    masonry::app::RenderRootSignal::RequestRedraw => {
                        need_redraw.insert(window_id);
                    }
                    masonry::app::RenderRootSignal::RequestAnimFrame => {
                        // TODO
                        need_redraw.insert(window_id);
                    }
                    masonry::app::RenderRootSignal::TakeFocus => {
                        window.winit_window.focus_window();
                    }
                    masonry::app::RenderRootSignal::SetCursor(cursor_icon) => {
                        window.winit_window.set_cursor(cursor_icon);
                    }
                    masonry::app::RenderRootSignal::SetSize(physical_size) => {
                        // TODO handle return value ??
                        let _ = window.winit_window.request_inner_size(physical_size);
                    }
                    masonry::app::RenderRootSignal::SetTitle(title) => {
                        window.winit_window.set_title(&title);
                    }
                    masonry::app::RenderRootSignal::DragWindow => {
                        // TODO handle return value ??
                        let _ = window.winit_window.drag_window().inspect_err(|err| {
                            log::error!("Unable to drag window => {}", err);
                        });
                    }
                    masonry::app::RenderRootSignal::DragResizeWindow(resize_direction) => {
                        let dir = masonry_resize_direction_to_winit(resize_direction);
                        let _ = window
                            .winit_window
                            .drag_resize_window(dir)
                            .inspect_err(|err| {
                                log::error!("Unable to drag window => {}", err);
                            });
                    }
                    masonry::app::RenderRootSignal::ToggleMaximized => {
                        window
                            .winit_window
                            .set_maximized(!window.winit_window.is_maximized());
                    }
                    masonry::app::RenderRootSignal::Minimize => {
                        window.winit_window.set_minimized(true);
                    }
                    masonry::app::RenderRootSignal::Exit => {
                        let _ = event_loop_proxy.send_event(EventLoopEvent::CloseWindow(window_id));
                    }
                    masonry::app::RenderRootSignal::ShowWindowMenu(logical_position) => {
                        window.winit_window.show_window_menu(logical_position);
                    }
                    masonry::app::RenderRootSignal::WidgetSelectedInInspector(widget_id) => {
                        window.render_root.use_render_root_ref(|render_root| {
                            let Some(widget) = render_root.get_widget(widget_id) else {
                                return;
                            };
                            let widget_name = widget.short_type_name();
                            let display_name = if let Some(debug_text) = widget.get_debug_text() {
                                format!("{widget_name}<{debug_text}>")
                            } else {
                                widget_name.into()
                            };
                            log::info!(
                                "Widget selected in inspector: {widget_id} - {display_name}"
                            );
                        });
                    }
                    masonry::app::RenderRootSignal::NewLayer(new_widget, point) => {
                        window.render_root.use_render_root_mut(|render_root| {
                            render_root.add_layer(new_widget, point);
                        });
                    }
                    masonry::app::RenderRootSignal::RemoveLayer(widget_id) => {
                        window.render_root.use_render_root_mut(|render_root| {
                            render_root.remove_layer(widget_id);
                        });
                    }
                    masonry::app::RenderRootSignal::RepositionLayer(widget_id, point) => {
                        window.render_root.use_render_root_mut(|render_root| {
                            render_root.reposition_layer(widget_id, point);
                        });
                    }
                }
            });
        }
        for window_id in need_redraw {
            self.use_window(window_id, |window| {
                window.winit_window.request_redraw();
            });
        }
    }
    fn create_window(
        &mut self,
        builder: Box<WindowBuilder>,
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) {
        let window_attributes = builder.window_attributes;
        match event_loop.create_window(window_attributes) {
            Ok(window) => {
                let window = Arc::new(window);
                let access_kit = accesskit_winit::Adapter::with_event_loop_proxy(
                    event_loop,
                    &window,
                    self.event_loop_proxy.clone(),
                );
                match Window::new(WindowNew {
                    window,
                    view: builder.view,
                    default_properties: self.default_properties.clone(),
                    access_kit,
                    event_loop_proxy: self.event_loop_proxy.clone(),
                    parent_owner: &self.owner,
                    base_color: builder.base_color,
                    render_context: &self.render_context,
                    signal_sender: self.signal_sender.clone(),
                }) {
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
        self.handle_signals(event_loop);
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
            EventLoopEvent::NewWindow(builder) => {
                self.create_window(builder, event_loop);
            }
            EventLoopEvent::CloseWindow(window_id) => {
                self.windows.remove(&window_id);
            }
            EventLoopEvent::SetClipboardContent(text) => {
                let _ = self
                    .clipboard_context
                    .set_contents(text)
                    .inspect_err(|err| log::error!("cannot set clipboard content => {err}"));
            }
        }
        self.handle_signals(event_loop);
    }
}
