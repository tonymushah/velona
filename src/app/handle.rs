use reactive_graph::owner::use_context;

use crate::app::AppEventLoopProxy;

#[derive(Debug, Clone)]
pub struct AppHandle {
    event_proxy: AppEventLoopProxy,
}

impl AppHandle {
    pub(crate) fn new(proxy: AppEventLoopProxy) -> AppHandle {
        AppHandle { event_proxy: proxy }
    }
}

impl super::el_event::EventProxyHandle for AppHandle {
    fn get_proxy(&self) -> &AppEventLoopProxy {
        &self.event_proxy
    }
}

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
