use accesskit::{ActionRequest, TreeUpdate};
use winit_core::{event_loop::ActiveEventLoop, window::Window};

#[derive(Clone, Debug, PartialEq)]
pub enum WindowEvent {
    InitialTreeRequested,
    ActionRequested(ActionRequest),
    AccessibilityDeactivated,
}

pub trait EventHandler: Send + Sync + 'static {
    fn handle_accesskit_event(&self, event: WindowEvent);
}

pub trait Adapter {
    fn handle_winit_window_event(&mut self, event: &winit_core::event::WindowEvent);
    fn update_if_active(&mut self, updater: Box<dyn FnOnce() -> TreeUpdate>);
}

pub struct CreateAdapterArgs<'a> {
    pub active_event_loop: &'a dyn ActiveEventLoop,
    pub window: &'a dyn Window,
    pub event_handler: Box<dyn EventHandler>,
}

pub trait AdapterFactory {
    type Adapter: Adapter
    where
        Self: Sized;
    fn create_adapter(&mut self, args: CreateAdapterArgs<'_>) -> Option<Self::Adapter>
    where
        Self: Sized;
    fn create_erased_adapter(&mut self, args: CreateAdapterArgs<'_>) -> Option<Box<dyn Adapter>>;
}
