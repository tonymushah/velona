use masonry::{
    core::{NewWidget, PointerButton, Widget},
    kurbo::Point,
    layout::Length,
    palette::css::{
        WHITE,
        // WHITE_SMOKE
    },
    properties::{
        Background, BorderColor, BorderWidth, BoxShadow, ContentColor, CornerRadius, Padding,
    },
    widgets::{Align, Button, Flex, Label},
};
use reactive_graph::{
    signal::{WriteSignal, signal},
    traits::{Get, Update},
};
use velona::{NewWidgetExt, components::label, window::builder::WindowBuilder};

fn button<U>(set_count: WriteSignal<u32>, update: U, text: &'static str) -> NewWidget<Button>
where
    U: Fn(&mut u32) + 'static,
{
    Button::new(Label::new(text).prepare())
        .prepare()
        .register_handler(move |press| {
            let Some(btt) = press.button.as_ref() else {
                return;
            };
            if matches!(btt, PointerButton::Primary) {
                set_count.update(&update);
            }
        })
        .append_static_propeperty(Padding::from_vh(Length::px(3.0), Length::px(8.0)))
        .append_static_propeperty(CornerRadius::all(Length::px(8.0)))
        .append_static_propeperty(Background::Color(WHITE))
        // .append_static_propeperty(ActiveBackground(Background::Color(WHITE_SMOKE)))
        .append_static_propeperty(BorderColor::new(
            masonry::peniko::color::AlphaColor::from_rgb8(255, 0, 41),
        ))
        // .append_static_propeperty(HoveredBorderColor(BorderColor::new(
        //     masonry::peniko::color::AlphaColor::from_rgb8(255, 0, 41),
        // )))
        .append_static_propeperty(BorderWidth::all(Length::px(3.0)))
        .append_static_propeperty(BoxShadow::new(
            masonry::peniko::color::AlphaColor::from_rgb8(255, 0, 41),
            Point::new(0.0, 4.0),
        ))
}

fn view() -> NewWidget<dyn Widget + 'static> {
    let (count, set_count) = signal(0u32);
    Align::centered(
        Flex::row()
            .with_fixed(button(
                set_count,
                |count| {
                    *count = count.checked_sub(1).unwrap_or_default();
                },
                "Decrement",
            ))
            .with_fixed(
                label(move || format!("Count: {}", count.get()))
                    .append_static_propeperty(ContentColor::new(WHITE)),
            )
            .with_fixed(button(
                set_count,
                |count| {
                    let Some(new_count) = count.checked_add(1) else {
                        return;
                    };
                    *count = new_count;
                },
                "Increment",
            ))
            .prepare(),
    )
    .prepare()
    .erased()
}

#[cfg_attr(feature = "hotpath", hotpath::main)]
fn main() {
    env_logger::init();
    velona::app::Builder::new(|_| velona_renderer_vello::VelloWindowRenderer::new())
        .window(WindowBuilder::new(view).with_title("aaaaaa"))
        .run()
        .unwrap()
}
