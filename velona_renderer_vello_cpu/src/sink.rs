use std::collections::VecDeque;

use imaging::record::{Scene, replay_transformed};
use imaging::{
    BlurredRoundedRect, ClipRef, Composite, FillRef, GeometryRef, GlyphRunRef, GroupRef, PaintSink,
    StrokeRef,
};
use imaging::{Filter, MaskMode};
use kurbo::{Affine, BezPath, Shape, StrokeOpts, stroke};
use peniko::{BlendMode, Brush, BrushRef, Fill, Style};
use softbuffer::{Buffer, PixelFormat};
use vello_common::fearless_simd;
use vello_common::filter_effects::{EdgeMode, Filter as VelloFilter, FilterGraph, FilterPrimitive};
use vello_common::paint::Image as VelloImage;
use vello_cpu::{
    Glyph as VelloGlyph, ImageSource, Level, Pixmap, PixmapMut, RasterizerSettings, RenderContext,
    Resources,
};

use crate::imaging_vello_cpu::CachedMask;
use crate::imaging_vello_cpu::{RendererError, VelloCpuRenderer};
use crate::utils::{f64_to_f32, swap_blue_and_red_channel, unpremultiply_rgba8_in_place};

#[derive(Debug)]
pub struct BufferSurfaceSink<'surface> {
    pub(crate) buffer: Buffer<'surface>,
    pub(crate) ctx: &'surface mut RenderContext,
    pub(crate) ressources: &'surface mut Resources,
    pub(crate) mask_cache: &'surface mut VecDeque<CachedMask>,
    pub(crate) rasterizer_settings: RasterizerSettings,
    pub(crate) width: u16,
    pub(crate) height: u16,
    pub(crate) tolerance: f64,
    pub(crate) error: Option<RendererError>,
    pub(crate) clip_depth: u32,
    pub(crate) group_depth: u32,
    pub(crate) simd_level: Level, // pub(crate) pixmap_mut: &'surface mut Pixmap,
}

impl<'surface> BufferSurfaceSink<'surface> {
    fn set_error_once(&mut self, err: RendererError) {
        if self.error.is_none() {
            self.error = Some(err);
        }
    }

    fn brush_to_paint(
        &mut self,
        brush: BrushRef<'_>,
        composite: Composite,
    ) -> Option<vello_cpu::PaintType> {
        let brush = brush.to_owned().multiply_alpha(composite.alpha);
        let paint: vello_cpu::PaintType = match brush {
            Brush::Solid(c) => Brush::Solid(c),
            Brush::Gradient(g) => Brush::Gradient(g),
            Brush::Image(image) => Brush::Image(VelloImage {
                image: ImageSource::from_peniko_image_data(&image.image),
                sampler: image.sampler,
            }),
        };
        Some(paint)
    }

    fn geometry_to_path(&self, geom: GeometryRef<'_>) -> BezPath {
        match geom {
            GeometryRef::Rect(r) => r.to_path(self.tolerance),
            GeometryRef::RoundedRect(rr) => rr.to_path(self.tolerance),
            GeometryRef::Path(p) => p.clone(),
            GeometryRef::OwnedPath(p) => p,
        }
    }

    fn clip_to_path(&mut self, clip: ClipRef<'_>) -> (Affine, BezPath, Fill) {
        match clip {
            ClipRef::Fill {
                transform,
                shape,
                fill_rule,
            } => (transform, self.geometry_to_path(shape), fill_rule),
            ClipRef::Stroke {
                transform,
                shape,
                stroke: style,
            } => {
                let path = self.geometry_to_path(shape);
                let outline = stroke(path.iter(), style, &StrokeOpts::default(), self.tolerance);
                (transform, outline, Fill::NonZero)
            }
        }
    }

