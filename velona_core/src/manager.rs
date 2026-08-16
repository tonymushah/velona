use async_task::Task;
use futures_channel::oneshot;

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
}
