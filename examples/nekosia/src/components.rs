mod image_metadata;
mod title_bar;

use std::time::Duration;

use clonelicious::clone;
use color::{AlphaColor, palette::css::BLACK};
use drag_resize_regions::ResizeRegion;
use futures_util::FutureExt;
use masonry_raw_box::RawBox;
use masonry_widget_wrappers::WidgetWrapper;
use velona::{
    masonry::{
        core::Widget,
        layout::AsUnit,
        properties::{Background, BorderColor, BorderWidth, CornerRadius},
        widgets::{Align, Button, Flex, Label, SizedBox},
    },
    reactive::traits::Read,
    suspense::Suspsense,
    widgets::{
        View, align::IntoAlign, label::IntoLabel, portal::IntoPortal, sized_box::NewSizedBoxExt,
    },
};
use velona_reactive_resource::Resource;
use velona_tokio::expect_tokio_handle;

use crate::{
    client::{NekosiaClient, endpoint::image_of_day::ImageOfTheDay, expect_client, provide_client},
    components::image_metadata::image_metadata_flex,
};

pub fn window_content() -> impl View {
    ResizeRegion::new(
        Flex::column()
            .with_fixed(title_bar::title_bar().into_new_widget())
            .with_fixed(
                Suspsense {
                    fallback: Align::centered(Label::new("Loading").prepare()).prepare(),
                }
                .from_async_fn(real_window_content)
                .into_new_widget()
                .into_portal()
                .content_must_fill(true)
                .constrain_horizontal(true)
                .prepare(),
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
    let client = expect_client();
    let resource = Resource::new(move || {
        velona_tokio::tokio_spawn(clone!(client => async move {
            client.read().await.send_endpoint(ImageOfTheDay).await
        }))
        .map(|res| anyhow::Ok(res??))
    });
    Flex::column()
        .with_fixed(
            Flex::row()
                .with_fixed(Button::new("Refetch".into_label().prepare()).prepare())
                .prepare(),
        )
        .with_fixed(
            SizedBox::new("Loading".into_label().prepare())
                .prepare()
                .child_opt(move || {
                    // log::debug!("aaaaaaaaaa");
                    let read = resource.read();
                    // log::trace!("read done");
                    let image_of_day = read.as_ref()?;
                    // log::debug!("Not empty");
                    Some(match image_of_day {
                        Ok(metadata) => image_metadata_flex(&metadata.object, None)
                            .prepare()
                            .erased(),
                        Err(err) => {
                            // log::error!("{err}");
                            Label::new(format!("{err}")).prepare().erased()
                        }
                    })
                }),
        )
        .prepare()
}
