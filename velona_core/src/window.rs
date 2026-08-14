pub(crate) mod builder;
pub(crate) mod handle;
// TODO report this as finished
pub(crate) mod renderer;
pub(crate) mod runner;

pub use self::{
    builder::WindowBuilder,
    handle::{WindowHandle, WindowHandleActionError},
    renderer::WindowRendererFactory,
};

use reactive_graph::owner::use_context;

/// Get the current window from the current context.
///
/// Return [`None`] if the [`WindowHandle`](handle::WindowHandle) is not found inside the current context.
pub fn use_window() -> Option<handle::WindowHandle> {
    use_context()
}

#[cfg(test)]
mod tests {
    use crate::utils::{is_send, is_send_sync};

    #[test]
    fn test_if_window_builder_is_send_sync() {
        is_send::<super::builder::WindowBuilder>();
    }

    #[test]
    fn test_if_window_handle_is_send_sync() {
        is_send_sync::<super::handle::WindowHandle>();
    }
}
