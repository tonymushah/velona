mod resize_region;
mod title_bar;

use masonry_widget_wrappers::WidgetWrapper;
use velona::{
    VelonaAppExt, WindowBuilder,
    masonry::{
        core::Widget,
        layout::AsUnit,
        palette::css::{BLACK, TRANSPARENT, WHITE},
        properties::{Background, BorderColor, BorderWidth, CornerRadius},
        widgets::{Flex, Label, Portal},
    },
    widgets::{View, align::IntoAlign},
    window::use_window,
};
use velona_renderer_vello_hybrid::{VelloHybridRendererOptions, create_wgpu_context};

use crate::resize_region::ResizeRegion;

fn view() -> impl View {
    ResizeRegion::new(
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
            ),
    )
    .into_new_widget()
    .with_props((BorderWidth::all(4.px()), BorderColor::new(BLACK)))
    .with_props(CornerRadius::all(4.px()))
    .with_props(Background::Color(WHITE))
}

#[cfg_attr(feature = "hotpath", hotpath::main)]
fn main() {
    env_logger::init();

    let g_context = create_wgpu_context(None, None);
    velona::Builder::new(move |_| {
        velona_renderer_vello_hybrid::VelloHybridWindowRenderer::with_options(
            g_context.clone(),
            VelloHybridRendererOptions::new().base_color(TRANSPARENT),
        )
    })
    .with_window(
        WindowBuilder::new(view)
            .with_title("Custom Title Bar")
            .with_decorations(false)
            .with_base_color(TRANSPARENT)
            .with_transparent(true)
            .with_blur(true),
    )
    .build()
    .run()
    .unwrap()
}
