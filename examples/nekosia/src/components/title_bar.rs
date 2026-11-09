use clonelicious::clone;
use drag_resize_regions::TitleBarContainer;
use masonry_widget_wrappers::WidgetWrapper;
use velona::{
    NewWidgetExt,
    masonry::{
        core::{Selector, Widget},
        layout::AsUnit,
        properties::{Background, ContentColor, CornerRadius},
        widgets::{Button, Flex, Label},
    },
    reactive::traits::Get,
    scoped_styling::{ApplyScopedStyles, ScopedPropstack, scoped_propstack},
    widgets::{View, button::NewButtonPressEventsExt},
    window::expect_window,
};

use crate::{
    app_ctx::{
        AppStylesStoreStoreFields, TitleBarActionButtonStyleStoreFields,
        TitleBarActionButtonsStyleStoreFields, TitleBarStyleStoreFields, expect_app_styles_store,
    },
    svgs::{MAXIMIZE_ICON, MINUS_ICON, X_ICON, build_static_svg},
};

pub fn title_bar() -> impl View {
    let window = expect_window();
    let app_styles = expect_app_styles_store();

    let button_styles = scoped_propstack! {
        ["buttons"] {
            CornerRadius::all(3.px());
            move|| app_styles.title_bar_style().action_buttons().base_background().get()
        }
        ["buttons", "min"] {
            #[hover]
            move || app_styles.title_bar_style().action_buttons().min().hover().get();
            #[active]
            move || app_styles.title_bar_style().action_buttons().min().active().get()
        }
        ["buttons", "max"] {
            #[hover]
            move || app_styles.title_bar_style().action_buttons().max().hover().get();
            #[active]
            move || app_styles.title_bar_style().action_buttons().max().active().get()
        }
        ["buttons", "close"] {
            #[hover]
            move || app_styles.title_bar_style().action_buttons().close().hover().get();
            #[active]
            move || app_styles.title_bar_style().action_buttons().close().active().get()
        }
    };

    TitleBarContainer::new(Flex::row()
        .main_axis_alignment(velona::masonry::properties::types::MainAxisAlignment::SpaceBetween)
        .cross_axis_alignment(velona::masonry::properties::types::CrossAxisAlignment::Center)
        .with_fixed(
            Label::new("Nekosia")
                .prepare()
                .with_props_reactive(move|| ContentColor::new(app_styles.title_bar_style().text_color().get())),
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
        .with_props_reactive(move|| Background::Color(app_styles.title_bar_style().border_color().get()))
    )
    .into_new_widget()
}

fn action_button(data: &'static [u8]) -> velona::masonry::core::NewWidget<Button> {
    Button::new(build_static_svg(data).prepare())
        .prepare()
        .with_class("buttons")
}
