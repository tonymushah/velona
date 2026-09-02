use std::{collections::VecDeque, num::NonZero, sync::Arc};

use softbuffer::{SoftBufferError, Surface as SoftSurface};
use vello_cpu::{Pixmap, RasterizerSettings, RenderContext, RenderSettings, Resources};
use velona_renderer::window_handle::WindowHandle;
use winit::event_loop::OwnedDisplayHandle;

use crate::{imaging_vello_cpu::CachedMask, sink::BufferSurfaceSink};

type InnerSurface = SoftSurface<OwnedDisplayHandle, Arc<dyn WindowHandle>>;

pub struct Surface {
    pub ctx: RenderContext,
    pub ressources: Resources,
    pub inner_surface: InnerSurface,
    pub surface_settings: SurfaceSettings,
    pub mask_cache: VecDeque<CachedMask>,
    pub pixmap: Pixmap,
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
        let ctx = RenderContext::new_with(width.get() as _, height.get() as _, settings.render);
        let surface = InnerSurface::new(context, window).unwrap();
        Self {
            ctx,
            ressources: Resources::new(),
            inner_surface: surface,
            surface_settings: settings,
            mask_cache: VecDeque::new(),
            width,
            height,
            pixmap: Pixmap::new(width.get() as _, height.get() as _),
            _d: (),
        }
    }
    pub fn next_sink(&mut self) -> Result<BufferSurfaceSink<'_>, SoftBufferError> {
        let buffer = self.inner_surface.next_buffer()?;
        Ok(BufferSurfaceSink {
            buffer,
            ressources: &mut self.ressources,
            ctx: &mut self.ctx,
            mask_cache: &mut self.mask_cache,
            rasterizer_settings: self.surface_settings.rasterizer,
            width: self.width.get() as _,
            height: self.height.get() as _,
            tolerance: self.surface_settings.tolerance,
            error: None,
            clip_depth: 0,
            group_depth: 0,
            pixmap_mut: &mut self.pixmap,
        })
    }
    fn sync_size(&mut self) {
        self.pixmap
            .resize(self.width.get() as _, self.height.get() as _);
        self.ctx
            .reset_and_resize(self.width.get() as _, self.height.get() as _);
        self.inner_surface.resize(self.width, self.height).unwrap();
        self.pixmap.shrink_to_fit();
    }

    pub fn set_size(&mut self, width: NonZero<u32>, height: NonZero<u32>) {
        self.height = height;
        self.width = width;
        self.sync_size();
        self.reset();
    }

    /// Drop any realized mask artifacts cached by the renderer.
    ///
    /// The cache is renderer-scoped so unchanged masked subscenes can be reused across renders.
    /// Call this if you need to release memory aggressively or after changing assumptions that
    /// affect mask realization outside the recorded scene itself.
    pub fn clear_cached_masks(&mut self) {
        self.mask_cache.clear();
    }

    pub fn reset(&mut self) {
        self.clear_cached_masks();
        self.ctx.reset();
        self.ressources.clear_images();
    }
}

#[derive(Debug, Default, Clone)]
pub struct SurfaceSettings {
    pub rasterizer: RasterizerSettings,
    pub render: RenderSettings,
    pub tolerance: f64,
}
