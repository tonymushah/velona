use winit_core::{error::RequestError, window::ImeRequestError};

/// A trait that allows you to consume [`Result`]
pub trait ConsumeResult {
    /// Consume the result and [`log::error`] the [`Err`] if any.
    #[track_caller]
    fn consume_with_log_err(self);
}

impl<T> ConsumeResult for Result<T, RequestError> {
    fn consume_with_log_err(self) {
        if let Err(err) = self {
            log::error!("{err}")
        }
    }
}

impl<T> ConsumeResult for Result<T, ImeRequestError> {
    fn consume_with_log_err(self) {
        if let Err(err) = self {
            log::error!("{err}")
        }
    }
}
