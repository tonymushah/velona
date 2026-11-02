use clonelicious::clone;
use futures_util::FutureExt;
use velona::{
    AnyNewWidget,
    masonry::{
        core::Widget,
        layout::AsUnit,
        properties::Padding,
        widgets::{Button, Flex, Label, SizedBox},
    },
    reactive::traits::Read,
    widgets::{View, button::NewButtonPressEventsExt, label::IntoLabel, sized_box::NewSizedBoxExt},
};
use velona_reactive_resource::Resource;

use crate::{
    app_ctx::ApplyAppButtonStyle,
    client::{endpoint::image_of_day::ImageOfTheDay, expect_client},
    components::image_metadata::image_view,
};

pub fn image_of_the_day() -> AnyNewWidget {
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
                .with_fixed(
                    Button::new("Refetch".into_label().prepare())
                        .prepare()
                        .apply_primary_button_style()
                        .on_primary(move || {
                            resource.refetch();
                        }),
                )
                .prepare()
                .with_props(Padding::from_vh(4.px(), 2.px())),
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
                        Ok(metadata) => image_view(&metadata.object).into_erased(),
                        Err(err) => {
                            // log::error!("{err}");
                            Label::new(format!("{err}")).prepare().erased()
                        }
                    })
                }),
        )
        .prepare()
        .erased()
}
