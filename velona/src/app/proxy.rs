use std::sync::mpsc;

use masonry::accesskit;
use thiserror::Error;
use winit::{event_loop::EventLoopProxy, window::WindowId};

use crate::app::EventLoopEvent;

#[derive(Debug, Clone)]
pub struct AppEventLoopProxy {
    winit_proxy: EventLoopProxy<()>,
    send: mpsc::Sender<EventLoopEvent>,
}

#[derive(Debug, Error)]
pub enum AppProxySendError {
    #[error("The mpsc receiver has already closed")]
    ClosedChannel(EventLoopEvent),
    #[error("The event loop already ended")]
    EventLoopExited,
}

impl AppEventLoopProxy {
    pub fn new(winit_proxy: EventLoopProxy<()>, send: mpsc::Sender<EventLoopEvent>) -> Self {
        Self { winit_proxy, send }
    }
    pub fn send_event(&self, event: EventLoopEvent) -> Result<(), AppProxySendError> {
        self.send
            .send(event)
            .map_err(|err| AppProxySendError::ClosedChannel(err.0))?;
        self.winit_proxy
            .send_event(())
            .map_err(|_| AppProxySendError::EventLoopExited)?;
        Ok(())
    }
    pub fn accesskit_handler(&self, window_id: WindowId) -> AccessKitAppEventLoopProxy {
        AccessKitAppEventLoopProxy {
            window_id,
            proxy: self.clone(),
        }
    }
}

pub struct AccessKitAppEventLoopProxy {
    window_id: WindowId,
    proxy: AppEventLoopProxy,
}

impl accesskit::ActivationHandler for AccessKitAppEventLoopProxy {
    fn request_initial_tree(&mut self) -> Option<accesskit::TreeUpdate> {
        self.proxy
            .send_event(
                accesskit_winit::Event {
                    window_id: self.window_id,
                    window_event: accesskit_winit::WindowEvent::InitialTreeRequested,
                }
                .into(),
            )
            .ok();
        None
    }
}

impl accesskit::ActionHandler for AccessKitAppEventLoopProxy {
    fn do_action(&mut self, request: accesskit::ActionRequest) {
        self.proxy
            .send_event(
                accesskit_winit::Event {
                    window_id: self.window_id,
                    window_event: accesskit_winit::WindowEvent::ActionRequested(request),
                }
                .into(),
            )
            .ok();
    }
}

impl accesskit::DeactivationHandler for AccessKitAppEventLoopProxy {
    fn deactivate_accessibility(&mut self) {
        self.proxy
            .send_event(
                accesskit_winit::Event {
                    window_id: self.window_id,
                    window_event: accesskit_winit::WindowEvent::AccessibilityDeactivated,
                }
                .into(),
            )
            .ok();
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::is_send_sync;

    use super::*;

    #[test]
    fn test_send_sync() {
        is_send_sync::<AppEventLoopProxy>();
    }
}
