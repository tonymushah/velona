use std::sync::Arc;

use velona::{
    Manager,
    masonry::{
        core::Widget,
        layout::AsUnit,
        palette::css::{BLACK, BLUE, RED, WHITE, WHITE_SMOKE},
        properties::{Background, ContentColor, Padding},
        widgets::{Button, Flex, Label, Svg},
    },
    widgets::{View, button::NewButtonPressEventsExt, svg::usvg},
    window::use_window,
};

const X_ICON: &[u8] = include_bytes!("../svgs/x.svg");

const MAXIMIZE_ICON: &[u8] = include_bytes!("../svgs/maximize.svg");

// const MINIMIZE_ICON: &[u8] = include_bytes!("../svgs/minimize.svg");

const MINUS_ICON: &[u8] = include_bytes!("../svgs/minus.svg");

pub fn title_bar() -> impl View {
    let window = use_window().expect("Cannot find the window handle in the current context");

    Flex::row()
        .main_axis_alignment(velona::masonry::properties::types::MainAxisAlignment::SpaceBetween)
        .cross_axis_alignment(velona::masonry::properties::types::CrossAxisAlignment::Center)
        .with_fixed(
            Button::new(
                Label::new("Custom title bar....")
                    .prepare()
                    .with_props(ContentColor::new(WHITE)),
            )
            .prepare()
            .on_primary(clonelicious::clone!(window => move | | {
                let window2 = window.clone();

                window.run_task(async move {
                    println!("sadsadsada");
                    if let Err(err) = window2.drag_window() {
                        log::error!("cannot drag window {err}");
                    }
                });
            })),
        )
        .with_fixed(
            Flex::row()
                .with_fixed(
                    action_button(MINUS_ICON)
                        .with_props(Background::Color(WHITE_SMOKE))
                        .on_primary(clonelicious::clone!(window => move | | {
                            let _ = window.set_minimized(true);
                        })),
                )
                .with_fixed(
                    action_button(MAXIMIZE_ICON)
                        .with_props(Background::Color(BLUE))
                        .on_primary(clonelicious::clone!(window => move | | {
                            let _ = window
                                .set_maximized(!window.is_maximized().unwrap_or_default());
                        })),
                )
                .with_fixed(
                    action_button(X_ICON)
                        .with_props(Background::Color(RED))
                        .on_primary(clonelicious::clone!(window => move | | {
                            let _ = window.close_window();
                        })),
                )
                .prepare(),
        )
        .prepare()
        .with_props((Background::Color(BLACK), Padding::from_vh(2.px(), 6.px())))
    // .prepare()
}

fn action_button(data: &[u8]) -> velona::masonry::core::NewWidget<Button> {
    Button::new(
        Svg::new(Arc::new(
            usvg::Tree::from_data(data, &Default::default()).unwrap(),
        ))
        .prepare(),
    )
    .prepare()
}
