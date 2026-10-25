mod title_bar;

use color::{AlphaColor, palette::css::BLACK};
use drag_resize_regions::ResizeRegion;
use masonry_widget_wrappers::WidgetWrapper;
use velona::{
    masonry::{
        layout::AsUnit,
        properties::{Background, BorderColor, BorderWidth, CornerRadius},
        widgets::Flex,
    },
    widgets::View,
};

pub fn window_content() -> impl View {
    ResizeRegion::new(Flex::column().with_fixed(title_bar::title_bar().into_new_widget()))
        .into_new_widget()
        .with_props(Background::Color(AlphaColor::from_rgb8(0xff, 0xff, 0xff)))
        .with_props((BorderWidth::all(5.px()), BorderColor::new(BLACK)))
        .with_props(CornerRadius::all(3.px()))
}
