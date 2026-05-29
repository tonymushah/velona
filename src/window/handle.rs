use std::sync::Weak;

use winit::window::Window;

use crate::app::{self, AppEventLoopProxy, AppHandle};

#[derive(Debug, Clone)]
pub struct WindowHandle {
    pub(crate) window: Weak<Window>,
    pub(crate) app_handle: AppHandle,
}

impl WindowHandle {
    fn use_raw_window<F, O>(&self, to_use: F) -> Option<O>
    where
        F: FnOnce(&Window) -> O,
    {
        self.window.upgrade().map(|window| to_use(&window))
    }
    pub fn request_redraw(&self) {
        self.use_raw_window(|window| {
            window.request_redraw();
        });
    }
    pub fn set_title(&self, title: &str) {
        self.use_raw_window(|window| {
            window.set_title(title);
        });
    }
}

impl app::el_event::EventProxyHandle for WindowHandle {
    fn get_proxy(&self) -> &AppEventLoopProxy {
        self.app_handle.get_proxy()
    }
}
