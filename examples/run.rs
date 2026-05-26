use masonry::{
    core::{NewWidget, PointerButton, Widget},
    kurbo::Point,
    palette::css::{WHITE, WHITE_SMOKE},
    properties::{
        ActiveBackground, Background, BorderColor, BorderWidth, BoxShadow, ContentColor,
        CornerRadius, HoveredBorderColor, Padding,
    },
    widgets::{Align, Button, Flex, Label},
};
use reactive_graph::{
    signal::{WriteSignal, signal},
    traits::{Get, Update},
};
use velona::{NewWidgetExt, widgets::label::label, window::WindowBuilder};

fn button<U>(set_count: WriteSignal<u32>, update: U, text: &'static str) -> NewWidget<Button>
where
    U: Fn(&mut u32) + 'static,
{
    Button::new(Label::new(text).with_auto_id())
        .with_auto_id()
        .register_handler(move |press| {
            let Some(btt) = press.button.as_ref() else {
                return;
            };
            if matches!(btt, PointerButton::Primary) {
                set_count.update(&update);
            }
        })
        .append_static_propeperty(Padding::from_vh(3.0, 8.0))
        .append_static_propeperty(CornerRadius::all(8.0))
        .append_static_propeperty(Background::Color(WHITE))
        .append_static_propeperty(ActiveBackground(Background::Color(WHITE_SMOKE)))
        .append_static_propeperty(BorderColor::new(
            masonry::peniko::color::AlphaColor::from_rgb8(255, 0, 41),
        ))
        .append_static_propeperty(HoveredBorderColor(BorderColor::new(
            masonry::peniko::color::AlphaColor::from_rgb8(255, 0, 41),
        )))
        .append_static_propeperty(BorderWidth::all(3.0))
        .append_static_propeperty(BoxShadow::new(
            masonry::peniko::color::AlphaColor::from_rgb8(255, 0, 41),
            Point::new(0.0, 4.0),
        ))
}

fn view() -> NewWidget<dyn Widget + 'static> {
    let (count, set_count) = signal(0u32);
    Align::centered(
        Flex::row()
            .with_child(button(
                set_count,
                |count| {
                    *count = count.checked_sub(1).unwrap_or_default();
                },
                "Decrement",
            ))
            .with_child(
                label(move || format!("Count: {}", count.get()))
                    .append_static_propeperty(ContentColor::new(WHITE)),
            )
            .with_child(button(
                set_count,
                |count| {
                    let Some(new_count) = count.checked_add(1) else {
                        return;
                    };
                    *count = new_count;
                },
                "Increment",
            ))
            .with_auto_id(),
    )
    .with_auto_id()
    .erased()
}

fn main() {
    env_logger::init();
    velona::app::Builder::default()
        .window(WindowBuilder::new(view).with_title("aaaaaa"))
        .run()
        .unwrap()
}