    fn filters_to_vello(&mut self, filters: &[Filter]) -> Option<VelloFilter> {
        if filters.is_empty() {
            return None;
        }

        let mut graph = FilterGraph::new();
        let mut last = None;
        for f in filters {
            let primitive = match *f {
                Filter::Flood { color } => FilterPrimitive::Flood { color },
                Filter::Blur {
                    std_deviation_x,
                    std_deviation_y,
                } => FilterPrimitive::GaussianBlur {
                    std_deviation: std_deviation_x.max(std_deviation_y),
                    edge_mode: EdgeMode::None,
                },
                Filter::DropShadow {
                    dx,
                    dy,
                    std_deviation_x,
                    std_deviation_y,
                    color,
                } => FilterPrimitive::DropShadow {
                    dx,
                    dy,
                    std_deviation: std_deviation_x.max(std_deviation_y),
                    color,
                    edge_mode: EdgeMode::None,
                },
                Filter::Offset { dx, dy } => FilterPrimitive::Offset { dx, dy },
            };
            last = Some(graph.add(primitive, None));
        }
        if let Some(out) = last {
            graph.set_output(out);
        } else {
            self.set_error_once(RendererError::UnsupportedFilter);
            return None;
        }
        Some(VelloFilter {
            graph: std::sync::Arc::new(graph),
        })
    }

    fn draw_glyph_run(
        &mut self,
        glyph_run: GlyphRunRef<'_>,
        glyphs: &mut dyn Iterator<Item = imaging::record::Glyph>,
    ) {
        let Some(paint) = self.brush_to_paint(glyph_run.brush, glyph_run.composite) else {
            return;
        };
        self.ctx.set_transform(glyph_run.transform);
        self.ctx
            .set_paint_transform(glyph_run.brush_transform.unwrap_or(Affine::IDENTITY));
        self.ctx.set_paint(paint);
        self.ctx.set_blend_mode(glyph_run.composite.blend);
        // TODO: Revisit this allocation. `PaintSink` currently gives us a
        // one-shot iterator, while glifo's glyph builders need a cloneable
        // glyph source.
        let glyphs = glyphs
            .map(|glyph| VelloGlyph {
                id: glyph.id,
                x: glyph.x,
                y: glyph.y,
            })
            .collect::<Vec<_>>();

        match glyph_run.style {
            Style::Fill(fill_rule) => {
                self.ctx.set_fill_rule(*fill_rule);
                let builder = self
                    .ctx
                    .glyph_run(self.ressources, glyph_run.font)
                    .font_size(glyph_run.font_size)
                    .hint(glyph_run.hint)
                    .normalized_coords(glyph_run.normalized_coords);
                let builder = if let Some(transform) = glyph_run.glyph_transform {
                    builder.glyph_transform(transform)
                } else {
                    builder
                };
                builder.fill_glyphs(glyphs.into_iter());
            }
            Style::Stroke(stroke) => {
                self.ctx.set_stroke(stroke.clone());
                let builder = self
                    .ctx
                    .glyph_run(self.ressources, glyph_run.font)
                    .font_size(glyph_run.font_size)
                    .hint(glyph_run.hint)
                    .normalized_coords(glyph_run.normalized_coords);
                let builder = if let Some(transform) = glyph_run.glyph_transform {
                    builder.glyph_transform(transform)
                } else {
                    builder
                };
                builder.stroke_glyphs(glyphs.into_iter());
            }
        }
    }

    fn draw_blurred_rounded_rect(&mut self, draw: BlurredRoundedRect) {
        self.ctx.set_transform(draw.transform);
        self.ctx
            .set_paint(draw.color.multiply_alpha(draw.composite.alpha));
        self.ctx.set_blend_mode(draw.composite.blend);
        self.ctx.fill_blurred_rounded_rect(
            &draw.rect,
            f64_to_f32(draw.radius),
            f64_to_f32(draw.std_dev),
            false,
        );
    }

