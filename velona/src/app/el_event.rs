use std::fmt::Debug;

use async_task::Runnable;
use masonry::app::RenderRootSignal;
use send_wrapper::SendWrapper;
use winit::{
    event_loop::{EventLoopClosed, EventLoopProxy},
    window::WindowId,
};

use crate::{
    app::executor::AppTaskProxy,
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

pub(crate) type AppEventLoopProxy = EventLoopProxy<EventLoopEvent>;

impl From<accesskit_winit::Event> for EventLoopEvent {
    fn from(value: accesskit_winit::Event) -> Self {
        Self::AccessKitAction(Box::new(value))
    }
}

pub(crate) trait EventProxyHandle {
    fn get_proxy(&self) -> &AppTaskProxy;
    fn send_event(&self, event: EventLoopEvent) -> Result<(), EventLoopClosed<EventLoopEvent>> {
        self.get_proxy().send_event(event)
    }
}

#[cfg(test)]
mod test {

    use crate::{app::AppEventLoopProxy, utils::is_send_sync};

    #[test]
    fn test_if_event_proxy_is_send_sync() {
        is_send_sync::<AppEventLoopProxy>();
    }
}
