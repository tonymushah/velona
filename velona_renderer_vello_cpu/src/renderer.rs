use std::{num::NonZero, sync::Arc};

use softbuffer::Context;
use velona_renderer::{WindowRenderer, window_handle::WindowHandle};
use winit::event_loop::OwnedDisplayHandle;

use crate::{
    BufferSurfaceSink,
    surface::{Surface, SurfaceSettings},
};

#[allow(clippy::large_enum_variant)]
enum RenderState {
    Suspended,
    Active(Surface),
}

pub struct VelloSoftbufferRenderer {
    render_state: RenderState,
    window_handle: Option<Arc<dyn WindowHandle>>,
    settings: SurfaceSettings,
    context: Arc<Context<OwnedDisplayHandle>>,
}

impl VelloSoftbufferRenderer {
    pub fn new(context: Arc<Context<OwnedDisplayHandle>>, settings: SurfaceSettings) -> Self {
        Self {
            render_state: RenderState::Suspended,
            window_handle: None,
            settings,
            context,
        }
    }
}

impl WindowRenderer for VelloSoftbufferRenderer {
    type ScenePainter<'a>
        = BufferSurfaceSink<'a>
    where
        Self: 'a;

    fn resume(&mut self, window: Arc<dyn WindowHandle>, width: u32, height: u32) {
        // Each `resume` must be preceded by `suspend` (or be the first call after
        // construction). Calling while `Pending` or `Active` is a state-machine bug
        // in the embedder: it would orphan the in-flight init's `WGPUContext` and
        // pay for a fresh adapter+device init on the fallback path below.
        if !matches!(self.render_state, RenderState::Suspended) {
            // #[cfg(feature = "tracing")]
            // tracing::warn!("WindowRenderer::resume called from non-Suspended state");
            return;
        }

        self.window_handle = Some(window.clone());
        let surface = Surface::new(
            NonZero::new(width).expect("Cannot have a zero width"),
            NonZero::new(height).expect("Cannot have a zero height"),
            &self.context,
            window,
            self.settings.clone(),
        );
        self.render_state = RenderState::Active(surface)
    }

    fn complete_resume(&mut self) -> bool {
        true
    }

    fn suspend(&mut self) {
        self.render_state = RenderState::Suspended
    }

    fn is_active(&self) -> bool {
        matches!(self.render_state, RenderState::Active(_))
    }

    fn set_size(&mut self, width: u32, height: u32) {
        if let RenderState::Active(active) = &mut self.render_state {
            active.set_size(
                NonZero::new(width).expect("Cannot have a zero width"),
                NonZero::new(height).expect("Cannot have a zero heigth"),
            );
        };
    }

    fn render<F: FnOnce(&mut Self::ScenePainter<'_>)>(&mut self, draw_fn: F) {
        if let RenderState::Active(active) = &mut self.render_state {
            let mut buffer = active.next_sink().unwrap();
            draw_fn(&mut buffer);
            buffer.write_in_buffer().unwrap();
            // buffer.pixmap_mut.shrink_to_fit();
            // log::trace!(
            //     "masks cache: ({}, {})",
            //     buffer.mask_cache.len(),
            //     buffer.mask_cache.capacity()
            // );
            // log::trace!(
            //     "pixmap: ({}, {})",
            //     buffer.pixmap_mut.data().len(),
            //     buffer.pixmap_mut.capacity()
            // );
            buffer.buffer.present().unwrap();
            active.reset();
        };
    }
    fn on_memory_warning(&mut self) {
        if let RenderState::Active(active) = &mut self.render_state {
            active.clear_cached_masks();
        }
    }
}
