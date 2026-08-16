use reactive_graph::owner::use_context;

use crate::{
    Manager,
    app::{
        EventLoopEvent,
        proxy::{AppEventLoopProxy, AppProxySendError, EventProxyHandle},
    },
    events::el_event::UnregisterEventHandler,
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
