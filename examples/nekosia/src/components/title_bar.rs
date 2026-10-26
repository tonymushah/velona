use clonelicious::clone;
use color::{
    AlphaColor,
    palette::css::{BLACK, WHITE},
};
use drag_resize_regions::TitleBarContainer;
use masonry_widget_wrappers::WidgetWrapper;
use velona::{
    masonry::{
        core::{Selector, Widget},
        layout::AsUnit,
        properties::{Background, ContentColor, CornerRadius},
        widgets::{Button, Flex, Label, Svg},
    },
    scoped_styling::{ApplyScopedStyles, ScopedPropstack, scoped_propstack},
    widgets::{View, button::NewButtonPressEventsExt, svg::usvg},
    window::expect_window,
};

const X_ICON: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/svgs/x.svg"));

const MAXIMIZE_ICON: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/svgs/maximize.svg"));

// const MINIMIZE_ICON: &[u8] = include_bytes!("../svgs/minimize.svg");

const MINUS_ICON: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/svgs/minus.svg"));

pub fn title_bar() -> impl View {
    let window = expect_window();

    let button_styles = scoped_propstack! {
        ["buttons"] {
            CornerRadius::all(3.px());
            Background::Color(AlphaColor::from_rgb8(0xaa, 0xaa, 0xaa))
        }
        ["buttons", "min"] {
            #[hover]
            Background::Color(AlphaColor::from_rgb8(0xaa, 0xff, 0xff));
            #[active]
            Background::Color(AlphaColor::from_rgb8(0x22, 0xff, 0xff))
        }
        ["buttons", "max"] {
            #[hover]
            Background::Color(AlphaColor::from_rgb8(0xaa, 0xff, 0x99));
            #[active]
            Background::Color(AlphaColor::from_rgb8(0x22, 0xff, 0x22))
        }
        ["buttons", "close"] {
            #[hover]
            Background::Color(AlphaColor::from_rgb8(0xff, 0xaa, 0xaa));
            #[active]
            Background::Color(AlphaColor::from_rgb8(0xff,0x22, 0x22))
        }
    };

    TitleBarContainer::new(Flex::row()
        .main_axis_alignment(velona::masonry::properties::types::MainAxisAlignment::SpaceBetween)
        .cross_axis_alignment(velona::masonry::properties::types::CrossAxisAlignment::Center)
        .with_fixed(
            Label::new("Nekosia")
                .prepare()
                .with_props(ContentColor::new(WHITE)),
        )
        .with_fixed(
            Flex::row()
                .main_axis_alignment(velona::masonry::properties::types::MainAxisAlignment::Center)
                .cross_axis_alignment(
                    velona::masonry::properties::types::CrossAxisAlignment::Center,
                )
                .with_fixed(
                    action_button(MINUS_ICON)
                        .with_class("min")
                        .on_primary(clone!(window => move | | {
                            if let Err(err) = window.set_minimized(true) {
                                log::error!("{err}");
                            }
                        }))
                        .apply(&button_styles))
                .with_fixed_spacer(10.px())
                .with_fixed(
                    action_button(MAXIMIZE_ICON)
                        .with_class("max")
                        .on_primary(clone!(window => move | | {
                            if let Err(err) = window.is_maximized().and_then(|maximized| window.set_maximized(!maximized)) {
                                log::error!("{err}")
                            }
                        }))
                        .apply(&button_styles)
                )
                .with_fixed_spacer(10.px())
                .with_fixed(
                    action_button(X_ICON)
                        .with_class("close")
                        .on_primary(clone!(window => move | | {
                            if let Err(err) = window.close_window() {
                                log::error!("{err}");
                            }
                        }))
                        .apply(&button_styles)
                )
                .prepare(),
        )
        .prepare()
        // .with_props(BorderColor::new(BLACK))
        // .with_props(BorderWidth::all(20.px()))
        .with_props(Background::Color(BLACK))
    )
    .into_new_widget()
}

fn action_button(data: &[u8]) -> velona::masonry::core::NewWidget<Button> {
    Button::new(
        Svg::new(std::sync::Arc::new(
            usvg::Tree::from_data(data, &Default::default()).unwrap(),
        ))
        .prepare(),
    )
    .prepare()
    .with_class("buttons")
}
