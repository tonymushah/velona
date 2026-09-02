use std::fmt::Debug;

use any_spawner::PinnedFuture;
use any_spawner::PinnedLocalFuture;
use futures_channel::oneshot;
use masonry_core::app::RenderRoot;
use masonry_core::app::RenderRootSignal;
use reactive_graph::owner::Owner;
use send_wrapper::SendWrapper;
use velona_executor::TaskId;
use winit::window::{Window, WindowId};

use crate::app::event_listener::{RegisterAppEvent, UnRegisterAppEventHandler};
use crate::events::property_stack::PropertyStackMethods;
use crate::manager::OtherManagerMethods;
use crate::window::event_listener::{RegisterWindowEventHandler, UnregisterWindowEventHandlerType};
use crate::{
    widget_ref::{EditWidgetFnEvent, UseWidgetFnEvent},
    window::builder::WindowBuilder,
    window::event_listener::HandlerId,
};

pub(crate) struct UseWindowRenderRootOnMain {
    pub(crate) window_id: WindowId,
    pub(crate) use_fn: Box<dyn FnOnce(&mut RenderRoot) + Send>,
}

impl Debug for UseWindowRenderRootOnMain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UseWindowRenderRootOnMain")
            .field("window_id", &self.window_id)
            .field("use_fn", &"fn ()")
            .finish()
    }
}

pub(crate) struct UseWinitWindowOnMain {
    pub(crate) window_id: WindowId,
    pub(crate) use_fn: Box<dyn FnOnce(&Window) + Send>,
}

impl Debug for UseWinitWindowOnMain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UseWinitWindowOnMain")
            .field("window_id", &self.window_id)
            .field("use_fn", &"fn ()")
            .finish()
    }
}

#[derive(Debug)]
pub(crate) struct GetWindowChildReactiveOwner {
    pub(crate) window_id: WindowId,
    pub(crate) sender: oneshot::Sender<Owner>,
}

#[derive(Debug)]
pub(crate) struct GetAppChildReactiveOwner {
    pub(crate) sender: oneshot::Sender<Owner>,
}

#[derive(Debug)]
pub(crate) enum RegisterEventHandler {
    App(RegisterAppEvent),
    Window {
        window_id: WindowId,
        type_: RegisterWindowEventHandler,
    },
}

#[derive(Debug)]
pub(crate) enum UnregisterEventHandler {
    Any(HandlerId),
    App(UnRegisterAppEventHandler),
    Window {
        window_id: WindowId,
        handler_id: HandlerId,
        type_: Option<UnregisterWindowEventHandlerType>,
    },
}

#[derive(derive_more::Debug)]
pub(crate) enum EventLoopEvent {
    AccessKitAction(Box<accesskit_winit::Event>),
    NewWindow(#[debug(skip)] Box<WindowBuilder>),
    CloseWindow(WindowId),
    SetClipboardContent(String),
    HandleRenderRootSignals(WindowId, Box<SendWrapper<RenderRootSignal>>),
    EditWidget(Box<EditWidgetFnEvent>),
    UseWidget(Box<UseWidgetFnEvent>),
    UseWindowRenderRoot(Box<UseWindowRenderRootOnMain>),
    UseWinitWindow(Box<UseWinitWindowOnMain>),
    GetWindowChildReactiveOwner(Box<GetWindowChildReactiveOwner>),
    GetAppChildReactiveOwner(Box<GetAppChildReactiveOwner>),
    RegisterHandler(Box<RegisterEventHandler>),
    UnRegisterHandler(Box<UnregisterEventHandler>),
    ManagerMethods(Box<OtherManagerMethods>),
    PropertyStack(Box<PropertyStackMethods>),
    PollTask(TaskId),
    PollAll,
    SpawnTaskLocal(#[debug(skip)] SendWrapper<PinnedLocalFuture<()>>),
    SpawnTask(#[debug(skip)] PinnedFuture<()>),
    #[cfg(feature = "subsecond")]
    DxCliMessages(velona_subsecond::DevserverMsg),
}

impl From<PropertyStackMethods> for EventLoopEvent {
    fn from(value: PropertyStackMethods) -> Self {
        Self::PropertyStack(Box::new(value))
    }
}

impl From<accesskit_winit::Event> for EventLoopEvent {
    fn from(value: accesskit_winit::Event) -> Self {
        Self::AccessKitAction(Box::new(value))
    }
}
