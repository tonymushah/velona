use color::palette::css::{BLACK, WHITE};
use drag_resize_regions::TitleBarContainer;
use masonry_widget_wrappers::WidgetWrapper;
use velona::{
    masonry::{
        core::Widget,
        properties::{Background, ContentColor},
        widgets::{Button, Flex, Label, Svg},
    }, widgets::{View, svg::usvg}, window::expect_window,
};

const X_ICON: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/svgs/x.svg"));

const MAXIMIZE_ICON: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/svgs/maximize.svg"));

// const MINIMIZE_ICON: &[u8] = include_bytes!("../svgs/minimize.svg");

const MINUS_ICON: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/svgs/minus.svg"));

pub fn title_bar() -> impl View {
    let window = expect_window();
    
    Flex::row()
        .main_axis_alignment(velona::masonry::properties::types::MainAxisAlignment::SpaceBetween)
        .cross_axis_alignment(velona::masonry::properties::types::CrossAxisAlignment::Center)
        .with_fixed(
            TitleBarContainer::new(
                Label::new("Nekosia")
                    .prepare()
                    .with_props(ContentColor::new(WHITE)),
            )
            .into_new_widget(),
        )
        .with_fixed(
            Flex::row()
                .main_axis_alignment(velona::masonry::properties::types::MainAxisAlignment::Center)
                .cross_axis_alignment(
                    velona::masonry::properties::types::CrossAxisAlignment::Center,
                )
                .with_fixed(action_button(MINUS_ICON))
                .with_fixed(action_button(MAXIMIZE_ICON))
                .with_fixed(action_button(X_ICON))
                .prepare(),
        )
        .prepare()
        // .with_props(BorderColor::new(BLACK))
        // .with_props(BorderWidth::all(20.px()))
        .with_props(Background::Color(BLACK))
}

fn action_button(data: &[u8]) -> velona::masonry::core::NewWidget<Button> {
    Button::new(
        Svg::new(std::sync::Arc::new(
            usvg::Tree::from_data(data, &Default::default()).unwrap(),
        ))
        .prepare(),
    )
    .prepare()
}