    fn render_mask(
        &mut self,
        scene: &Scene,
        mode: MaskMode,
        transform: Affine,
    ) -> Option<vello_cpu::Mask> {
        if let Some(mask) = self.lookup_cached_mask(scene, mode, transform) {
            return Some(mask);
        }

        let mut renderer = VelloCpuRenderer::new(self.width, self.height);
        renderer.set_tolerance(self.tolerance);
        replay_transformed(scene, &mut renderer, transform);
        if let Some(err) = renderer.error.take() {
            self.set_error_once(err);
            return None;
        }
        if renderer.clip_depth != 0 {
            self.set_error_once(RendererError::Internal(
                "unbalanced clip stack in mask scene",
            ));
            return None;
        }
        if renderer.group_depth != 0 {
            self.set_error_once(RendererError::Internal(
                "unbalanced group stack in mask scene",
            ));
            return None;
        }

        let mut pixmap = Pixmap::new(
            self.buffer.width().get() as _,
            self.buffer.height().get() as _,
        );
        renderer.ctx.flush();
        renderer.ctx.render(&mut pixmap, &mut renderer.resources);
        let mask = match mode {
            MaskMode::Alpha => vello_cpu::Mask::new_alpha(&pixmap),
            MaskMode::Luminance => vello_cpu::Mask::new_luminance(&pixmap),
        };
        self.store_cached_mask(scene, mode, transform, mask.clone());
        Some(mask)
    }

    fn lookup_cached_mask(
        &self,
        scene: &Scene,
        mode: MaskMode,
        transform: Affine,
    ) -> Option<vello_cpu::Mask> {
        self.mask_cache
            .iter()
            .find(|entry| {
                entry.mode == mode && entry.transform == transform && entry.scene == *scene
            })
            .map(|entry| entry.mask.clone())
    }

    fn store_cached_mask(
        &mut self,
        scene: &Scene,
        mode: MaskMode,
        transform: Affine,
        mask: vello_cpu::Mask,
    ) {
        // TODO: If more backends end up wanting realized-mask caches, add a portable scene/cache
        // key at the imaging layer instead of retaining whole scenes in backend-local caches.
        self.mask_cache.push_back(CachedMask {
            scene: scene.clone(),
            mode,
            transform,
            mask,
        });
    }

    pub(crate) fn write_in_buffer(&mut self) -> Result<(), RendererError> {
        if let Some(err) = self.error.take() {
            return Err(err);
        }
        if self.clip_depth != 0 {
            return Err(RendererError::Internal("unbalanced clip stack"));
        }
        if self.group_depth != 0 {
            return Err(RendererError::Internal("unbalanced group stack"));
        }

        self.ctx.flush();

        let pixmap_mut = if let Some(pix) =
            PixmapMut::new(self.width as _, self.height as _, self.buffer.data_u8())
        {
            pix
        } else {
            let Some(pix) = PixmapMut::new(
                self.width as _,
                self.height as _,
                self.buffer
                    .data_u8()
                    .split_at_mut(usize::from(self.width) * usize::from(self.height) * 4)
                    .0,
            ) else {
                return Ok(());
            };
            pix
        };

        self.ctx
            .render_with(pixmap_mut, self.ressources, self.rasterizer_settings);
        unpremultiply_rgba8_in_place(self.buffer.data_u8());

        if PixelFormat::default() == PixelFormat::Bgra8 {
            let level = self.simd_level;
            fearless_simd::dispatch!(level, simd => swap_blue_and_red_channel(simd, self.buffer.data_u8()));
        }

        // log::trace!("buffer size: {}", self.buffer.pixels().len());
        Ok(())
    }
}

impl<'surface> PaintSink for BufferSurfaceSink<'surface> {
    fn push_clip(&mut self, clip: ClipRef<'_>) {
        if self.error.is_some() {
            return;
        }
        let (xf, path, fill_rule) = self.clip_to_path(clip);
        self.ctx.set_transform(xf);
        self.ctx.set_fill_rule(fill_rule);
        self.ctx.push_clip_path(&path);
        self.clip_depth += 1;
    }

