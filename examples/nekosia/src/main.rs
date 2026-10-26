use velona::{VelonaAppExt, WindowBuilder, masonry::palette::css::TRANSPARENT};
use velona_renderer_vello_hybrid::{VelloHybridRendererOptions, create_wgpu_context};
use velona_tokio::RuntimeWraper;

#[cfg_attr(feature = "hotpath", hotpath::main)]
fn main() {
    env_logger::init();

    let tokio_runtime = RuntimeWraper::new(
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap(),
    );

    let g_context = create_wgpu_context(None, None);
    velona::Builder::new(move |_| {
        velona_renderer_vello_hybrid::VelloHybridWindowRenderer::with_options(
            g_context.clone(),
            VelloHybridRendererOptions::new().base_color(TRANSPARENT),
        )
    })
    .with_spawn_fn(tokio_runtime.boxed_spawn_fn())
    .provide_context(tokio_runtime.handle())
    .with_window(
        WindowBuilder::new(nekosia::window_content)
            .with_title("Nekosia")
            .with_decorations(false)
            .with_base_color(TRANSPARENT)
            .with_transparent(true)
            .with_blur(true),
    )
    .build()
    .run()
    .unwrap()
}
