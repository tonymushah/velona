use velona_renderer::WindowRenderer;

use crate::app::AppHandle;

/// A trait that create a [`WindowRenderer`]
///
/// This might be useful if you want to create your own renderer factory structs
/// with custom logic that the a simple [`FnMut`] can't provide.
pub trait WindowRendererFactory {
    type WindowRenderer: WindowRenderer;
    fn create(&mut self, app_handle: &AppHandle) -> Self::WindowRenderer;
}

impl<F, W> WindowRendererFactory for F
where
    F: FnMut(&AppHandle) -> W,
    W: WindowRenderer,
{
    type WindowRenderer = W;
    fn create(&mut self, app_handle: &AppHandle) -> Self::WindowRenderer {
        (self)(app_handle)
    }
}
