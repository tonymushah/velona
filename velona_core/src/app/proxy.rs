use std::{fmt::Debug, sync::Arc};

use masonry_core::accesskit;
use thiserror::Error;
use winit::{event_loop::EventLoopProxy, window::WindowId};

use crate::{app::EventLoopEvent, utils::FlumeSender};

#[derive(derive_more::Debug, Clone)]
pub struct AppEventLoopProxy {
    winit_proxy: Arc<dyn WinitEventLoopProxy>,
    #[cfg_attr(feature = "hotpath", debug(ignore))]
    send: FlumeSender<EventLoopEvent>,
}

#[derive(Debug, thiserror::Error)]
#[error("The event loop has already exited")]
pub struct EventLoopExisted;

/// A trait from [`winit`] 0.31
/// to ease the migration once it is released
pub(crate) trait WinitEventLoopProxy: Debug + Send + Sync {
    fn wake_up(&self) -> Result<(), EventLoopExisted>;
}

impl WinitEventLoopProxy for EventLoopProxy<()> {
    fn wake_up(&self) -> Result<(), EventLoopExisted> {
        self.send_event(()).map_err(|_| EventLoopExisted)
    }
}

impl From<EventLoopExisted> for AppProxySendError {
    fn from(_: EventLoopExisted) -> Self {
        Self::EventLoopExited
    }
}

#[derive(Debug, Error)]
pub(crate) enum AppProxySendError {
    #[error("The mpsc receiver has already closed")]
    ClosedChannel(EventLoopEvent),
    #[error("The event loop already ended")]
    EventLoopExited,
}

impl AppEventLoopProxy {
    pub fn new<T>(winit_proxy: T, send: FlumeSender<EventLoopEvent>) -> Self
    where
        T: WinitEventLoopProxy + 'static,
    {
        Self {
            winit_proxy: Arc::new(winit_proxy),
            send,
        }
    }
    pub fn send_event(&self, event: EventLoopEvent) -> Result<(), AppProxySendError> {
        self.send
            .send(event)
            .map_err(|err| AppProxySendError::ClosedChannel(err.0))?;
        self.winit_proxy.wake_up()?;
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

pub(crate) trait EventProxyHandle {
    fn get_proxy(&self) -> &AppEventLoopProxy;
    fn send_event(&self, event: EventLoopEvent) -> Result<(), AppProxySendError> {
        self.get_proxy().send_event(event)
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
