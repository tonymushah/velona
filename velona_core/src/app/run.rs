use std::collections::HashSet;
use std::time::Instant;
use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};

use any_spawner::{PinnedFuture, PinnedLocalFuture};
use copypasta::{ClipboardContext, ClipboardProvider};
use dpi::PhysicalSize;
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
use ui_events_velona_core::WindowEventTranslation;
use velona_core_accesskit::{AdapterFactory, CreateAdapterArgs};
use velona_executor::{TaskId, VelonaTasksExecutor};
use velona_renderer::WindowRenderer;
use winit_core::window::{ImeCapabilities, ImeEnableRequest, ImeRequestData};
use winit_core::{
    application::ApplicationHandler, event::WindowEvent, event_loop::ActiveEventLoop,
    window::WindowId,
};

use super::window::Window;

use crate::app::OnEventLoopInitFns;
use crate::app::event_listener::{
    AppEventHandlers, EmitAppEventToHandlers, UnRegisterAppEventHandler,
};
use crate::app::executor::SpawnFn;
use crate::app::proxy::AppEventLoopProxy;
use crate::events::el_event::{RegisterEventHandler, UnregisterEventHandler};
use crate::events::property_stack::PropertyStackMethods;
use crate::manager::OtherManagerMethods;
use crate::utils::HandlerId;
use crate::window;
use crate::{
    app::proxy::EventProxyHandle,
    app::{AppHandle, EventLoopEvent, window::WindowNew},
    utils::convert_winit_event::{masonry_resize_direction_to_winit, winit_ime_to_masonry},
    utils::{FlumeReceiver, todo_warn_of_something},
    window::{builder::WindowBuilder, renderer::WindowRendererFactory},
};

pub struct App<W>
where
    W: WindowRenderer,
{
    pub(crate) windows: HashMap<WindowId, Box<Window<W>>>,
    pub(crate) default_properties: Arc<DefaultProperties>,
    pub(crate) builder_windows: Option<Vec<WindowBuilder>>,
    pub(crate) owner: Owner,
    pub(crate) window_renderer_factory: Box<dyn WindowRendererFactory<WindowRenderer = W>>,
    pub(crate) clipboard_context: Rc<RefCell<ClipboardContext>>,
    pub(crate) can_create_surfaces: bool,
    pub(crate) receiver: FlumeReceiver<EventLoopEvent>,
    pub(crate) sender: flume::Sender<EventLoopEvent>,
    pub(crate) on_event_loop_init: Option<OnEventLoopInitFns>,
    pub(crate) app_event_listeners: AppEventHandlers,
    pub(crate) fut_executor: Option<VelonaTasksExecutor>,
    pub(crate) accesskit_adapter_factory: Option<Box<dyn AdapterFactory>>,
    pub(crate) first_time_ui_event: Option<Instant>,
    pub(crate) spawn_fn: Option<SpawnFn>,
}

