use derive_more::{Display, FromStr};
use enum_all_variants::AllVariants;
use masonry::{
    core::Widget,
    layout::{Length, UnitPoint},
    palette::css::{BLACK, WHITE, WHITE_SMOKE},
    properties::{Background, BorderColor, BorderWidth, CornerRadius, Padding},
    theme::DEFAULT_SPACER_LEN,
    widgets::{Align, Flex, Label, Selector, SizedBox},
};
use reactive_graph::{
    computed::Memo,
    signal::signal,
    traits::{Get, Set},
};
use velona::{AnyNewWidget, Builder, NewWidgetExt, WindowBuilder, widgets::align::NewAlign};

#[derive(Debug, Clone, Copy, Display, FromStr, AllVariants)]
enum Alignment {
    TopLeft,
    Top,
    TopRight,
    Left,
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

impl From<Alignment> for UnitPoint {
    fn from(value: Alignment) -> Self {
        match value {
            Alignment::TopLeft => UnitPoint::TOP_LEFT,
            Alignment::Top => UnitPoint::TOP,
            Alignment::TopRight => UnitPoint::TOP_RIGHT,
            Alignment::Left => UnitPoint::LEFT,
            Alignment::Center => UnitPoint::CENTER,
            Alignment::Right => UnitPoint::RIGHT,
            Alignment::BottomLeft => UnitPoint::BOTTOM_LEFT,
            Alignment::Bottom => UnitPoint::BOTTOM,
            Alignment::BottomRight => UnitPoint::BOTTOM_RIGHT,
        }
    }
}

fn main_view() -> AnyNewWidget {
    let (align, set_align) = signal(Alignment::Center);
    let align_memo: Memo<UnitPoint> = Memo::new(move |_| align.get().into());
    Flex::column()
        .cross_axis_alignment(masonry::properties::types::CrossAxisAlignment::Center)
        .with_fixed(
            Flex::row()
                .with_fixed(Label::new("Alignment: ").prepare())
                .with_fixed_spacer(DEFAULT_SPACER_LEN)
                .with_fixed(
                    SizedBox::new(
                        Selector::new(
                            Alignment::all_variants()
                                .iter()
                                .map(|d| d.to_string())
                                .collect(),
                        )
                        .prepare()
                        .on_action(move |changes| {
                            if let Ok(align) = changes.selected_content.parse::<Alignment>() {
                                set_align.set(align);
                            }
                        })
                        .static_propeperty(BorderColor::new(BLACK))
                        .static_propeperty(BorderWidth::all(Length::px(3.0)))
                        .static_propeperty(CornerRadius::all(Length::px(8.0)))
                        .static_propeperty(Padding::from_vh(Length::px(4.0), Length::px(8.0)))
                        .static_propeperty(Background::Color(WHITE_SMOKE)),
                    )
                    .prepare()
                    .erased(),
                )
                .main_axis_alignment(masonry::properties::types::MainAxisAlignment::Center)
                .prepare(),
        )
        .with_fixed(
            SizedBox::new(
                Align::centered(Label::new("Some text lmao").prepare())
                    .prepare()
                    .alignment(move || align_memo.get()),
            )
            .height(Length::px(500.0))
            .width(Length::px(250.0))
            .prepare(),
        )
        .prepare()
        .erased()
}

fn main() {
    env_logger::init();
    Builder::new(|_| velona_renderer_vello::VelloWindowRenderer::new())
        .window(
            WindowBuilder::new(main_view)
                .with_title("Align")
                .base_color(WHITE),
        )
        .run()
        .unwrap();
}
