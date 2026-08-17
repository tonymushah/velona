//! Basically a fork remix of [`anyrender_vello`](https://docs.rs/anyrender_vello/)

mod renderer;
pub(crate) mod scene_sink;

use std::sync::{Arc, RwLock};

use imaging::record::ValidateError;
pub use renderer::*;
use vello::wgpu::{Features, Limits};
use wgpu_context::WGPUContext;

pub fn default_wgpu_features() -> Features {
    Features::CLEAR_TEXTURE | Features::PIPELINE_CACHE
}

pub fn create_wgpu_context(
    features: Option<Features>,
    limits: Option<Limits>,
) -> Arc<RwLock<WGPUContext>> {
    Arc::new(RwLock::new(WGPUContext::with_features_and_limits(
        Some(features.unwrap_or_default() | default_wgpu_features()),
        limits,
    )))
}

/// Errors that can occur when rendering via Vello.
#[derive(Debug)]
pub enum SinkSceneError {
    /// The scene is invalid (unbalanced stacks).
    InvalidScene(ValidateError),
    /// An unsupported image-brush use was encountered.
    UnsupportedImageBrush,
    /// A filter configuration could not be translated.
    UnsupportedFilter,
    /// A mask mode or masking primitive is not supported by this backend.
    UnsupportedMask,
    /// Glyph draws with non-default blend modes are not supported by this backend yet.
    UnsupportedGlyphBlend,
    /// Glyph brush transforms are not supported by older Vello compatibility lanes.
    UnsupportedGlyphBrushTransform,
    /// Blurred rounded rect draws with non-default blend modes are not supported by this backend yet.
    UnsupportedBlurredRoundedRectBlend,
    /// The clip/group stack was not well-nested for this backend.
    ///
    /// Vello uses a single layer stack for both clipping and blending; `imaging` tracks these as
    /// separate stacks, so scenes that interleave them (e.g. `push_clip`, `push_group`, `pop_clip`)
    /// cannot be represented directly.
    UnbalancedLayerStack,
    /// Vello returned a render error.
    Render(vello::Error),
    /// An internal invariant was violated.
    Internal(&'static str),
}

impl core::fmt::Display for SinkSceneError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl core::error::Error for SinkSceneError {}