// ------- Utilities --------- //
#[cfg_attr(feature = "hotpath", hotpath::measure_all)]
impl<W> App<W>
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
        self.use_window(window_id, |window| fun(&mut window.render_root.tree))
    }
    fn use_window_render_root_ref<F, R>(&mut self, window_id: WindowId, fun: F) -> Option<R>
    where
        F: FnOnce(&RenderRoot) -> R,
    {
        self.use_window_ref(window_id, |window| fun(&window.render_root.tree))
    }
    fn create_window_owner_children(&self, window_id: WindowId) -> Option<Owner> {
        self.use_window_ref(window_id, |window| window.create_children_owner())
    }
    fn create_app_handle(
        &self,
        event_loop: &dyn winit_core::event_loop::ActiveEventLoop,
    ) -> AppHandle {
        AppHandle::new(AppEventLoopProxy::new(
            event_loop.create_proxy(),
            self.sender.clone(),
        ))
    }

    fn on_init(&mut self, event_loop: &(dyn ActiveEventLoop + 'static)) {
        let app_handle = self.create_app_handle(event_loop);

        self.fut_executor = Some({
            let proxy = app_handle.get_proxy().clone();
            VelonaTasksExecutor::new(move |id| {
                let _ = proxy.send_event(EventLoopEvent::PollTask(id));
            })
        });

        #[cfg(feature = "subsecond")]
        {
            use crate::events::el_event::EventLoopEvent;
            // Changes fut
            {
                let proxy = app_handle.get_proxy().clone();
                velona_subsecond::connect_to_dx_cli(move |msg| {
                    let _ = proxy.send_event(EventLoopEvent::DxCliMessages(msg));
                });
            }
        }

        let spawn_fn = self
            .spawn_fn
            .take()
            .unwrap_or_else(|| Box::new(|_| panic!("No spawn_fn provided")));

        match any_spawner::Executor::init_local_custom_executor(super::executor::AppExecutor::new(
            spawn_fn,
            app_handle.get_proxy().clone(),
        )) {
            Ok(_) => {}
            Err(err) => {
                panic!("{err}")
            }
        }

        if let Some(on_init) = self.on_event_loop_init.take() {
            for func in on_init {
                func(&app_handle);
            }
        }
        if let Some(builder_windows) = self.builder_windows.take() {
            if builder_windows.is_empty() {
                log::warn!("No window provided! Exiting...");
                event_loop.exit();
            } else {
                for window in builder_windows {
                    if app_handle
                        .send_event(EventLoopEvent::NewWindow(Box::new(window)))
                        .is_err()
                    {
                        log::warn!("the event loop is already dead lol");
                    }
                }
            }
        }
    }
    #[track_caller]
    fn fut_executor(&mut self) -> &mut VelonaTasksExecutor {
        self.fut_executor.as_mut().expect("The internal task executor must be available. Perhaps `winit` forgot to call the new_event with StartCause::Init")
    }
}

