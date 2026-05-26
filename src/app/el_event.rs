use winit::{event_loop::EventLoopProxy, window::WindowId};

use crate::window::builder::WindowBuilder;

pub(crate) enum EventLoopEvent {
    AccessKitAction(Box<accesskit_winit::Event>),
    RunTask(Box<async_task::Runnable>),
    NewWindow(Box<WindowBuilder>),
    CloseWindow(WindowId),
    SetClipboardContent(String),
}

pub(crate) type AppEventLoopProxy = EventLoopProxy<EventLoopEvent>;

impl From<accesskit_winit::Event> for EventLoopEvent {
    fn from(value: accesskit_winit::Event) -> Self {
        Self::AccessKitAction(Box::new(value))
    }
}

#[cfg(test)]
mod test {
    use crate::utils::is_send_sync;

    #[test]
    fn test_if_event_is_send_sync() {
        is_send_sync::<super::EventLoopEvent>();
    }
}
