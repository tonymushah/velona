use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};

use async_task::Runnable;
use copypasta::{ClipboardContext, ClipboardProvider};
use log::warn;
use masonry_core::app::RenderRootSignal;
use masonry_core::{
    app::RenderRoot,
    core::{
        DefaultProperties, TextEvent, WindowEvent as MasonryWindowEvent,
        keyboard::{Key, KeyState},
    },
};
use reactive_graph::owner::Owner;
use ui_events_winit::WindowEventTranslation;
use velona_renderer::WindowRenderer;
use winit::{
    application::ApplicationHandler, dpi::PhysicalSize, event::WindowEvent,
    event_loop::ActiveEventLoop, window::WindowId,
};

use super::window::Window;

use crate::app::OnEventLoopInitFns;
use crate::app::event_listener::{AppEventHandlers, EmitAppEventToHandlers};
use crate::events::el_event::{RegisterEventHandler, UnregisterEventHandler};
use crate::utils::HandlerId;
use crate::window;
use crate::{
    app::proxy::EventProxyHandle,
    app::{AppHandle, EventLoopEvent, window::WindowNew},
    utils::convert_winit_event::{masonry_resize_direction_to_winit, winit_ime_to_masonry},
    utils::{FlumeReceiver, todo_warn_of_something},
    window::{builder::WindowBuilder, renderer::WindowRendererFactory},
};

pub(crate) struct AppRunner<W>
where
    W: WindowRenderer,
{
    pub(crate) app_handle: AppHandle,
    pub(crate) windows: HashMap<WindowId, Box<Window<W>>>,
    pub(crate) default_properties: Arc<DefaultProperties>,
    pub(crate) builder_windows: Option<Vec<WindowBuilder>>,
    pub(crate) owner: Owner,
    pub(crate) window_renderer_factory: Box<dyn WindowRendererFactory<WindowRenderer = W>>,
    pub(crate) clipboard_context: Rc<RefCell<ClipboardContext>>,
    pub(crate) suspended: bool,
    pub(crate) receiver: FlumeReceiver<EventLoopEvent>,
    pub(crate) on_event_loop_init: Option<OnEventLoopInitFns>,
    pub(crate) app_event_listeners: AppEventHandlers,
}

// ------- Utilities --------- //
#[cfg_attr(feature = "hotpath", hotpath::measure_all)]
impl<W> AppRunner<W>
where
    W: WindowRenderer,
{
    fn use_window<F, R>(&mut self, window_id: WindowId, fun: F) -> Option<R>
    where
        F: FnOnce(&mut Window<W>) -> R,
    {
        if let Some(window) = self.windows.get_mut(&window_id) {
            Some(fun(window))
        } else {
            warn!("No matching window state found for {:?}", window_id);
            None
        }
    }
    fn use_window_ref<F, R>(&self, window_id: WindowId, fun: F) -> Option<R>
    where
        F: FnOnce(&Window<W>) -> R,
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
    fn use_window_render_root_ref<F, R>(&mut self, window_id: WindowId, fun: F) -> Option<R>
    where
        F: FnOnce(&RenderRoot) -> R,
    {
        self.use_window_ref(window_id, |window| {
            window
                .render_root
                .use_inner_render_root_ref(|r| fun(&r.tree))
        })
        .flatten()
    }
    fn create_window_owner_children(&self, window_id: WindowId) -> Option<Owner> {
        self.use_window_ref(window_id, |window| window.create_children_owner())
    }
}

