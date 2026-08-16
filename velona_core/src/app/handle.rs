use reactive_graph::owner::use_context;
use winit::event::{DeviceEvent, DeviceId};

use crate::{
    Manager,
    app::{
        EventLoopEvent,
        event_listener::{RegisterAppEvent, RegisterAppEventType, UnRegisterAppEventHandler},
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
    pub fn unregister_device_event_listener(
        &self,
        handler_id: HandlerId,
    ) -> Result<(), AppHandleActionError> {
        self.send_event(EventLoopEvent::UnRegisterHandler(Box::new(
            UnregisterEventHandler::App(UnRegisterAppEventHandler {
                handler_id,
                type_: Some(super::event_listener::UnRegisterAppEventType::Device),
            }),
        )))?;
        Ok(())
    }
    /// Register a callback that will run when a [memory warning](winit::application::ApplicationHandler::memory_warning) is emitted.
    ///
    /// Worth noting that this listener will not run inside of the current context owner.
    pub fn register_memory_warning_event_listener<L>(
        &self,
        listener: L,
    ) -> Result<HandlerId, AppHandleActionError>
    where
        L: Fn() + Send + 'static,
    {
        let handler_id = HandlerId::next();
        self.send_event(EventLoopEvent::RegisterHandler(Box::new(
            RegisterEventHandler::App(RegisterAppEvent {
                handler_id,
                type_: RegisterAppEventType::MemoryWarning(Box::new(listener)),
            }),
        )))?;
        Ok(handler_id)
    }
    pub fn unregister_memory_warning_listener(
        &self,
        handler_id: HandlerId,
    ) -> Result<(), AppHandleActionError> {
        self.send_event(EventLoopEvent::UnRegisterHandler(Box::new(
            UnregisterEventHandler::App(UnRegisterAppEventHandler {
                handler_id,
                type_: Some(super::event_listener::UnRegisterAppEventType::MemoryWarning),
            }),
        )))?;
        Ok(())
    }
    /// Register a callback that will run when the app [resumes](winit::application::ApplicationHandler::resumed) its execution.
    ///
    /// Worth noting that this listener will not run inside of the current context owner.
    pub fn register_app_resumed_event_listener<L>(
        &self,
        listener: L,
    ) -> Result<HandlerId, AppHandleActionError>
    where
        L: Fn() + Send + 'static,
    {
        let handler_id = HandlerId::next();
        self.send_event(EventLoopEvent::RegisterHandler(Box::new(
            RegisterEventHandler::App(RegisterAppEvent {
                handler_id,
                type_: RegisterAppEventType::Resumed(Box::new(listener)),
            }),
        )))?;
        Ok(handler_id)
    }
    pub fn unregister_app_resumed_warning_listener(
        &self,
        handler_id: HandlerId,
    ) -> Result<(), AppHandleActionError> {
        self.send_event(EventLoopEvent::UnRegisterHandler(Box::new(
            UnregisterEventHandler::App(UnRegisterAppEventHandler {
                handler_id,
                type_: Some(super::event_listener::UnRegisterAppEventType::Resumed),
            }),
        )))?;
        Ok(())
    }
    /// Register a callback that will run when the app got [suspended](winit::application::ApplicationHandler::suspended).
    ///
    /// Worth noting that this listener will not run inside of the current context owner.
    pub fn register_app_suspended_event_listener<L>(
        &self,
        listener: L,
    ) -> Result<HandlerId, AppHandleActionError>
    where
        L: Fn() + Send + 'static,
    {
        let handler_id = HandlerId::next();
        self.send_event(EventLoopEvent::RegisterHandler(Box::new(
            RegisterEventHandler::App(RegisterAppEvent {
                handler_id,
                type_: RegisterAppEventType::Suspended(Box::new(listener)),
            }),
        )))?;
        Ok(handler_id)
    }
    pub fn unregister_app_suspended_listener(
        &self,
        handler_id: HandlerId,
    ) -> Result<(), AppHandleActionError> {
        self.send_event(EventLoopEvent::UnRegisterHandler(Box::new(
            UnregisterEventHandler::App(UnRegisterAppEventHandler {
                handler_id,
                type_: Some(super::event_listener::UnRegisterAppEventType::Suspended),
            }),
        )))?;
        Ok(())
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
