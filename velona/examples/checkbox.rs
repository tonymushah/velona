use masonry::{
    core::Widget,
    palette::css::WHITE,
    properties::types::MainAxisAlignment,
    theme::DEFAULT_SPACER_LEN,
    widgets::{Align, Flex},
};
use velona::{
    AnyNewWidget, Builder, NewWidgetExt, WindowBuilder, components::checkbox as _checkbox,
};
use velona_core::reactive::signal::signal;

fn view() -> AnyNewWidget {
    let (checked, set_checked) = signal(false);
    let checked1 = move || checked();
    Align::centered(
        Flex::column()
            .with_fixed_spacer(DEFAULT_SPACER_LEN)
            .with_fixed(
                _checkbox(checked1, move || {
                    if checked() {
                        "Checkbox checked"
                    } else {
                        "Checkbox not checked"
                    }
                })
                .on_action(move |event| {
                    set_checked(event.0);
                }),
            )
            .main_axis_alignment(MainAxisAlignment::Center)
            .prepare(),
    )
    .prepare()
    .erased()
}

#[cfg_attr(feature = "hotpath", hotpath::main)]
fn main() {
    env_logger::init();
    Builder::new(|_| velona_renderer_vello::VelloWindowRenderer::new())
        .window(
            WindowBuilder::new(view)
                .with_title("Checkbox")
                .base_color(WHITE),
        )
        .run()
        .unwrap();
}
