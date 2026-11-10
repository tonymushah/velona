mod image_fetch;
mod tags;

use futures_util::FutureExt;
use heck::ToKebabCase;
use velona::{
    AnyNewWidget,
    masonry::{
        core::{StyleProperty, Widget},
        widgets::Flex,
    },
    reactive::traits::Read,
    widgets::{View, label::IntoLabel, sized_box::IntoNewSizedBox},
};
use velona_reactive_resource::Resource;
use velona_tokio::expect_tokio_handle;

use crate::{
    booru::provide_booru_context,
    client::{endpoint::tags::GetTags, expect_client},
};

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub(super) enum FetchError {
    Client(#[from] crate::client::error::Error),
    TokioJoin(#[from] tokio::task::JoinError),
}

pub fn booru_page() -> AnyNewWidget {
    booru_page_inner().into_erased()
}

pub(super) type ResourceTagsResult = Resource<Result<crate::booru::BooruTagsAvailable, FetchError>>;

fn booru_page_inner() -> impl View {
    let client = expect_client();
    let tokio_handle = expect_tokio_handle();
    provide_booru_context();
    let tags: ResourceTagsResult = Resource::new(move || {
        let tokio_handle = tokio_handle.clone();
        let client = client.clone();
        tokio_handle
            .spawn(async move {
                let read = client.read().await;
                read.wait_rate_limiter().await;
                read.send_endpoint(GetTags)
                    .await
                    .map(|mut e| {
                        e.anime.iter_mut().for_each(|tag| {
                            *tag = tag.to_kebab_case();
                        });
                        e.characters.iter_mut().for_each(|tag| {
                            *tag = tag.to_kebab_case();
                        });
                        e.anime.iter_mut().for_each(|tag| {
                            *tag = tag.to_kebab_case();
                        });
                        e
                    })
                    .map(|data| crate::booru::BooruTagsAvailable {
                        tags: data
                            .tags
                            .into_iter()
                            .map(|e| e.into_boxed_str().into())
                            .collect(),
                        anime: data
                            .anime
                            .into_iter()
                            .map(|e| e.into_boxed_str().into())
                            .collect(),
                        characters: data
                            .characters
                            .into_iter()
                            .map(|e| e.into_boxed_str().into())
                            .collect(),
                    })
                    .inspect(|e| {
                        log::info!("anime: {}", e.anime.len());
                        log::info!("tags: {}", e.tags.len());
                        log::info!("characters: {}", e.characters.len());
                    })
            })
            .map(|e| Ok::<_, FetchError>(e??))
    });
    let mut flex = Flex::column()
        .with_fixed(show_fetch_error(tags).into_new_widget())
        .with_fixed(tags::tags_view(tags).into_new_widget());

    flex = image_fetch::image_fetching(flex);

    flex.prepare()
}

pub fn show_fetch_error<T: 'static>(resource: Resource<Result<T, FetchError>>) -> impl View {
    (move || {
        let tags_read = resource.read();
        let err = tags_read.as_ref()?.as_ref().err()?;
        Some(
            Flex::column()
                .cross_axis_alignment(
                    velona::masonry::properties::types::CrossAxisAlignment::Center,
                )
                .with_fixed(format!("{err}").into_label().prepare())
                .with_fixed(
                    "Details"
                        .into_label()
                        .with_style(StyleProperty::Underline(true))
                        .prepare(),
                )
                .with_fixed(match err {
                    FetchError::Client(crate::client::error::Error::NekosiaRequest(err)) => {
                        format!("{} (status: {})", err.message, err.status)
                            .into_label()
                            .prepare()
                    }
                    FetchError::Client(crate::client::error::Error::Reqwest(err)) => format!(
                        "Reqwest err: [url: {}] [type: {}]",
                        err.url().map(|e| e.as_str()).unwrap_or("(nothing)"),
                        if err.is_body() {
                            "body"
                        } else if err.is_builder() {
                            "builder"
                        } else if err.is_connect() {
                            "connect"
                        } else if err.is_decode() {
                            "decode"
                        } else if err.is_dns() {
                            "dns"
                        } else if err.is_redirect() {
                            "redirect"
                        } else if err.is_request() {
                            "request"
                        } else if err.is_status() {
                            "status"
                        } else if err.is_timeout() {
                            "timeout"
                        } else if err.is_upgrade() {
                            "upgrade"
                        } else {
                            "unknown"
                        }
                    )
                    .into_label()
                    .prepare(),
                    FetchError::Client(crate::client::error::Error::Url(url)) => {
                        format!("Url parsing error: {}", url).into_label().prepare()
                    }
                    FetchError::TokioJoin(join_error) => format!("Tokio Join error ({join_error})")
                        .into_label()
                        .prepare(),
                    FetchError::Client(crate::client::error::Error::TooManyRequests) => {
                        "Too many requests has been sent!".into_label().prepare()
                    }
                })
                .prepare(),
        )
    })
    .into_new_sized_box()
}
