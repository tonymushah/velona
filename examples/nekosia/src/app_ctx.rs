use color::{
    AlphaColor,
    palette::css::{BLACK, WHITE},
};
use reactive_stores::Store;
use velona::{
    masonry::{peniko::Color, properties::Background},
    reactive::owner::{expect_context, provide_context},
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

#[derive(Debug, Store)]
pub struct AppStylesStore {
    content_color: Color,
    content_background: Background,
    title_bar_style: TitleBarStyle,
}

pub fn init_app_styles_store() -> Store<AppStylesStore> {
    let data = AppStylesStore {
        content_background: Background::Color(WHITE),
        content_color: BLACK,
        title_bar_style: TitleBarStyle::default(),
    };
    let store = Store::new(data);
    provide_context(store);
    store
}

pub fn expect_app_styles_store() -> Store<AppStylesStore> {
    expect_context()
}
