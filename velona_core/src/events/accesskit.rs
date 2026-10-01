use accesskit_xplat::WindowEvent;
use winit_core::window::WindowId;

#[derive(Debug)]
pub struct AccessKitWindowEvent {
    pub window_id: WindowId,
    pub window_event: WindowEvent,
}
