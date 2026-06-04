//! This basically a fork of [`anyrender::WindowRenderer`](https://docs.rs/anyrender/latest/anyrender/trait.WindowRenderer.html)
//! but the `ScenePainter` uses a [`PaintSink`].
//!

use std::sync::Arc;

use imaging::PaintSink;

use crate::window_handle::WindowHandle;

pub mod window_handle;

/// Abstraction for rendering a scene to a window
pub trait WindowRenderer {
    type ScenePainter<'a>: PaintSink
    where
        Self: 'a;

    /// Begin resuming the renderer. `on_ready` fires when initialization completes —
    /// synchronously inside `resume` on native, asynchronously (via
    /// `wasm_bindgen_futures::spawn_local`) on `wasm32-unknown-unknown`. After it
    /// fires, the embedder must call [`complete_resume`](Self::complete_resume) to
    /// transition the renderer to the active state.
    fn resume<F: FnOnce() + 'static>(
        &mut self,
        window: Arc<dyn WindowHandle>,
        width: u32,
        height: u32,
        on_ready: F,
    );

    /// Finalize a previously-initiated resume. Returns `true` once the renderer is
    /// active and ready to render. Idempotent on already-active renderers; returns
    /// `false` if a pending init has not yet produced a result.
    ///
    /// Backends whose `resume` finishes synchronously inline should return `true`
    /// directly. There is intentionally no default: forgetting to override this on
    /// an async-init backend would silently no-op rendering.
    fn complete_resume(&mut self) -> bool;

    fn suspend(&mut self);
    fn is_active(&self) -> bool;

    /// Returns `true` while an asynchronous resume is in flight (after `resume`
    /// but before `complete_resume` has succeeded). Defaults to `false` for
    /// backends with synchronous initialization.
    fn is_pending(&self) -> bool {
        false
    }

    fn set_size(&mut self, width: u32, height: u32);
    fn render<F: FnOnce(&mut Self::ScenePainter<'_>)>(&mut self, draw_fn: F);
}
