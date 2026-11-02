mod image_metadata;
mod image_of_the_day;
mod routes;
mod title_bar;

use std::time::Duration;

use color::{AlphaColor, palette::css::BLACK};
use drag_resize_regions::ResizeRegion;
use masonry_raw_box::RawBox;
use masonry_widget_wrappers::WidgetWrapper;
use velona::{
    masonry::{
        core::Widget,
        layout::AsUnit,
        properties::{Background, BorderColor, BorderWidth, CornerRadius},
        widgets::{Align, Flex, Label},
    },
    suspense::Suspsense,
    widgets::{View, align::IntoAlign, label::IntoLabel, portal::IntoPortal},
};
use velona_tokio::expect_tokio_handle;

use crate::{
    client::{NekosiaClient, provide_client},
    components::routes::routes,
};

pub fn window_content() -> impl View {
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
    .with_props((BorderWidth::all(5.px()), BorderColor::new(BLACK)))
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
            provide_client(client);
            RawBox::new(after_client_loaded().into_new_widget()).prepare()
        }
        Err(err) => RawBox::new(
            format!("{err}")
                .into_label()
                .prepare()
                .align_centered()
                .prepare(),
        )
        .prepare(),
    }
}

fn after_client_loaded() -> impl View {
    routes()
}
