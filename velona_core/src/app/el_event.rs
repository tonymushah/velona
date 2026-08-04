use std::fmt::Debug;

use async_task::Runnable;
use masonry_core::app::RenderRoot;
use masonry_core::{app::RenderRootSignal, core::WidgetId};
use send_wrapper::SendWrapper;
use winit::window::{Window, WindowId};

use crate::{
    app::proxy::{AppEventLoopProxy, AppProxySendError},
    widget_ref::{EditWidgetFnEvent, UseWidgetFnEvent},
    window::builder::WindowBuilder,
    window_event_handler::{HandlerFn, HandlerId, NoParamHandlerFn},
};

pub(crate) struct RegisterWidgetActionHandler {
    pub(crate) handler_id: HandlerId,
    pub(crate) window_id: WindowId,
    pub(crate) widget_id: WidgetId,
    pub(crate) handler_fn: HandlerFn,
}

impl Debug for RegisterWidgetActionHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegisterWidgetActionHandler")
            .field("handler_id", &self.handler_id)
            .field("widget_id", &self.widget_id)
            .field("handler_fn", &"fn ()")
            .finish()
    }
}

#[derive(Debug)]
pub(crate) struct UnregisterHandler {
    pub(crate) handler_id: HandlerId,
    pub(crate) window_id: Option<WindowId>,
}

pub(crate) struct RegisterOnWindowDestroyHandler {
    pub(crate) handler_id: HandlerId,
    pub(crate) window_id: WindowId,
    pub(crate) handler: NoParamHandlerFn,
}

impl Debug for RegisterOnWindowDestroyHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegisterOnWindowDestroyHandler")
            .field("handler_id", &self.handler_id)
            .field("window_id", &self.window_id)
            .field("handler", &"fn ()")
            .finish()
    }
}

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

pub(crate) enum EventLoopEvent {
    AccessKitAction(Box<accesskit_winit::Event>),
    RunTask(Runnable),
    NewWindow(Box<WindowBuilder>),
    CloseWindow(WindowId),
    SetClipboardContent(String),
    HandleRenderRootSignals(WindowId, Box<SendWrapper<RenderRootSignal>>),
    EditWidget(Box<EditWidgetFnEvent>),
    UseWidget(Box<UseWidgetFnEvent>),
    RegisterWidgetActionHandler(Box<RegisterWidgetActionHandler>),
    UnregisterEventHandler(Box<UnregisterHandler>),
    RegisterOnWindowDestroy(Box<RegisterOnWindowDestroyHandler>),
    UseWindowRenderRoot(Box<UseWindowRenderRootOnMain>),
    UseWinitWindow(Box<UseWinitWindowOnMain>),
}

impl Debug for EventLoopEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AccessKitAction(arg0) => f.debug_tuple("AccessKitAction").field(arg0).finish(),
            Self::RunTask(_) => write!(f, "RunTasks"),
            Self::NewWindow(_) => f.debug_tuple("NewWindow").finish(),
            Self::CloseWindow(arg0) => f.debug_tuple("CloseWindow").field(arg0).finish(),
            Self::SetClipboardContent(arg0) => {
                f.debug_tuple("SetClipboardContent").field(arg0).finish()
            }
            Self::HandleRenderRootSignals(id, _) => f
                .debug_tuple("HandleRenderRootSignals")
                .field(id)
                .field(&"NonSend")
                .finish(),
            Self::EditWidget(arg0) => f.debug_tuple("EditWidget").field(arg0).finish(),
            Self::UseWidget(arg0) => f.debug_tuple("UseWidget").field(arg0).finish(),
            Self::RegisterWidgetActionHandler(arg0) => f
                .debug_tuple("RegisterWidgetActionHandler")
                .field(arg0)
                .finish(),
            Self::UnregisterEventHandler(arg0) => {
                f.debug_tuple("UnregisterEventHandler").field(arg0).finish()
            }
            Self::RegisterOnWindowDestroy(arg0) => f
                .debug_tuple("RegisterOnWindowDestroy")
                .field(arg0)
                .finish(),
            Self::UseWindowRenderRoot(arg0) => {
                f.debug_tuple("UseWindowRenderRoot").field(arg0).finish()
            }
            Self::UseWinitWindow(arg0) => f.debug_tuple("UseWinitWindow").field(arg0).finish(),
        }
    }
}

impl From<accesskit_winit::Event> for EventLoopEvent {
    fn from(value: accesskit_winit::Event) -> Self {
        Self::AccessKitAction(Box::new(value))
    }
}

pub(crate) trait EventProxyHandle {
    fn get_proxy(&self) -> &AppEventLoopProxy;
    fn send_event(&self, event: EventLoopEvent) -> Result<(), AppProxySendError> {
        self.get_proxy().send_event(event)
    }
}
