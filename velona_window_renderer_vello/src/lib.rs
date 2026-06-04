//! Basically a fork remix of [`anyrender_vello`](https://docs.rs/anyrender_vello/)

mod renderer;

pub use renderer::*;

pub fn factory_default<P>(_p: P) -> VelloWindowRenderer {
    VelloWindowRenderer::new()
}
