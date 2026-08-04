// TODO finish implementation

use std::sync::mpsc::Receiver;

use crate::{
    app::{
        EventLoopEvent,
        proxy::{AppEventLoopProxy, WinitEventLoopProxy},
    },
    window::handle::WindowHandle,
};

#[derive(Debug)]
struct DummyEventLoopProxy;

impl WinitEventLoopProxy for DummyEventLoopProxy {
    fn wake_up(&self) -> Result<(), crate::app::proxy::EventLoopExisted> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct TestingWindow {
    proxy: AppEventLoopProxy,
    receiver: Receiver<EventLoopEvent>,
    window_handle: WindowHandle,
}
