use async_task::Task;
use futures_channel::oneshot;
use winit::{
    event_loop::{ControlFlow, DeviceEvents, OwnedDisplayHandle},
    monitor::MonitorHandle,
    window::{CustomCursor, CustomCursorSource},
};

use crate::{
    WindowBuilder,
    app::{AppHandle, AppHandleActionError, EventLoopEvent, proxy::EventProxyHandle},
    events::el_event::{GetAppChildReactiveOwner, UnregisterEventHandler},
    utils::HandlerId,
    window::handle::WindowHandle,
};

#[derive(Debug, thiserror::Error)]
pub enum CreateWindowError {
    #[error("The app is already closed or exiting")]
    AppAlreadyClosed,
    // TODO implement this properly
    #[error("Cannot create window because of other error")]
    OtherError,
}

#[derive(Debug)]
pub(crate) enum OtherManagerMethods {
    SetControlFlow(ControlFlow),
    RegisterCustomCursor(CustomCursorSource, oneshot::Sender<CustomCursor>),
    ListenDeviceEventsMode(DeviceEvents),
    SystemTheme(oneshot::Sender<Option<winit::window::Theme>>),
    PrimaryMonitor(oneshot::Sender<Option<MonitorHandle>>),
    Exit,
    AvailableMonitors(oneshot::Sender<Vec<MonitorHandle>>),
    OwnedDisplayHandle(oneshot::Sender<OwnedDisplayHandle>),
}

#[allow(private_bounds)]
pub trait Manager: EventProxyHandle {
    fn app_handle(&self) -> AppHandle {
        AppHandle::new(self.get_proxy().clone())
    }
    /// Create a new window
    fn create_window(
        &self,
        mut builder: WindowBuilder,
    ) -> impl Future<Output = Result<WindowHandle, CreateWindowError>> + Send + 'static {
        let (send, receiver) = oneshot::channel::<WindowHandle>();
        let proxy = self.get_proxy().clone();
        builder.window_handle_send = Some(send);
        async move {
            if proxy
                .send_event(EventLoopEvent::NewWindow(Box::new(builder)))
                .is_err()
            {
                return Err(CreateWindowError::AppAlreadyClosed);
            }
            let res = receiver.await;
            match res {
                Ok(handle) => Ok(handle),
                Err(_) => Err(CreateWindowError::OtherError),
            }
        }
    }
    /// Run a [`task`](Future) that will run on the main thread
    fn run_task<F, O>(&self, task: F) -> Task<O>
    where
        F: Future<Output = O> + Send + 'static,
        O: Send + 'static,
    {
        self.get_proxy().create_task(task)
    }
    /// Return a child [`Owner`](reactive_graph::owner::Owner) of this window handle
    fn app_child_reactive_owner(
        &self,
    ) -> impl Future<Output = Result<reactive_graph::owner::Owner, AppHandleActionError>> + Send
    {
        let (sender, receiver) = oneshot::channel();
        let res = self.send_event(EventLoopEvent::GetAppChildReactiveOwner(Box::new(
            GetAppChildReactiveOwner { sender },
        )));
        async move {
            res?;
            receiver.await.map_err(|_| AppHandleActionError::AppExited)
        }
    }
    fn remove_handler(&self, handler_id: HandlerId) {
        if let Err(err) = self.send_event(EventLoopEvent::UnRegisterHandler(Box::new(
            UnregisterEventHandler::Any(handler_id),
        ))) {
            log::error!("{err}");
        }
    }
    fn set_control_flow(&self, control_flow: ControlFlow) {
        let _ = self.send_event(EventLoopEvent::ManagerMethods(Box::new(
            OtherManagerMethods::SetControlFlow(control_flow),
        )));
    }
    /// See [`winit::cursor::CustomCursor`] for more details
    fn register_custom_cursor(
        &self,
        source: CustomCursorSource,
    ) -> impl Future<Output = Result<CustomCursor, AppHandleActionError>> + Send {
        let (send, receive) = oneshot::channel();
        let res = self.send_event(EventLoopEvent::ManagerMethods(Box::new(
            OtherManagerMethods::RegisterCustomCursor(source, send),
        )));
        async move {
            res?;
            receive.await.map_err(|_| AppHandleActionError::AppExited)
        }
    }
    fn listen_device_events_mode(&self, mode: DeviceEvents) {
        let _ = self.send_event(EventLoopEvent::ManagerMethods(Box::new(
            OtherManagerMethods::ListenDeviceEventsMode(mode),
        )));
    }
    fn system_theme(
        &self,
    ) -> impl Future<Output = Result<Option<winit::window::Theme>, AppHandleActionError>> + Send
    {
        let (send, receive) = oneshot::channel();
        let res = self.send_event(EventLoopEvent::ManagerMethods(Box::new(
            OtherManagerMethods::SystemTheme(send),
        )));
        async move {
            res?;
            receive.await.map_err(|_| AppHandleActionError::AppExited)
        }
    }
    fn primary_monitor(
        &self,
    ) -> impl Future<Output = Result<Option<MonitorHandle>, AppHandleActionError>> + Send {
        let (send, receive) = oneshot::channel();
        let res = self.send_event(EventLoopEvent::ManagerMethods(Box::new(
            OtherManagerMethods::PrimaryMonitor(send),
        )));
        async move {
            res?;
            receive.await.map_err(|_| AppHandleActionError::AppExited)
        }
    }
    fn exit(&self) -> Result<(), AppHandleActionError> {
        self.send_event(EventLoopEvent::ManagerMethods(Box::new(
            OtherManagerMethods::Exit,
        )))?;
        Ok(())
    }
    fn available_monitors(
        &self,
    ) -> impl Future<Output = Result<Vec<MonitorHandle>, AppHandleActionError>> + Send {
        let (send, receive) = oneshot::channel();
        let res = self.send_event(EventLoopEvent::ManagerMethods(Box::new(
            OtherManagerMethods::AvailableMonitors(send),
        )));
        async move {
            res?;
            receive.await.map_err(|_| AppHandleActionError::AppExited)
        }
    }
    fn owned_display_handle(
        &self,
    ) -> impl Future<Output = Result<OwnedDisplayHandle, AppHandleActionError>> + Send {
        let (send, receive) = oneshot::channel();
        let res = self.send_event(EventLoopEvent::ManagerMethods(Box::new(
            OtherManagerMethods::OwnedDisplayHandle(send),
        )));
        async move {
            res?;
            receive.await.map_err(|_| AppHandleActionError::AppExited)
        }
    }
}