// ------- Window creation -------- //
#[cfg_attr(feature = "hotpath", hotpath::measure_all)]
impl<W> AppRunner<W>
where
    W: WindowRenderer,
{
    fn create_window(
        &mut self,
        builder: Box<WindowBuilder>,
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) {
        let window_attributes = builder.window_attributes;
        match event_loop.create_window(window_attributes) {
            Ok(window) => {
                let window = Arc::new(window);
                let access_kit = accesskit_winit::Adapter::with_direct_handlers(
                    event_loop,
                    &window,
                    self.app_handle.get_proxy().accesskit_handler(window.id()),
                    self.app_handle.get_proxy().accesskit_handler(window.id()),
                    self.app_handle.get_proxy().accesskit_handler(window.id()),
                );
                match Window::new(WindowNew {
                    window,
                    view: builder.view,
                    default_properties: builder
                        .default_propreties
                        .unwrap_or(self.default_properties.clone()),
                    access_kit,
                    app_handle: self.app_handle.clone(),
                    parent_owner: &self.owner,
                    base_color: builder.base_color,
                    factory: &mut *self.window_renderer_factory
                        as &mut dyn WindowRendererFactory<WindowRenderer = W>,
                    size_policy: builder.size_policy,
                    use_system_fonts: builder.use_system_fonts,
                }) {
                    Ok(mut new_instance) => {
                        if !self.suspended {
                            new_instance.resume();
                        }
                        if let Some(sender) = builder.window_handle_send {
                            let _ = sender.send(new_instance.get_handle());
                        }
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

// ------- Event Handling --------- //
#[cfg_attr(feature = "hotpath", hotpath::measure_all)]
impl<W> AppRunner<W>
where
    W: WindowRenderer,
{
    fn register_event_handler(&mut self, handler: RegisterEventHandler) {
        match handler {
            RegisterEventHandler::App(register_app_event) => {
                self.app_event_listeners
                    .register_handler(register_app_event);
            }
            RegisterEventHandler::Window { window_id, type_ } => {
                self.use_window(window_id, |window| {
                    window.window_event_listeners.add_handler_fn(type_);
                });
            }
        }
    }
    fn unregister_handler_from_global(&mut self, handler_id: &HandlerId) {
        for window in self.windows.values_mut() {
            if window
                .window_event_listeners
                .remove_handler(handler_id, None)
            {
                break;
            }
        }
        todo!()
    }
    fn handle_unregister_event_handler(&mut self, event: UnregisterEventHandler) {
        match event {
            UnregisterEventHandler::Any(handler_id) => {
                self.unregister_handler_from_global(&handler_id);
            }
            UnregisterEventHandler::App(un_register_app_event_handler) => {
                self.app_event_listeners
                    .unregister_handler(un_register_app_event_handler);
            }
            UnregisterEventHandler::Window {
                window_id,
                handler_id,
                type_,
            } => {
                self.use_window(window_id, |window| {
                    window
                        .window_event_listeners
                        .remove_handler(&handler_id, type_);
                });
            }
        }
    }
}

// --- WINIT miscs --- //
#[cfg_attr(feature = "hotpath", hotpath::measure_all)]
impl<W> AppRunner<W>
where
    W: WindowRenderer,
{
    fn run_task(&self, run: Runnable) {
        run.run();
    }
    fn resume_windows(&mut self) {
        for window in self.windows.values_mut() {
            window.resume();
        }
    }
    fn suspend_windows(&mut self) {
        for window in self.windows.values_mut() {
            window.suspend();
        }
    }
    fn run_exiting_task(&self) -> usize {
        let mut tasks = 0usize;
        while let Some(EventLoopEvent::RunTask(run)) = self.receiver.try_iter().next() {
            run.run();
            tasks += 1;
        }
        tasks
    }
}

// --- WINIT event loop handlers --- //
#[cfg_attr(feature = "hotpath", hotpath::measure_all)]
impl<W> AppRunner<W>
where
    W: WindowRenderer,
{
    fn handle_redraw_request(&mut self, window_id: WindowId) {
        self.use_window(window_id, |win| {
            if win.complete_resume() {
                match win.render() {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("Unable to render {}", e);
                    }
                }
            }
        });
    }
    fn handle_resize_event(&mut self, window_id: WindowId, size: PhysicalSize<u32>) {
        self.use_window_render_root(window_id, |render_root| {
            render_root.handle_window_event(MasonryWindowEvent::Resize(size));
        });
        self.use_window(window_id, |window| {
            window.sync_surface_render_root_size();
        });
    }
    fn handle_signal(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: WindowId,
        signal: RenderRootSignal,
    ) {
        let event_loop_proxy = self.app_handle.get_proxy().clone();

        self.use_window(window_id, |window| {
            match signal {
                RenderRootSignal::Action(any_debug, widget_id) => {
                    let child_owner = window.create_children_owner();

                    child_owner.with(|| {
                        window.window_event_listeners.handle_event(
                            window::event_listener::HandleEvent::Widget {
                                widget_id,
                                action: &any_debug,
                            },
                        )
                    });
                }
                RenderRootSignal::StartIme => {
                    window.winit_window.set_ime_allowed(true);
                }
                RenderRootSignal::EndIme => {
                    window.winit_window.set_ime_allowed(false);
                }
                RenderRootSignal::ImeMoved(logical_position, logical_size) => {
                    window
                        .winit_window
                        .set_ime_cursor_area(logical_position, logical_size);
                }
                RenderRootSignal::ClipboardStore(text) => {
                    let _ = event_loop_proxy.send_event(EventLoopEvent::SetClipboardContent(text));
                }
                RenderRootSignal::RequestRedraw => {
                    window.winit_window.request_redraw();
                }
                RenderRootSignal::RequestAnimFrame => {
                    // TODO
                    window.winit_window.request_redraw();
                }
                RenderRootSignal::TakeFocus => {
                    window.winit_window.focus_window();
                }
                RenderRootSignal::SetCursor(cursor_icon) => {
                    window.winit_window.set_cursor(cursor_icon);
                }
                RenderRootSignal::SetSize(physical_size) => {
                    // TODO handle return value ??
                    let _ = window.winit_window.request_inner_size(physical_size);
                }
                RenderRootSignal::SetTitle(title) => {
                    window.winit_window.set_title(&title);
                }
                RenderRootSignal::DragWindow => {
                    // TODO handle return value ??
                    let _ = window.winit_window.drag_window().inspect_err(|err| {
                        log::error!("Unable to drag window => {}", err);
                    });
                }
                RenderRootSignal::DragResizeWindow(resize_direction) => {
                    let dir = masonry_resize_direction_to_winit(resize_direction);
                    let _ = window
                        .winit_window
                        .drag_resize_window(dir)
                        .inspect_err(|err| {
                            log::error!("Unable to drag window => {}", err);
                        });
                }
                RenderRootSignal::ToggleMaximized => {
                    window
                        .winit_window
                        .set_maximized(!window.winit_window.is_maximized());
                }
                RenderRootSignal::Minimize => {
                    window.winit_window.set_minimized(true);
                }
                RenderRootSignal::Exit => {
                    let _ = event_loop_proxy.send_event(EventLoopEvent::CloseWindow(window_id));
                }
                RenderRootSignal::ShowWindowMenu(logical_position) => {
                    window.winit_window.show_window_menu(logical_position);
                }
                RenderRootSignal::WidgetSelectedInInspector(widget_id) => {
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
                        log::info!("Widget selected in inspector: {widget_id} - {display_name}");
                    });
                }
                RenderRootSignal::NewLayer(_type, new_widget, point) => {
                    // TODO implement type
                    window.render_root.use_render_root_mut(|render_root| {
                        render_root.add_layer(new_widget, point);
                    });
                }
                RenderRootSignal::RemoveLayer(widget_id) => {
                    window.render_root.use_render_root_mut(|render_root| {
                        render_root.remove_layer(widget_id);
                    });
                }
                RenderRootSignal::RepositionLayer(widget_id, point) => {
                    window.render_root.use_render_root_mut(|render_root| {
                        render_root.reposition_layer(widget_id, point);
                    });
                }
            }
        });
    }

    fn handle_app_events(&mut self, event_loop: &ActiveEventLoop) {
        while let Some(event) = self.receiver.try_iter().next() {
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
                EventLoopEvent::RunTask(run) => {
                    self.run_task(run);
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
                        .borrow_mut()
                        .set_contents(text)
                        .inspect_err(|err| log::error!("cannot set clipboard content => {err}"));
                }
                EventLoopEvent::HandleRenderRootSignals(window_id, signal) => {
                    self.handle_signal(event_loop, window_id, signal.take());
                }
                EventLoopEvent::EditWidget(edit_widget_fn_event) => {
                    let maybe_owner =
                        self.create_window_owner_children(edit_widget_fn_event.window_id);
                    self.use_window_render_root(edit_widget_fn_event.window_id, |root| {
                        if root.has_widget(edit_widget_fn_event.widget_id) {
                            root.edit_widget(edit_widget_fn_event.widget_id, |widget_mut| {
                                if let Some(owner) = maybe_owner {
                                    owner.with_cleanup(|| {
                                        (edit_widget_fn_event.edit_fn)(widget_mut);
                                    })
                                } else {
                                    (edit_widget_fn_event.edit_fn)(widget_mut);
                                }
                            });
                        }
                    });
                }
                EventLoopEvent::UseWidget(use_widget_fn_event) => {
                    let maybe_owner =
                        self.create_window_owner_children(use_widget_fn_event.window_id);
                    self.use_window_render_root_ref(use_widget_fn_event.window_id, |root| {
                        let Some(widget_ref) = root.get_widget(use_widget_fn_event.widget_id)
                        else {
                            return;
                        };
                        if let Some(owner) = maybe_owner {
                            owner.with_cleanup(|| {
                                (use_widget_fn_event.use_fn)(widget_ref);
                            })
                        } else {
                            (use_widget_fn_event.use_fn)(widget_ref);
                        }
                    });
                }

                EventLoopEvent::UseWindowRenderRoot(use_window_render_root_on_main) => {
                    self.use_window_render_root(
                        use_window_render_root_on_main.window_id,
                        use_window_render_root_on_main.use_fn,
                    );
                }
                EventLoopEvent::UseWinitWindow(use_winit_window_on_main) => {
                    self.use_window_ref(use_winit_window_on_main.window_id, |window| {
                        (use_winit_window_on_main.use_fn)(&window.winit_window);
                    });
                }
                EventLoopEvent::GetWindowChildReactiveOwner(get_window_child_reactive_owner) => {
                    self.use_window_ref(get_window_child_reactive_owner.window_id, |window| {
                        let res = get_window_child_reactive_owner
                            .sender
                            .send(window.create_children_owner());
                        if res.is_err() {
                            log::warn!("Cannot send window child owner");
                        }
                    });
                }
                EventLoopEvent::GetAppChildReactiveOwner(get_app_child_reactive_owner) => {
                    if get_app_child_reactive_owner
                        .sender
                        .send(self.owner.child())
                        .is_err()
                    {
                        log::warn!("Cannot send app child owner");
                    }
                }
                EventLoopEvent::RegisterHandler(register_event_handler) => {
                    self.register_event_handler(*register_event_handler);
                }
                EventLoopEvent::UnRegisterHandler(unregister_event_handler) => {
                    self.handle_unregister_event_handler(*unregister_event_handler)
                }
            }
        }
    }
}

impl<W> Drop for AppRunner<W>
where
    W: WindowRenderer,
{
    fn drop(&mut self) {
        self.owner.cleanup();
        let task_runned = self.run_exiting_task();

        log::trace!("Number of drop tasks: {task_runned}");
    }
}

#[cfg_attr(feature = "hotpath", hotpath::measure_all)]
impl<W> ApplicationHandler<()> for AppRunner<W>
where
    W: WindowRenderer,
{
    fn new_events(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        cause: winit::event::StartCause,
    ) {
        if cause == winit::event::StartCause::Init {
            if let Some(on_init) = self.on_event_loop_init.take() {
                for func in on_init {
                    func(&self.app_handle);
                }
            }
            if let Some(builder_windows) = self.builder_windows.take() {
                if builder_windows.is_empty() {
                    log::warn!("No window provided! Exiting...");
                    event_loop.exit();
                } else {
                    for window in builder_windows {
                        if self
                            .app_handle
                            .send_event(EventLoopEvent::NewWindow(Box::new(window)))
                            .is_err()
                        {
                            log::warn!("the event loop is already dead lol");
                        }
                    }
                }
            }
        }
    }
    fn resumed(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        self.suspended = false;
        self.resume_windows();
        self.app_event_listeners
            .emit(EmitAppEventToHandlers::Resumed);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: WindowId,
        event: winit::event::WindowEvent,
    ) {
        // #[cfg(feature = "hotpath")]
        // hotpath::dbg!((&window_id, &event));
        self.use_window(window_id, |window| {
            window
                .access_kit
                .process_event(&window.winit_window, &event);
        });
        let clipboard_context = self.clipboard_context.clone();
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

                                _rr.tree.handle_text_event(TextEvent::ClipboardPaste(
                                    clipboard_context.borrow_mut().get_contents().unwrap(),
                                ));
                            });
                        } else {
                            window.render_root.use_inner_render_root_mut(|rr| {
                                rr.tree
                                    .handle_text_event(masonry_core::core::TextEvent::Keyboard(k));
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
            WindowEvent::Destroyed if self.windows.is_empty() => {
                event_loop.exit();
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
                    render_root.handle_text_event(masonry_core::core::TextEvent::Ime(ime));
                });
            }
            WindowEvent::ScaleFactorChanged {
                scale_factor,
                // TODO use this??
                inner_size_writer: _,
            } => {
                self.use_window_render_root(window_id, |rr| {
                    rr.handle_window_event(masonry_core::core::WindowEvent::Rescale(scale_factor));
                });
            }
            _e => {
                // log::trace!("event {:#?} handling is not implemented yet", _e);
            }
        }
    }
    fn memory_warning(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        self.windows.shrink_to_fit();
        self.windows
            .values_mut()
            .for_each(|w| w.on_memory_warning());
        self.app_event_listeners
            .emit(EmitAppEventToHandlers::MemoryWarning);
        self.app_event_listeners.shrink_to_fit();
    }
    fn suspended(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        self.suspended = true;
        self.suspend_windows();
        self.app_event_listeners
            .emit(EmitAppEventToHandlers::Suspended);
    }
    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, _: ()) {
        // #[cfg(feature = "hotpath")]
        // hotpath::dbg!(&event);
        self.handle_app_events(event_loop);
    }
    fn exiting(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        log::warn!("Exiting...");
        let task_runned = self.run_exiting_task();
        log::trace!("Number of exiting tasks: {task_runned}");
    }
    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        self.app_event_listeners
            .emit(EmitAppEventToHandlers::Device(device_id, &event));
    }
}
