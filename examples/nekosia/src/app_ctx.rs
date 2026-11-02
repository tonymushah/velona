use std::sync::Arc;

use color::{
    AlphaColor,
    palette::css::{BLACK, WHITE},
};
use reactive_stores::Store;
use velona::{
    masonry::{
        core::{NewWidget, Selector},
        layout::AsUnit,
        peniko::Color,
        properties::{Background, BorderColor, BorderWidth, CornerRadius, Padding},
        widgets::Button,
    },
    reactive::{
        owner::{expect_context, provide_context},
        traits::Get,
    },
    scoped_styling::{ApplyScopedStyles, ScopedPropstack, scoped_propstack},
};

#[derive(Debug, Store)]
pub struct TitleBarActionButtonStyle {
    hover: Background,
    active: Background,
}

#[derive(Debug, Store)]
pub struct TitleBarActionButtonsStyle {
    base_background: Background,
    min: TitleBarActionButtonStyle,
    max: TitleBarActionButtonStyle,
    close: TitleBarActionButtonStyle,
}

impl Default for TitleBarActionButtonsStyle {
    fn default() -> Self {
        TitleBarActionButtonsStyle {
            base_background: Background::Color(AlphaColor::from_rgb8(0xaa, 0xaa, 0xaa)),
            min: TitleBarActionButtonStyle {
                hover: Background::Color(AlphaColor::from_rgb8(0xaa, 0xff, 0xff)),
                active: Background::Color(AlphaColor::from_rgb8(0x22, 0xff, 0xff)),
            },
            max: TitleBarActionButtonStyle {
                hover: Background::Color(AlphaColor::from_rgb8(0xaa, 0xff, 0x99)),
                active: Background::Color(AlphaColor::from_rgb8(0x22, 0xff, 0x22)),
            },
            close: TitleBarActionButtonStyle {
                hover: Background::Color(AlphaColor::from_rgb8(0xff, 0xaa, 0xaa)),
                active: Background::Color(AlphaColor::from_rgb8(0xff, 0x22, 0x22)),
            },
        }
    }
}

#[derive(Debug, Store)]
pub struct TitleBarStyle {
    text_color: Color,
    background: Background,
    border_color: Color,
    action_buttons: TitleBarActionButtonsStyle,
}

impl Default for TitleBarStyle {
    fn default() -> Self {
        TitleBarStyle {
            text_color: WHITE,
            background: Background::Color(BLACK),
            border_color: BLACK,
            action_buttons: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ButtonsPropStack(pub Arc<ScopedPropstack>);

#[derive(Debug, Store)]
pub struct ContentButtonStyleUnit {
    active: Background,
    hover: Background,
    border_color: BorderColor,
}

#[derive(Debug, Store)]
pub struct ContentButtonStyles {
    primary: ContentButtonStyleUnit,
}

impl Default for ContentButtonStyles {
    fn default() -> Self {
        ContentButtonStyles {
            primary: ContentButtonStyleUnit {
                active: Background::Color(AlphaColor::from_rgba8(0x22, 0x22, 0x22, 0xff)),
                hover: Background::Color(AlphaColor::from_rgba8(0xdd, 0xdd, 0xdd, 0xff)),
                border_color: BorderColor::new(AlphaColor::from_rgba8(0, 0, 0, 0xff)),
            },
        }
    }
}

#[derive(Debug, Store)]
pub struct AppStylesStore {
    content_color: Color,
    content_background: Background,
    title_bar_style: TitleBarStyle,
    content_buttons_styles: ContentButtonStyles,
}

pub trait ApplyAppButtonStyle {
    fn apply_primary_button_style(self) -> Self;
}

impl ApplyAppButtonStyle for NewWidget<Button> {
    fn apply_primary_button_style(self) -> Self {
        let style = expect_content_button_styles();
        self.with_classes(["base_button".to_string(), "primary".to_owned()])
            .apply(&*style.0)
    }
}

pub fn init_app_styles_store() -> Store<AppStylesStore> {
    let data = AppStylesStore {
        content_background: Background::Color(WHITE),
        content_color: BLACK,
        title_bar_style: TitleBarStyle::default(),
        content_buttons_styles: Default::default(),
    };
    let store = Store::new(data);
    provide_context(store);

    let propstack = scoped_propstack! {
        ["base_button"] {
            Padding::from_vh(4.px(), 8.px());
            BorderWidth::all(2.px());
            CornerRadius::all(4.px())
        }
        ["base_button", "primary"] {
            move || store.content_buttons_styles().primary().border_color().get();
            #[hover]
            move || store.content_buttons_styles().primary().hover().get();
            #[active]
            move || store.content_buttons_styles().primary().active().get();
        }
    };
    provide_context(ButtonsPropStack(Arc::new(propstack)));

    store
}

pub fn expect_app_styles_store() -> Store<AppStylesStore> {
    expect_context()
}

fn expect_content_button_styles() -> ButtonsPropStack {
    expect_context()
}
