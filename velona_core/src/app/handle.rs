use reactive_graph::owner::use_context;
use winit::event::{DeviceEvent, DeviceId};

use crate::{
    Manager,
    app::{
        EventLoopEvent,
        event_listener::{RegisterAppEvent, RegisterAppEventType},
        proxy::{AppEventLoopProxy, AppProxySendError, EventProxyHandle},
    },
    events::el_event::{RegisterEventHandler, UnregisterEventHandler},
    utils::HandlerId,
};

#[derive(Debug, thiserror::Error)]
pub enum AppHandleActionError {
    #[error("The app has already exited")]
    AppExited,
}

impl From<AppProxySendError> for AppHandleActionError {
    fn from(_: AppProxySendError) -> Self {
        Self::AppExited
    }
}

#[derive(Debug, Clone)]
pub struct AppHandle {
    event_proxy: AppEventLoopProxy,
}

impl AppHandle {
    pub(crate) fn new(proxy: AppEventLoopProxy) -> AppHandle {
        AppHandle { event_proxy: proxy }
    }
}

impl AppHandle {
    /// Register an event listener that will listen to any [device event](winit::application::ApplicationHandler::device_event).
    ///
    /// Worth noting that this listener will not run inside of the current context owner.
    pub fn register_device_event_listener<L>(
        &self,
        listener: L,
    ) -> Result<HandlerId, AppHandleActionError>
    where
        L: Fn(DeviceId, &DeviceEvent) + Send + 'static,
    {
        let handler_id = HandlerId::next();
        self.send_event(EventLoopEvent::RegisterHandler(Box::new(
            RegisterEventHandler::App(RegisterAppEvent {
                handler_id,
                type_: RegisterAppEventType::Device(Box::new(listener)),
            }),
        )))?;
        Ok(handler_id)
    }
}

impl EventProxyHandle for AppHandle {
    fn get_proxy(&self) -> &AppEventLoopProxy {
        &self.event_proxy
    }
}

impl Manager for AppHandle {}

/// Get the current app handle.
pub fn use_app_handle() -> Option<AppHandle> {
    use_context()
}

#[cfg(test)]
mod tests {
    use crate::utils::is_send_sync;

    #[test]
    fn is_app_handle_send_sync() {
        is_send_sync::<super::AppHandle>();
    }
}
