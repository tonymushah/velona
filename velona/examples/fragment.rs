use masonry::{
    core::{NewWidget, Widget},
    layout::Length,
    palette::css::{BLACK, GREEN, WHEAT},
    peniko::color::AlphaColor,
    properties::{
        // ActiveBackground,
        Background,
        BorderColor,
        BorderWidth,
        CheckmarkColor,
        CornerRadius,
        // HoveredBorderColor,
        Padding,
        types::MainAxisAlignment,
    },
    widgets::{Align, Button, Flex, Label},
};
use reactive_graph::{
    signal::signal,
    traits::{Get, Read, Set, Update},
};
use velona::{
    AnyNewWidget, Builder, NewWidgetExt, WindowBuilder,
    components::{checkbox as _checkbox, label, sized_box},
    widgets::button::NewButtonPressEventsExt,
};

enum ViewToUse {
    Text,
    Checkbox,
    Count,
}

fn text() -> AnyNewWidget {
    Label::new("loreman sadsanhjiaijfpsamfanjfksa sadsam asnsa sasafas ")
        .prepare()
        .erased()
}

fn checkbox() -> AnyNewWidget {
    let (checked, set_checked) = signal(false);
    _checkbox(
        move || checked.get(),
        move || {
            if checked.get() {
                "Unchecked checkbox..."
            } else {
                "Checked checkbox!!"
            }
        },
    )
    .static_propeperty(CheckmarkColor { color: GREEN })
    .on(move |checked| {
        set_checked.set(checked.0);
    })
    .erased()
}

trait ButtonExt {
    fn apply_custom_styles(self) -> Self;
    fn apply_counter_button_style(self) -> Self;
}

impl ButtonExt for NewWidget<Button> {
    fn apply_custom_styles(self) -> Self {
        self.static_propeperty(Padding::from_vh(Length::px(4f64), Length::px(8f64)))
            .static_propeperty(CornerRadius::all(Length::px(8f64)))
    }

    fn apply_counter_button_style(self) -> Self {
        self.apply_custom_styles()
            .static_propeperty(Background::Color("#f1aeff".parse().unwrap()))
            // .append_static_propeperty(ActiveBackground(Background::Color(
            //     "#de67f8".parse().unwrap(),
            // )))
            .static_propeperty(BorderColor::new(BLACK))
            // .append_static_propeperty(HoveredBorderColor(BorderColor::new(BLACK)))
            .static_propeperty(BorderWidth::all(Length::px(1f64)))
    }
}

fn counter() -> AnyNewWidget {
    let (count, set_count) = signal(0usize);

    Flex::row()
        .with_fixed(
            Button::with_text("-")
                .prepare()
                .on_primary(move || {
                    set_count.update(|ev| {
                        if let Some(val) = ev.checked_sub(1) {
                            *ev = val
                        }
                    });
                })
                .apply_counter_button_style(),
        )
        .with_fixed(label(move || format!("{}", count.get())))
        .with_fixed(
            Button::with_text("+")
                .prepare()
                .on_primary(move || {
                    set_count.update(|ev| {
                        if let Some(val) = ev.checked_add(1) {
                            *ev = val
                        }
                    });
                })
                .apply_counter_button_style(),
        )
        .prepare()
        .erased()
}

fn main_view() -> AnyNewWidget {
    let (view, set_view) = signal(ViewToUse::Text);
    Align::centered(
        Flex::column()
            .with_fixed(
                Flex::row()
                    .with_fixed(
                        Button::with_text("Some text")
                            .prepare()
                            .on_primary(move || {
                                set_view.set(ViewToUse::Text);
                            })
                            .static_propeperty(Background::Color(AlphaColor::from_rgb8(
                                200, 100, 100,
                            )))
                            .apply_custom_styles(),
                    )
                    .with_fixed(
                        Button::with_text("Some checkbox")
                            .prepare()
                            .on_primary(move || {
                                set_view.set(ViewToUse::Checkbox);
                            })
                            .static_propeperty(Background::Color(AlphaColor::from_rgb8(
                                200, 100, 100,
                            )))
                            .apply_custom_styles(),
                    )
                    .with_fixed(
                        Button::with_text("Count")
                            .prepare()
                            .on_primary(move || {
                                set_view.set(ViewToUse::Count);
                            })
                            .static_propeperty(Background::Color(AlphaColor::from_rgb8(
                                200, 100, 100,
                            )))
                            .apply_custom_styles(),
                    )
                    .main_axis_alignment(MainAxisAlignment::Center)
                    .prepare(),
            )
            .with_fixed(sized_box(move || match *view.read() {
                ViewToUse::Text => text(),
                ViewToUse::Checkbox => checkbox(),
                ViewToUse::Count => counter(),
            }))
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
            WindowBuilder::new(main_view)
                .with_title("Fragment")
                .base_color(WHEAT),
        )
        .run()
        .unwrap();
}