// ------- Window creation -------- //
#[cfg_attr(feature = "hotpath", hotpath::measure_all)]
impl<W> App<W>
where
    W: WindowRenderer,
{
    fn create_window_access_kit_adapter(
        &mut self,
        window: &dyn winit_core::window::Window,
        event_loop: &dyn winit_core::event_loop::ActiveEventLoop,
    ) -> Option<Box<dyn velona_core_accesskit::Adapter>> {
        let app_handle = self
            .create_app_handle(event_loop)
            .get_proxy()
            .accesskit_handler(window.id());
        if let Some(factory) = self.accesskit_adapter_factory.as_mut() {
            factory.create_erased_adapter(CreateAdapterArgs {
                active_event_loop: event_loop,
                window,
                event_handler: Box::new(app_handle),
            })
        } else {
            None
        }
    }
    fn create_window(
        &mut self,
        builder: Box<WindowBuilder>,
        event_loop: &dyn winit_core::event_loop::ActiveEventLoop,
    ) {
        let window_attributes = builder.window_attributes;
        match event_loop.create_window(window_attributes) {
            Ok(window) => {
                let window = window;
                let access_kit = self.create_window_access_kit_adapter(&*window, event_loop);
                match Window::new(WindowNew {
                    window: Arc::new(window),
                    view: builder.view,
                    default_properties: builder
                        .default_propreties
                        .unwrap_or(self.default_properties.clone()),
                    access_kit,
                    app_handle: self.create_app_handle(event_loop),
                    parent_owner: &self.owner,
                    base_color: builder.base_color,
                    factory: &mut *self.window_renderer_factory
                        as &mut dyn WindowRendererFactory<WindowRenderer = W>,
                    size_policy: builder.size_policy,
                    use_system_fonts: builder.use_system_fonts,
                }) {
                    Ok(mut new_instance) => {
                        if !self.can_create_surfaces {
                            self.fut_executor().spawn(new_instance.resume());
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
impl<W> App<W>
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
                return;
            }
        }
        self.app_event_listeners
            .unregister_handler(UnRegisterAppEventHandler {
                handler_id: *handler_id,
                type_: None,
            });
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
impl<W> App<W>
where
    W: WindowRenderer,
{
    fn resume_windows_surfaces(&mut self) {
        for window in self.windows.values_mut() {
            self.fut_executor
                .as_mut()
                .expect("FutExecutor not loaded yet")
                .spawn(window.resume());
        }
    }
    fn suspend_windows_surfaces(&mut self) {
        for window in self.windows.values_mut() {
            window.suspend();
        }
    }
}

// --- Future executor --- //
#[cfg_attr(feature = "hotpath", hotpath::measure_all)]
impl<W> App<W>
where
    W: WindowRenderer,
{
    fn poll_task(&mut self, task_id: TaskId) {
        self.fut_executor().poll_task(task_id);
    }
    fn run_exiting_task(&mut self) -> usize {
        let mut tasks = 0usize;
        while let Some(EventLoopEvent::PollTask(task_id)) = self.receiver.try_iter().next() {
            self.poll_task(task_id);
            tasks += 1;
        }
        tasks
    }
    fn spawn_task(&mut self, task: TaskType<()>) {
        self.fut_executor().spawn(match task {
            TaskType::Send(pin) => pin,
            TaskType::NonSend(pin) => pin,
        });
    }
}

enum TaskType<T> {
    Send(PinnedFuture<T>),
    NonSend(PinnedLocalFuture<T>),
}

// --- manager method handling --- //
#[cfg_attr(feature = "hotpath", hotpath::measure_all)]
impl<W> App<W>
where
    W: WindowRenderer,
{
    fn execute_manager_methods(&self, ev: &dyn ActiveEventLoop, cmd: OtherManagerMethods) {
        match cmd {
            OtherManagerMethods::SetControlFlow(control_flow) => {
                ev.set_control_flow(control_flow);
            }
            OtherManagerMethods::RegisterCustomCursor(custom_cursor_source, sender) => {
                let _ = sender.send(ev.create_custom_cursor(custom_cursor_source));
            }
            OtherManagerMethods::ListenDeviceEventsMode(device_events) => {
                ev.listen_device_events(device_events);
            }
            OtherManagerMethods::SystemTheme(sender) => {
                let _ = sender.send(ev.system_theme());
            }
            OtherManagerMethods::PrimaryMonitor(sender) => {
                let _ = sender.send(ev.primary_monitor());
            }
            OtherManagerMethods::Exit => {
                ev.exit();
            }
            OtherManagerMethods::AvailableMonitors(sender) => {
                let _ = sender.send(ev.available_monitors().collect());
            }
            OtherManagerMethods::OwnedDisplayHandle(sender) => {
                let _ = sender.send(ev.owned_display_handle());
            }
        }
    }
}

// --- Property Stack methods handling --- //
#[cfg_attr(feature = "hotpath", hotpath::measure_all)]
impl<W> App<W>
where
    W: WindowRenderer,
{
    fn handle_property_stack_methods(&mut self, method: PropertyStackMethods) {
        match method.type_ {
            crate::events::property_stack::PropertyStackMethodsType::Add { stack, sender } => {
                if sender.is_canceled() {
                    log::warn!("Receiver already dropped, aborting");
                    return;
                }
                self.use_window_render_root(method.window_id, |rr| {
                    if let Err(err) = sender.send(rr.insert_property_stack(stack)) {
                        log::error!("Cannot send {} to its receiver", err);
                        rr.remove_property_stack(err);
                    }
                });
            }
            crate::events::property_stack::PropertyStackMethodsType::Replace { id, stack } => {
                self.use_window_render_root(method.window_id, |rr| {
                    if rr.has_property_stack(id) {
                        rr.replace_property_stack(id, stack);
                    }
                });
            }
            crate::events::property_stack::PropertyStackMethodsType::Remove { id } => {
                self.use_window_render_root(method.window_id, |rr| {
                    rr.remove_property_stack(id);
                });
            }
            crate::events::property_stack::PropertyStackMethodsType::IsPresent { id, sender } => {
                if sender.is_canceled() {
                    log::warn!("Receiver already dropped, aborting");
                    return;
                }
                self.use_window_render_root(method.window_id, |rr| {
                    if sender.send(rr.has_property_stack(id)).is_err() {
                        log::error!("Cannot send data to its receiver")
                    }
                });
            }
        }
    }
}

// --- WINIT event loop handlers --- //
#[cfg_attr(feature = "hotpath", hotpath::measure_all)]
impl<W> App<W>
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
        _event_loop: &dyn winit_core::event_loop::ActiveEventLoop,
        window_id: WindowId,
        signal: RenderRootSignal,
        to_redraw: &mut HashSet<WindowId>,
    ) {
        let app_handle = self.create_app_handle(_event_loop);

        let event_loop_proxy = app_handle.get_proxy();

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
                    let maybe_request = ImeEnableRequest::new(
                        ImeCapabilities::new()
                            .with_cursor_area()
                            .with_hint_and_purpose(),
                        ImeRequestData::default(),
                    );
                    if let Some(request) = maybe_request {
                        window
                            .winit_window
                            .request_ime_update(winit_core::window::ImeRequest::Enable(request));
                    }
                }
                RenderRootSignal::EndIme => {
                    window
                        .winit_window
                        .request_ime_update(winit_core::window::ImeRequest::Disable);
                }
                RenderRootSignal::ImeMoved(logical_position, logical_size) => {
                    window
                        .winit_window
                        .request_ime_update(winit_core::window::ImeRequest::Update(
                            ImeRequestData::default()
                                .with_cursor_area(logical_position.into(), logical_size.into()),
                        ));
                }
                RenderRootSignal::ClipboardStore(text) => {
                    let _ = event_loop_proxy.send_event(EventLoopEvent::SetClipboardContent(text));
                }
                RenderRootSignal::RequestRedraw => {
                    to_redraw.insert(window_id);
                }
                RenderRootSignal::RequestAnimFrame => {
                    to_redraw.insert(window_id);
                }
                RenderRootSignal::TakeFocus => {
                    window.winit_window.focus_window();
                }
                RenderRootSignal::SetCursor(cursor_icon) => {
                    window.winit_window.set_cursor(cursor_icon.into());
                }
                RenderRootSignal::SetSize(physical_size) => {
                    // TODO handle return value ??
                    let _ = window
                        .winit_window
                        .request_surface_size(physical_size.into());
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
                    window
                        .winit_window
                        .show_window_menu(logical_position.into());
                }
                RenderRootSignal::WidgetSelectedInInspector(widget_id) => {
                    let render_root = &window.render_root.tree;
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
                }
                RenderRootSignal::NewLayer(_type, new_widget, point) => {
                    // TODO implement type
                    window.render_root.tree.add_layer(new_widget, point);
                }
                RenderRootSignal::RemoveLayer(widget_id) => {
                    window.render_root.tree.remove_layer(widget_id);
                }
                RenderRootSignal::RepositionLayer(widget_id, point) => {
                    window.render_root.tree.reposition_layer(widget_id, point);
                }
            }
        });
    }

    fn handle_app_events(&mut self, event_loop: &dyn ActiveEventLoop) {
        let mut need_redraw = HashSet::<WindowId>::default();
        while let Some(event) = self.receiver.try_iter().next() {
            match event {
                EventLoopEvent::AccessKitAction(event) => {
                    self.use_window(event.window_id, |window| match event.window_event {
                        velona_core_accesskit::WindowEvent::InitialTreeRequested => {
                            window
                                .render_root
                                .tree
                                .handle_window_event(MasonryWindowEvent::EnableAccessTree);
                        }
                        velona_core_accesskit::WindowEvent::ActionRequested(action_request) => {
                            window.render_root.tree.handle_access_event(action_request);
                        }
                        velona_core_accesskit::WindowEvent::AccessibilityDeactivated => {
                            window
                                .render_root
                                .tree
                                .handle_window_event(MasonryWindowEvent::DisableAccessTree);
                        }
                    });
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
                    self.handle_signal(event_loop, window_id, signal.take(), &mut need_redraw);
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
                        (use_winit_window_on_main.use_fn)(&**window.winit_window);
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
                EventLoopEvent::ManagerMethods(cmd) => {
                    self.execute_manager_methods(event_loop, *cmd);
                }
                EventLoopEvent::PropertyStack(property_stack_methods) => {
                    self.handle_property_stack_methods(*property_stack_methods);
                }
                EventLoopEvent::PollTask(task_id) => {
                    self.poll_task(task_id);
                }
                EventLoopEvent::SpawnTaskLocal(send_wrapper) => {
                    self.spawn_task(TaskType::NonSend(send_wrapper.take()));
                }
                EventLoopEvent::SpawnTask(pin) => {
                    self.spawn_task(TaskType::Send(pin));
                }
                #[cfg(feature = "subsecond")]
                EventLoopEvent::DxCliMessages(msg) => match msg {
                    velona_subsecond::DevserverMsg::HotReload(hot_reload_msg) => {
                        if let Some(jump_table) = hot_reload_msg.jump_table.as_ref().cloned()
                            && hot_reload_msg.for_build_id == Some(dioxus_cli_config::build_id())
                        {
                            let our_pid = if cfg!(target_family = "wasm") {
                                None
                            } else {
                                Some(std::process::id())
                            };

                            if hot_reload_msg.for_pid == our_pid {
                                let res = unsafe { subsecond::apply_patch(jump_table) };
                                if let Err(err) = res {
                                    log::error!("cannot hotreload {err}");
                                }
                            }
                        }
                    }
                    velona_subsecond::DevserverMsg::FullReloadCommand => {
                        log::info!("Should reload but it can't hehe...")
                    }
                    velona_subsecond::DevserverMsg::Shutdown => {
                        event_loop.exit();
                    }
                    _ => {}
                },
                EventLoopEvent::PollAll => {
                    let res = self.fut_executor().poll_all();
                    log::trace!("{:#?}", res);
                }
                EventLoopEvent::ManagerActions(manager_erased_action) => {
                    self.app_event_listeners
                        .emit(EmitAppEventToHandlers::ErasedAction(&manager_erased_action));
                }
            }
        }
        for window_id in need_redraw {
            self.use_window(window_id, |window| {
                window.winit_window.request_redraw();
            });
        }
    }
}

