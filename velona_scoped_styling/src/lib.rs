use velona_core::window::{WindowHandle, use_window};

mod propstack;

pub use propstack::ScopedPropstack;

pub(crate) fn use_window_local() -> WindowHandle {
    use_window().expect("Cannot find current window handle in the current context")
}
