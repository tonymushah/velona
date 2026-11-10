use futures_util::FutureExt;
use velona::{
    masonry::{
        core::Widget,
        layout::AsUnit,
        properties::types::CrossAxisAlignment,
        widgets::{Button, Flex, FlexParams},
    },
    reactive::traits::{Get, Read},
    widgets::{
        View, button::NewButtonPressEventsExt, label::IntoLabel, sized_box::IntoNewSizedBox,
    },
};
use velona_reactive_resource::Resource;
use velona_tokio::expect_tokio_handle;

use crate::{
    app_ctx::ApplyAppButtonStyle,
    booru::{BooruContextStoreFields, BooruTagsSelectionStoreFields, expect_booru_context},
    client::{endpoint::image_caterory::GetRandomFromCategory, expect_client},
    components::{
        booru_page::{FetchError, show_fetch_error},
        image_metadata::image_view,
    },
};

pub fn image_fetching(flex: Flex) -> Flex {
    let ctx = expect_booru_context();
    let client = expect_client();
    let tokio_handle = expect_tokio_handle();

    let image_fetch = Resource::new(move || {
        let mut endpoint =
            GetRandomFromCategory::new(ctx.selected_category().get().unwrap_or("random".into()));
        endpoint
            .session(crate::client::endpoint::image_caterory::SessionType::Id(
                ctx.session().get(),
            ))
            .additional_tags(
                ctx.selection()
                    .additional_tags()
                    .read()
                    .iter()
                    .map(|e| String::from(&**e))
                    .collect(),
            )
            .blacklisted_tags(
                ctx.selection()
                    .excluded_tags()
                    .read()
                    .iter()
                    .map(|e| String::from(&**e))
                    .collect(),
            )
            .rating(ctx.rating().get());
        let client = client.clone();
        tokio_handle
            .spawn(async move {
                let client_read = client.read().await;
                client_read.wait_rate_limiter().await;
                client_read.send_endpoint(endpoint).await
            })
            .map(|e| Ok::<_, FetchError>(e??))
    });
    flex.with_fixed_spacer(12.px())
        .with(
            Button::new("Refetch".into_label().prepare())
                .prepare()
                .apply_primary_button_style()
                .on_primary(move || {
                    image_fetch.refetch();
                }),
            FlexParams::new(0., None, CrossAxisAlignment::Center),
        )
        .with_fixed(show_fetch_error(image_fetch).into_new_widget())
        .with_fixed_spacer(8.px())
        .with_fixed(
            (move || {
                let if_read = image_fetch.read();
                let data = if_read.as_ref()?.as_ref().ok()?;
                let image = data.images.first()?;
                Some(image_view(image))
            })
            .into_new_sized_box(),
        )
}
