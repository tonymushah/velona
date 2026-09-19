use velona::{WindowBuilder, masonry::palette::css::WHITE};
use velona_renderer_vello::create_wgpu_context;
use velona_router::Router;

fn main_router() -> Router {
    Router::default()
}

#[cfg_attr(feature = "hotpath", hotpath::main)]
fn main() {
    env_logger::init();
    let g_context = create_wgpu_context(None, None);
    velona::Builder::new(move |_| {
        velona_renderer_vello::VelloWindowRenderer::new(g_context.clone())
    })
    .with_window(
        WindowBuilder::new(main_router().build())
            .with_title("Router example")
            .with_base_color(WHITE),
    )
    .run()
    .unwrap()
}