impl<W> Drop for App<W>
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
impl<W> ApplicationHandler for App<W>
where
    W: WindowRenderer,
{
    fn can_create_surfaces(&mut self, _event_loop: &dyn ActiveEventLoop) {
        self.can_create_surfaces = true;
        self.resume_windows_surfaces();
    }
    fn destroy_surfaces(&mut self, _event_loop: &dyn ActiveEventLoop) {
        self.can_create_surfaces = false;
        self.suspend_windows_surfaces();
    }

    fn new_events(
        &mut self,
        event_loop: &dyn winit_core::event_loop::ActiveEventLoop,
        cause: winit_core::event::StartCause,
    ) {
        if cause == winit_core::event::StartCause::Init {
            self.on_init(event_loop);
        }
    }
    fn resumed(&mut self, _event_loop: &dyn winit_core::event_loop::ActiveEventLoop) {
        self.resume_windows_surfaces();
        self.app_event_listeners
            .emit(EmitAppEventToHandlers::Resumed);
    }

    fn window_event(
        &mut self,
        event_loop: &dyn winit_core::event_loop::ActiveEventLoop,
        window_id: WindowId,
        event: winit_core::event::WindowEvent,
    ) {
        // #[cfg(feature = "hotpath")]
        // hotpath::dbg!((&window_id, &event));
        self.use_window(window_id, |window| {
            window.forward_to_accesskit_adapter(&event);
        });
        let clipboard_context = self.clipboard_context.clone();
        let first_time_ui_event = self.first_time_ui_event.clone();
        let maybe_first_time = self
            .use_window(window_id, |window| {
                handle_ui_translated_event(first_time_ui_event, &event, clipboard_context, window)
            })
            .flatten();
        if let Some(first_time) = maybe_first_time {
            self.first_time_ui_event.replace(first_time);
        }
        match event {
            WindowEvent::Destroyed if self.windows.is_empty() => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.handle_redraw_request(window_id);
            }
            WindowEvent::SurfaceResized(size) => {
                self.handle_resize_event(window_id, size);
            }
            WindowEvent::CloseRequested => {
                self.windows.remove(&window_id);
            }
            WindowEvent::Ime(ime) => {
                if let Some(ime) = winit_ime_to_masonry(ime) {
                    self.use_window_render_root(window_id, |render_root| {
                        render_root.handle_text_event(masonry_core::core::TextEvent::Ime(ime));
                    });
                }
            }
            WindowEvent::ScaleFactorChanged {
                scale_factor,
                // TODO use this??
                ..
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
    fn memory_warning(&mut self, _event_loop: &dyn winit_core::event_loop::ActiveEventLoop) {
        self.windows.shrink_to_fit();
        self.windows
            .values_mut()
            .for_each(|w| w.on_memory_warning());
        self.app_event_listeners
            .emit(EmitAppEventToHandlers::MemoryWarning);
        self.app_event_listeners.shrink_to_fit();
        self.fut_executor().shrink_to_fit();
    }
    fn suspended(&mut self, _event_loop: &dyn winit_core::event_loop::ActiveEventLoop) {
        self.app_event_listeners
            .emit(EmitAppEventToHandlers::Suspended);
    }
    fn proxy_wake_up(&mut self, event_loop: &dyn winit_core::event_loop::ActiveEventLoop) {
        // #[cfg(feature = "hotpath")]
        // hotpath::dbg!(&event);
        self.handle_app_events(event_loop);
    }

    fn device_event(
        &mut self,
        _event_loop: &dyn ActiveEventLoop,
        device_id: Option<winit_core::event::DeviceId>,
        event: winit_core::event::DeviceEvent,
    ) {
        if let Some(device_id) = device_id {
            self.app_event_listeners
                .emit(EmitAppEventToHandlers::Device(device_id, &event));
        }
    }
}

fn handle_ui_translated_event<W: WindowRenderer>(
    mut first_time_stamp: Option<Instant>,
    event: &WindowEvent,
    clipboard_context: Rc<RefCell<copypasta::x11_clipboard::X11ClipboardContext>>,
    window: &mut Window<W>,
) -> Option<Instant> {
    if !matches!(
        *event,
        WindowEvent::KeyboardInput {
            is_synthetic: true,
            ..
        }
    ) {
        let time = Instant::now()
            .duration_since(*first_time_stamp.get_or_insert_with(Instant::now))
            .as_nanos() as u64;

        if let Some(wet) =
            window
                .event_reducer
                .reduce(window.winit_window.scale_factor(), event, time)
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
                        match clipboard_context.borrow_mut().get_contents() {
                            Ok(content) => {
                                window
                                    .render_root
                                    .tree
                                    .handle_text_event(TextEvent::ClipboardPaste(content));
                                todo_warn_of_something("Clipboard Paste");
                            }
                            Err(err) => {
                                log::error!("Cannot get clipboard content: {err}")
                            }
                        }
                    } else {
                        window
                            .render_root
                            .tree
                            .handle_text_event(masonry_core::core::TextEvent::Keyboard(k));
                    }
                }
                WindowEventTranslation::Pointer(p) => {
                    window.render_root.tree.handle_pointer_event(p);
                }
            }
        }
    }
    first_time_stamp
}
