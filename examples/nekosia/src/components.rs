mod image_metadata;
mod image_of_the_day;
mod routes;
mod title_bar;

use std::time::Duration;

use color::AlphaColor;
use drag_resize_regions::ResizeRegion;
use masonry_widget_wrappers::WidgetWrapper;
use velona::{
    NewWidgetExt,
    masonry::{
        core::Widget,
        layout::AsUnit,
        properties::{Background, BorderColor, BorderWidth, CornerRadius},
        widgets::{Align, Flex, Label},
    },
    reactive::traits::Get,
    suspense::Suspsense,
    widgets::{View, align::IntoAlign, label::IntoLabel, portal::IntoPortal},
};
use velona_tokio::expect_tokio_handle;

use crate::{
    app_ctx::{AppStylesStoreStoreFields, TitleBarStyleStoreFields, init_app_styles_store},
    client::{NekosiaClient, provide_client},
    components::routes::routes,
    image_cache::{ImageCache, provide_image_cache},
};

pub fn window_content() -> impl View {
    let app_styles = init_app_styles_store();
    ResizeRegion::new(
        Flex::column()
            .with_fixed(title_bar::title_bar().into_new_widget())
            .with(
                Suspsense {
                    fallback: Align::centered(Label::new("Loading").prepare()).prepare(),
                }
                .from_async_fn(real_window_content)
                .into_new_widget()
                .into_portal()
                .content_must_fill(true)
                .constrain_horizontal(true)
                .prepare(),
                1.,
            ),
    )
    .into_new_widget()
    .with_props(Background::Color(AlphaColor::from_rgb8(0xff, 0xff, 0xff)))
    .with_props(BorderWidth::all(5.px()))
    .with_props_reactive(move || {
        BorderColor::new(app_styles.title_bar_style().border_color().get())
    })
    .with_props(CornerRadius::all(3.px()))
}

async fn load_client() -> anyhow::Result<NekosiaClient> {
    let tokio_handle = expect_tokio_handle();
    let reqwes_client = tokio_handle
        .spawn(async {
            reqwest::Client::builder()
                .brotli(true)
                .zstd(true)
                .gzip(true)
                .connect_timeout(Duration::from_secs(40))
                .build()
        })
        .await??;
    let client = NekosiaClient::new(reqwes_client);
    Ok(client)
}

async fn real_window_content() -> impl View {
    match load_client().await {
        Ok(client) => {
            provide_image_cache(ImageCache::new(
                client.use_reqwest_client().clone(),
                expect_tokio_handle(),
            ));
            provide_client(client);

            after_client_loaded().into_erased()
        }
        Err(err) => format!("{err}")
            .into_label()
            .prepare()
            .align_centered()
            .prepare()
            .erased(),
    }
}

fn after_client_loaded() -> impl View {
    routes()
}
