use masonry::{
    core::{NewWidget, PointerButton, Widget},
    palette::css::{BLACK, GREEN, WHEAT},
    peniko::color::AlphaColor,
    properties::{
        ActiveBackground, Background, BorderColor, BorderWidth, CheckmarkColor, CornerRadius,
        HoveredBorderColor, Padding,
    },
    widgets::{Align, Button, Flex, Label},
};
use reactive_graph::{
    signal::signal,
    traits::{Get, Read, Set, Update},
};
use velona::{
    AnyNewWidget, Builder, NewWidgetExt, WindowBuilder,
    fragment::fragment,
    widgets::{checkbox::_checkbox, label::label},
};

enum ViewToUse {
    Text,
    Checkbox,
    Count,
}

fn text() -> AnyNewWidget {
    Label::new("loreman sadsanhjiaijfpsamfanjfksa sadsam asnsa sasafas ")
        .with_auto_id()
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
    .append_static_propeperty(CheckmarkColor { color: GREEN })
    .register_handler(move |checked| {
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
        self.append_static_propeperty(Padding::from_vh(4f64, 8f64))
            .append_static_propeperty(CornerRadius::all(8f64))
    }

    fn apply_counter_button_style(self) -> Self {
        self.apply_custom_styles()
            .append_static_propeperty(Background::Color("#f1aeff".parse().unwrap()))
            .append_static_propeperty(ActiveBackground(Background::Color(
                "#de67f8".parse().unwrap(),
            )))
            .append_static_propeperty(BorderColor::new(BLACK))
            .append_static_propeperty(HoveredBorderColor(BorderColor::new(BLACK)))
            .append_static_propeperty(BorderWidth::all(1f64))
    }
}

fn counter() -> AnyNewWidget {
    let (count, set_count) = signal(0usize);

    Flex::row()
        .with_child(
            Button::with_text("-")
                .with_auto_id()
                .register_handler(move |ev| {
                    if let Some(PointerButton::Primary) = ev.button {
                        set_count.update(|ev| {
                            if let Some(val) = ev.checked_sub(1) {
                                *ev = val
                            }
                        });
                    }
                })
                .apply_counter_button_style(),
        )
        .with_child(label(move || format!("{}", count.get())))
        .with_child(
            Button::with_text("+")
                .with_auto_id()
                .register_handler(move |ev| {
                    if let Some(PointerButton::Primary) = ev.button {
                        set_count.update(|ev| {
                            if let Some(val) = ev.checked_add(1) {
                                *ev = val
                            }
                        });
                    }
                })
                .apply_counter_button_style(),
        )
        .with_auto_id()
        .erased()
}

fn main_view() -> AnyNewWidget {
    let (view, set_view) = signal(ViewToUse::Text);
    Align::centered(
        Flex::column()
            .with_child(
                Flex::row()
                    .with_child(
                        Button::with_text("Some text")
                            .with_auto_id()
                            .register_handler(move |ev| {
                                if let Some(PointerButton::Primary) = &ev.button {
                                    set_view.set(ViewToUse::Text);
                                }
                            })
                            .append_static_propeperty(Background::Color(AlphaColor::from_rgb8(
                                200, 100, 100,
                            )))
                            .apply_custom_styles(),
                    )
                    .with_child(
                        Button::with_text("Some checkbox")
                            .with_auto_id()
                            .register_handler(move |ev| {
                                if let Some(PointerButton::Primary) = &ev.button {
                                    set_view.set(ViewToUse::Checkbox);
                                }
                            })
                            .append_static_propeperty(Background::Color(AlphaColor::from_rgb8(
                                200, 100, 100,
                            )))
                            .apply_custom_styles(),
                    )
                    .with_child(
                        Button::with_text("Count")
                            .with_auto_id()
                            .register_handler(move |ev| {
                                if let Some(PointerButton::Primary) = &ev.button {
                                    set_view.set(ViewToUse::Count);
                                }
                            })
                            .append_static_propeperty(Background::Color(AlphaColor::from_rgb8(
                                200, 100, 100,
                            )))
                            .apply_custom_styles(),
                    )
                    .with_auto_id(),
            )
            .with_child(fragment(move || match *view.read() {
                ViewToUse::Text => text(),
                ViewToUse::Checkbox => checkbox(),
                ViewToUse::Count => counter(),
            }))
            .with_auto_id(),
    )
    .with_auto_id()
    .erased()
}

fn main() {
    Builder::default()
        .window(
            WindowBuilder::new(main_view)
                .with_title("Fragment")
                .base_color(WHEAT),
        )
        .run()
        .unwrap();
}
