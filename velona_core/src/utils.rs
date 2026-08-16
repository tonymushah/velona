use log::warn;

pub(crate) mod convert_winit_event;
pub(crate) mod events;
mod handler_id;
pub mod memo;
mod res_log;

pub use handler_id::HandlerId;

pub use res_log::ConsumeResult;

pub(crate) fn todo_warn_of_something(something: &'static str) {
    if something.is_empty() {
        warn!("Not yet implemented")
    } else {
        warn!("Not yet implemented {something}")
    }
}

#[allow(dead_code)]
pub(crate) fn todo_warn() {
    todo_warn_of_something("");
}

#[cfg(test)]
pub(crate) fn is_send_sync<T>()
where
    T: Send + Sync,
{
}

#[cfg(test)]
pub(crate) fn is_send<T>()
where
    T: Send,
{
}

// pub(crate) fn noop() {}

pub use crate::window::event_handlers::{
    register_typed_widget_action_handler, register_widget_action_handler,
};

#[cfg(not(feature = "hotpath"))]
pub(crate) use flume::{Receiver as FlumeReceiver, Sender as FlumeSender};
#[cfg(feature = "hotpath")]
pub(crate) use hotpath::wrap::flume::{Receiver as FlumeReceiver, Sender as FlumeSender};

#[cfg(not(feature = "hotpath"))]
pub(crate) fn flume_channel<T: Send + 'static>() -> (FlumeSender<T>, FlumeReceiver<T>) {
    flume::unbounded()
}

#[cfg(feature = "hotpath")]
pub(crate) fn flume_channel<T: Send + std::fmt::Debug + 'static>()
-> (FlumeSender<T>, FlumeReceiver<T>) {
    hotpath::channel!(flume::unbounded(), log = true)
}
