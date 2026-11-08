use color::AlphaColor;
use velona::{
    AnyNewWidget, NewWidgetExt,
    masonry::{
        core::{NewWidget, Selector, Widget},
        layout::AsUnit,
        properties::{Background, BorderColor, BorderWidth, Padding},
        widgets::{Button, Flex, Split},
    },
    reactive::computed::Memo,
    scoped_styling::{ApplyScopedStyles, ScopedPropstack, scoped_propstack},
    widgets::{
        View, align::IntoAlign, button::NewButtonPressEventsExt, label::IntoLabel,
        portal::IntoPortal,
    },
};
use velona_router::{Route, Router, components::outlet, use_navigation_controller};

use crate::components::{booru_page, image_of_the_day};

pub fn routes() -> impl View {
    Router::default()
        .route(
            Route::layout(routes_layout)
                .child(Route::index(image_of_the_day::image_of_the_day))
                .child(
                    Route::static_("booru", booru_page::booru_layout)
                        .child(Route::index(|| redirect("/booru/random")))
                        .child(Route::params("category", booru_page::booru_page)),
                )
                .child(Route::wildcard("any", not_found)),
        )
        .into_view()
}

fn redirect(route: &'static str) -> AnyNewWidget {
    if let Err(err) = use_navigation_controller().push(route) {
        log::error!("{err}");
    }

    ().into_erased()
}

fn routes_layout() -> AnyNewWidget {
    Split::new(
        sidebar_content()
            .into_erased()
            .into_portal()
            .content_must_fill(true)
            .prepare(),
        outlet()
            .into_portal()
            .content_must_fill(true)
            .constrain_horizontal(true)
            .prepare()
            .with_props(Padding::from_vh(0.px(), 10.px())),
    )
    .solid_bar(true)
    .draggable(false)
    .split_point_from_start(150.px())
    .prepare()
    .erased()
}

fn sidebar_content() -> impl View {
    let btn_style = scoped_propstack! {
        [] {
            BorderColor::new(AlphaColor::from_rgb8(0xfa, 0xfa, 0xfa));
            Background::Color(AlphaColor::from_rgb8(0xfa, 0xfe, 0xfd));
            BorderWidth::all(1.px());
        }
        ["active-route"] {
            Background::Color(AlphaColor::from_rgb8(0xef, 0xef, 0xef))
        }
    };
    Flex::column()
        .cross_axis_alignment(velona::masonry::properties::types::CrossAxisAlignment::Center)
        .main_axis_alignment(velona::masonry::properties::types::MainAxisAlignment::Start)
        .with_fixed(sidebar_button("/".into(), "Image of the day".into(), false).apply(&btn_style))
        .with_fixed(sidebar_button("/booru".into(), "Booru".into(), true).apply(&btn_style))
        .prepare()
}

fn sidebar_button(route: String, label: String, start_with: bool) -> NewWidget<Button> {
    let n_c = use_navigation_controller();
    let current = n_c.current();
    let is_active = {
        let route = route.clone();
        Memo::new(move |_| {
            let current = current();
            if start_with {
                current.path() == route
            } else {
                current.path().starts_with(&route)
            }
        })
    };
    Button::new(label.into_label().prepare())
        .prepare()
        .on_primary(move || {
            if let Err(err) = n_c.push(&route) {
                log::error!("{err}");
            }
        })
        .class_opt(move || {
            if is_active() {
                Some("active-route".into())
            } else {
                None
            }
        })
        .disabled_reactive(is_active)
}

fn not_found() -> AnyNewWidget {
    "Not found or Not yet implement"
        .into_label()
        .prepare()
        .align_centered()
        .prepare()
        .erased()
}
