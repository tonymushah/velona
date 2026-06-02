use async_task::Task;
use futures_channel::oneshot;

use crate::{
    WindowBuilder,
    app::{EventLoopEvent, el_event::EventProxyHandle},
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
}
