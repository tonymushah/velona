/// A trait that allows you to consume [`Result`]
pub trait ConsumeResult {
    /// Consume the result and [`log::error`] the [`Err`] if any.
    fn consume_with_log_err(self);
}
