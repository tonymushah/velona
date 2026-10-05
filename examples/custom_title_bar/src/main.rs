mod title_bar;

use velona::{
    VelonaAppExt, WindowBuilder,
    masonry::{
        core::Widget,
        palette::css::WHITE,
        widgets::{Flex, Label, Portal},
    },
    widgets::{View, align::IntoAlign},
};
use velona_renderer_vello_hybrid::create_wgpu_context;

fn view() -> impl View {
    Flex::column()
        .with_fixed(title_bar::title_bar().into_erased())
        .with_fixed(
            Portal::new(
                Label::new("Somemthing")
                    .prepare()
                    .align_centered()
                    .prepare(),
            )
            .content_must_fill(true)
            .constrain_horizontal(true)
            .prepare(),
        )
        .prepare()
}

#[cfg_attr(feature = "hotpath", hotpath::main)]
fn main() {
    env_logger::init();

    let g_context = create_wgpu_context(None, None);
    velona::Builder::new(move |_| {
        velona_renderer_vello_hybrid::VelloHybridWindowRenderer::new(g_context.clone())
    })
    .with_window(
        WindowBuilder::new(view)
            .with_title("Custom Title Bar")
            .with_decorations(false)
            .with_base_color(WHITE),
    )
    .build()
    .run()
    .unwrap()
}
