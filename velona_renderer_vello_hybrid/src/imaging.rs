// Copyright 2026 the Velona Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

mod image_registry;
mod scene_sink;
mod wgpu_support;

use imaging::{
    RenderSource, RgbaImage,
    record::{Scene, ValidateError, replay},
};

use vello_hybrid::{RenderError, RenderSize, RenderTargetConfig, TextureBindings};

pub use scene_sink::VelloHybridSceneSink;
use wgpu::{TextureFormat, wgt::CommandEncoderDescriptor};

use crate::imaging::{
    image_registry::HybridImageRegistry,
    wgpu_support::{
        OffscreenTarget, ReadbackError, read_texture_into, unpremultiply_rgba8_in_place,
    },
};

/// Errors that can occur when rendering via Vello hybrid.
#[derive(Debug)]
pub enum Error {
    /// The scene is invalid (unbalanced stacks).
    InvalidScene(ValidateError),
    /// An image brush was encountered on a sink path that has no renderer-backed image resolver.
    UnsupportedImageBrush,
    /// A filter configuration could not be translated.
    UnsupportedFilter,
    /// Masks are not supported by this backend yet.
    UnsupportedMask,
    /// Blurred rounded rect draws are not supported by this backend yet.
    UnsupportedBlurredRoundedRect,
    /// Vello hybrid returned a render error.
    Render(RenderError),
    /// An internal invariant was violated.
    Internal(&'static str),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl core::error::Error for Error {}

#[derive(Debug)]
pub(crate) struct VelloHybridRendererState {
    renderer: vello_hybrid::Renderer,
    resources: vello_hybrid::Resources,
    device: wgpu::Device,
    queue: wgpu::Queue,
    tolerance: f64,
    image_registry: HybridImageRegistry,
}

/// Target-oriented renderer that executes `imaging` commands using `vello_hybrid` + `wgpu`.
///
/// This type owns backend state and uploaded images, but it does not own an offscreen render
/// target. Use it when the host application owns the destination texture view.
#[derive(Debug)]
pub struct VelloHybridRenderer {
    state: VelloHybridRendererState,
    target: Option<OffscreenTarget>,
}
/// [`VelloHybridRenderer`] implements [`TextureRenderer`] with [`TextureViewTarget`].
impl VelloHybridRendererState {
    fn checked_size(width: u32, height: u32) -> Result<(u16, u16), Error> {
        let width = u16::try_from(width).map_err(|_| Error::Internal("render width too large"))?;
        let height =
            u16::try_from(height).map_err(|_| Error::Internal("render height too large"))?;
        Ok((width, height))
    }

    fn new_with_target_config(
        device: wgpu::Device,
        queue: wgpu::Queue,
        target_config: &RenderTargetConfig,
    ) -> Self {
        let (renderer, resources) = vello_hybrid::Renderer::new(&device, target_config);

        Self {
            renderer,
            resources,
            device,
            queue,
            tolerance: 0.1,
            image_registry: HybridImageRegistry::default(),
        }
    }

    fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        Self::new_with_target_config(
            device,
            queue,
            &RenderTargetConfig {
                format: TextureFormat::Rgba8Unorm,
                width: 1,
                height: 1,
            },
        )
    }

    fn clear_cached_images(&mut self) {
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("imaging_vello_hybrid clear cached images"),
            });
        self.image_registry
            .clear(&mut self.resources, &mut self.renderer, &mut encoder);
        self.queue.submit([encoder.finish()]);
    }

    /// Lower a semantic [`imaging::record::Scene`] into a native [`vello_hybrid::Scene`].
    fn render_to_view(
        &mut self,
        scene: &vello_hybrid::Scene,
        texture_view: &wgpu::TextureView,
        width: u32,
        height: u32,
    ) -> Result<(), Error> {
        let render_size = RenderSize { width, height };
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("imaging_vello_hybrid render"),
            });

        self.renderer
            .render(
                scene,
                &mut self.resources,
                &self.device,
                &self.queue,
                &mut encoder,
                &render_size,
                texture_view,
                &TextureBindings::new(),
            )
            .map_err(Error::Render)?;

        self.queue.submit([encoder.finish()]);
        Ok(())
    }
}

