mod tags;

use futures_util::FutureExt;
use velona::{
    AnyNewWidget,
    masonry::{core::Widget, widgets::Flex},
    widgets::View,
};
use velona_reactive_resource::Resource;
use velona_tokio::expect_tokio_handle;

use crate::client::{endpoint::tags::GetTags, expect_client};

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub(super) enum FetchError {
    Client(#[from] crate::client::error::Error),
    TokioJoin(#[from] tokio::task::JoinError),
}

pub fn booru_page() -> AnyNewWidget {
    booru_page_inner().into_erased()
}

type ResourceTagsResult =
    Resource<Result<crate::client::endpoint::tags::GetTagsResponse, FetchError>>;

fn booru_page_inner() -> impl View {
    let client = expect_client();
    let tokio_handle = expect_tokio_handle();
    let tags: ResourceTagsResult = Resource::new(move || {
        let tokio_handle = tokio_handle.clone();
        let client = client.clone();
        tokio_handle
            .spawn(async move { client.read().await.send_endpoint(GetTags).await })
            .map(|e| Ok::<_, FetchError>(e??))
    });
    Flex::column()
        .with_fixed(tags::tags_error(tags).into_new_widget())
        .with_fixed(tags::tags_view(tags).into_new_widget())
        .prepare()
}
