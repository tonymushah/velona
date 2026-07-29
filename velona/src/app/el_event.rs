use std::fmt::Debug;

use async_task::Runnable;
use masonry::app::RenderRootSignal;
use send_wrapper::SendWrapper;
use winit::window::WindowId;

use crate::{
    app::proxy::{AppEventLoopProxy, AppProxySendError},
    widget_ref::{EditWidgetFnEvent, UseWidgetFnEvent},
    window::builder::WindowBuilder,
};

pub(crate) enum EventLoopEvent {
    AccessKitAction(Box<accesskit_winit::Event>),
    RunTask(Runnable),
    NewWindow(Box<WindowBuilder>),
    CloseWindow(WindowId),
    SetClipboardContent(String),
    HandleRenderRootSignals(WindowId, Box<SendWrapper<RenderRootSignal>>),
    EditWidget(Box<EditWidgetFnEvent>),
    UseWidget(Box<UseWidgetFnEvent>),
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