impl VelloHybridRenderer {
    /// Create a renderer bound to an existing `wgpu` device and queue.
    #[must_use]
    pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        Self {
            state: VelloHybridRendererState::new(device, queue),
            target: None,
        }
    }

    #[must_use]
    pub fn new_with_target_config(
        device: wgpu::Device,
        queue: wgpu::Queue,
        target_config: &RenderTargetConfig,
    ) -> Self {
        Self {
            state: VelloHybridRendererState::new_with_target_config(device, queue, target_config),
            target: None,
        }
    }

    /// Set the tolerance used when converting shapes to paths.
    pub fn set_tolerance(&mut self, tolerance: f64) {
        self.state.tolerance = tolerance;
    }

    /// Destroy all uploaded hybrid image resources cached by this renderer.
    pub fn clear_cached_images(&mut self) {
        self.state.clear_cached_images();
    }

    /// Lower a semantic [`imaging::record::Scene`] into a native [`vello_hybrid::Scene`].
    pub fn encode_scene(
        &mut self,
        scene: &Scene,
        width: u16,
        height: u16,
    ) -> Result<vello_hybrid::Scene, Error> {
        scene.validate().map_err(Error::InvalidScene)?;
        let mut native = vello_hybrid::Scene::new(width, height);
        native.reset();
        let tolerance = self.state.tolerance;
        {
            let mut sink = VelloHybridSceneSink::with_renderer(&mut native, self);
            sink.set_tolerance(tolerance);
            replay(scene, &mut sink);
            sink.finish()?;
        }
        Ok(native)
    }

    pub fn encode_source<S: RenderSource + ?Sized>(
        &mut self,
        source: &mut S,
        width: u32,
        height: u32,
    ) -> Result<vello_hybrid::Scene, Error> {
        source.validate().map_err(Error::InvalidScene)?;
        let (width, height) = VelloHybridRendererState::checked_size(width, height)?;
        let mut native = vello_hybrid::Scene::new(width, height);
        native.reset();
        let tolerance = self.state.tolerance;
        {
            let mut sink = VelloHybridSceneSink::with_renderer(&mut native, self);
            sink.set_tolerance(tolerance);
            source.paint_into(&mut sink);
            sink.finish()?;
        }
        Ok(native)
    }

    /// Render a native [`vello_hybrid::Scene`] into a caller-provided texture view.
    pub fn render_to_texture_view(
        &mut self,
        scene: &vello_hybrid::Scene,
        texture_view: &wgpu::TextureView,
        width: u32,
        height: u32,
    ) -> Result<(), Error> {
        self.state
            .render_to_view(scene, texture_view, width, height)
    }

    fn ensure_target(&mut self, width: u16, height: u16) -> &OffscreenTarget {
        let target = self.target.get_or_insert_with(|| {
            OffscreenTarget::new(&self.state.device, u32::from(width), u32::from(height))
        });
        target.resize(&self.state.device, u32::from(width), u32::from(height));
        target
    }
}

impl VelloHybridRenderer {
    /// Render a native [`vello_hybrid::Scene`] into an RGBA8 image (unpremultiplied).
    pub fn render_into(
        &mut self,
        scene: &vello_hybrid::Scene,
        width: u16,
        height: u16,
        image: &mut RgbaImage,
    ) -> Result<(), Error> {
        let target = self.ensure_target(width, height);
        let texture_view = target.texture_view().clone();
        let target_texture = target.texture().clone();
        let target_width = target.width();
        let target_height = target.height();
        self.render_to_texture_view(scene, &texture_view, target_width, target_height)?;
        readback_into(
            &self.state.device,
            &self.state.queue,
            &target_texture,
            target_width,
            target_height,
            image,
        )
    }

    /// Render a native [`vello_hybrid::Scene`] and return an RGBA8 image (unpremultiplied).
    pub fn render(
        &mut self,
        scene: &vello_hybrid::Scene,
        width: u16,
        height: u16,
    ) -> Result<RgbaImage, Error> {
        let mut image = RgbaImage::new(u32::from(width), u32::from(height));
        self.render_into(scene, width, height, &mut image)?;
        Ok(image)
    }
}

fn readback_into(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    width: u32,
    height: u32,
    image: &mut RgbaImage,
) -> Result<(), Error> {
    read_texture_into(device, queue, texture, width, height, image).map_err(map_readback_error)?;
    unpremultiply_rgba8_in_place(&mut image.data);
    Ok(())
}

// fn readback_into_target(
//     device: &wgpu::Device,
//     queue: &wgpu::Queue,
//     texture: &wgpu::Texture,
//     target: ImageBufferTarget<'_>,
// ) -> Result<(), ReadbackError> {
//     let width = target.width;
//     let height = target.height;
//     let bytes_per_row = target.bytes_per_row;
//     let data = target.data;
//     read_texture_into_target(
//         device,
//         queue,
//         texture,
//         width,
//         height,
//         &mut *data,
//         bytes_per_row,
//     )?;
//     unpremultiply_rgba8_target(&mut *data, width, bytes_per_row);
//     Ok(())
// }

fn map_readback_error(err: ReadbackError) -> Error {
    match err {
        ReadbackError::DevicePoll => Error::Internal("device poll failed"),
        ReadbackError::CallbackDropped => Error::Internal("map_async callback dropped"),
        ReadbackError::BufferMap => Error::Internal("buffer map failed"),
        ReadbackError::InvalidTargetStride => Error::Internal("image target row stride too small"),
        ReadbackError::InvalidTargetBuffer => Error::Internal("image target buffer too small"),
    }
}
