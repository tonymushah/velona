pub mod client;

use color::{
    AlphaColor,
    palette::css::{BLACK, WHITE},
};
use drag_resize_regions::{ResizeRegion, TitleBarContainer};
use masonry_widget_wrappers::WidgetWrapper;
use velona::{
    masonry::{
        core::Widget,
        layout::AsUnit,
        properties::{Background, BorderColor, BorderWidth, ContentColor, CornerRadius},
        widgets::{Flex, Label},
    },
    widgets::View,
};

pub fn layout() -> impl View {
    ResizeRegion::new(Flex::column().with_fixed(title_bar().into_new_widget()))
        .into_new_widget()
        .with_props(Background::Color(AlphaColor::from_rgb8(0xff, 0xff, 0xff)))
        .with_props((BorderWidth::all(5.px()), BorderColor::new(BLACK)))
        .with_props(CornerRadius::all(3.px()))
}

fn title_bar() -> impl View {
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
                .prepare(),
        )
        .prepare()
        // .with_props(BorderColor::new(BLACK))
        // .with_props(BorderWidth::all(20.px()))
        .with_props(Background::Color(BLACK))
}
