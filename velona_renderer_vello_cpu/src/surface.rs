use std::{num::NonZero, sync::Arc};

use softbuffer::{
    // Buffer, SoftBufferError,
    Surface as SoftSurface,
};
use vello_cpu::{RasterizerSettings, RenderSettings};
use velona_renderer::window_handle::WindowHandle;
use winit::event_loop::OwnedDisplayHandle;

use crate::imaging_vello_cpu::VelloCpuRenderer;

type InnerSurface = SoftSurface<OwnedDisplayHandle, Arc<dyn WindowHandle>>;

pub struct Surface {
    pub renderer: VelloCpuRenderer,
    pub inner_surface: InnerSurface,
    width: NonZero<u32>,
    height: NonZero<u32>,
    _d: (),
}

impl Surface {
    pub fn new(
        width: NonZero<u32>,
        height: NonZero<u32>,
        context: &softbuffer::Context<OwnedDisplayHandle>,
        window: Arc<dyn WindowHandle>,
        settings: SurfaceSettings,
    ) -> Self {
        let mut surface = InnerSurface::new(context, window).unwrap();
        if surface.supports_alpha_mode(softbuffer::AlphaMode::Ignored) {
            let _ = surface.configure(width, height, softbuffer::AlphaMode::Ignored);
        } else {
            let _ = surface.configure(width, height, softbuffer::AlphaMode::Opaque);
        }
        let mut s = Self {
            renderer: VelloCpuRenderer::new(
                width.get() as _,
                height.get() as _,
                settings.render,
                settings.rasterizer,
            ),
            inner_surface: surface,
            width,
            height,
            _d: (),
        };
        s.renderer.set_tolerance(settings.tolerance);
        s
    }
    // pub fn next_sink(&mut self) -> Result<Buffer<'_>, SoftBufferError> {
    //     let buffer = self.inner_surface.next_buffer()?;
    //     if buffer.height() != self.height || buffer.width() != self.width {
    //         self.width = buffer.width();
    //         self.height = buffer.height();
    //         self.renderer
    //             .reset_and_resize(self.width.get() as _, self.height.get() as _);
    //     }
    //     Ok(buffer)
    // }
    fn sync_size(&mut self) {
        self.renderer.ctx.flush();
        self.renderer
            .reset_and_resize(self.width.get() as _, self.height.get() as _);
        self.inner_surface.resize(self.width, self.height).unwrap();
    }

    pub fn set_size(&mut self, width: NonZero<u32>, height: NonZero<u32>) {
        self.height = height;
        self.width = width;
        self.sync_size();
    }

    /// Drop any realized mask artifacts cached by the renderer.
    ///
    /// The cache is renderer-scoped so unchanged masked subscenes can be reused across renders.
    /// Call this if you need to release memory aggressively or after changing assumptions that
    /// affect mask realization outside the recorded scene itself.
    pub fn clear_cached_masks(&mut self) {
        self.renderer.mask_cache.clear();
    }
    pub fn reset(&mut self) {
        self.clear_cached_masks();
        self.renderer.reset();
        self.renderer.resources.clear_images();
    }
}

#[derive(Debug, Default, Clone)]
pub struct SurfaceSettings {
    pub rasterizer: RasterizerSettings,
    pub render: RenderSettings,
    pub tolerance: f64,
}
