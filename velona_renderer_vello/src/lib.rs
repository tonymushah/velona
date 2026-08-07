//! Basically a fork remix of [`anyrender_vello`](https://docs.rs/anyrender_vello/)

mod renderer;

use std::sync::{Arc, RwLock};

use imaging_vello::wgpu::{Features, Limits};
pub use renderer::*;
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