    fn pop_clip(&mut self) {
        if self.error.is_some() {
            return;
        }
        if self.clip_depth == 0 {
            self.set_error_once(RendererError::Internal("pop_clip underflow"));
            return;
        }
        self.ctx.pop_clip_path();
        self.clip_depth -= 1;
    }

    fn push_group(&mut self, group: GroupRef<'_>) {
        if self.error.is_some() {
            return;
        }
        let (clip_path, clip_transform) = match group.clip {
            None => (None, Affine::IDENTITY),
            Some(clip) => {
                let (xf, path, fill_rule) = self.clip_to_path(clip);
                self.ctx.set_fill_rule(fill_rule);
                (Some(path), xf)
            }
        };

        self.ctx.set_transform(clip_transform);

        let blend: Option<BlendMode> = Some(group.composite.blend);
        let opacity: Option<f32> = Some(group.composite.alpha);
        let mask = group
            .mask
            .and_then(|mask| self.render_mask(mask.mask.scene, mask.mask.mode, mask.transform));
        let filter = self.filters_to_vello(group.filters);
        self.ctx
            .push_layer(clip_path.as_ref(), blend, opacity, mask, filter);
        self.group_depth += 1;
    }

    fn pop_group(&mut self) {
        if self.error.is_some() {
            return;
        }
        if self.group_depth == 0 {
            self.set_error_once(RendererError::Internal("pop_group underflow"));
            return;
        }
        self.ctx.pop_layer();
        self.group_depth -= 1;
    }

    fn fill(&mut self, draw: FillRef<'_>) {
        if self.error.is_some() {
            return;
        }

        let Some(paint) = self.brush_to_paint(draw.brush, draw.composite) else {
            return;
        };
        self.ctx.set_transform(draw.transform);
        self.ctx.set_fill_rule(draw.fill_rule);
        self.ctx
            .set_paint_transform(draw.brush_transform.unwrap_or(Affine::IDENTITY));
        self.ctx.set_blend_mode(draw.composite.blend);
        self.ctx.set_paint(paint);

        match draw.shape {
            GeometryRef::Rect(r) => self.ctx.fill_rect(&r),
            GeometryRef::RoundedRect(rr) => {
                let path = rr.to_path(self.tolerance);
                self.ctx.fill_path(&path);
            }
            GeometryRef::Path(p) => self.ctx.fill_path(p),
            GeometryRef::OwnedPath(p) => self.ctx.fill_path(&p),
        }
    }

    fn stroke(&mut self, draw: StrokeRef<'_>) {
        if self.error.is_some() {
            return;
        }

        let Some(paint) = self.brush_to_paint(draw.brush, draw.composite) else {
            return;
        };
        self.ctx.set_transform(draw.transform);
        self.ctx.set_stroke(draw.stroke.clone());
        self.ctx
            .set_paint_transform(draw.brush_transform.unwrap_or(Affine::IDENTITY));
        self.ctx.set_blend_mode(draw.composite.blend);
        self.ctx.set_paint(paint);

        match draw.shape {
            GeometryRef::Rect(r) => self.ctx.stroke_rect(&r),
            GeometryRef::RoundedRect(rr) => {
                let path = rr.to_path(self.tolerance);
                self.ctx.stroke_path(&path);
            }
            GeometryRef::Path(p) => self.ctx.stroke_path(p),
            GeometryRef::OwnedPath(p) => self.ctx.stroke_path(&p),
        }
    }

    fn glyph_run(
        &mut self,
        draw: GlyphRunRef<'_>,
        glyphs: &mut dyn Iterator<Item = imaging::record::Glyph>,
    ) {
        if self.error.is_some() {
            return;
        }
        self.draw_glyph_run(draw, glyphs);
    }

    fn blurred_rounded_rect(&mut self, draw: BlurredRoundedRect) {
        if self.error.is_some() {
            return;
        }
        self.draw_blurred_rounded_rect(draw);
    }
}
