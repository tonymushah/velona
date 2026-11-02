use clonelicious::clone;
use futures_util::FutureExt;
use velona::{
    AnyNewWidget,
    masonry::{
        core::Widget,
        widgets::{Button, Flex, Label, SizedBox},
    },
    reactive::traits::Read,
    widgets::{View, label::IntoLabel, sized_box::NewSizedBoxExt},
};
use velona_reactive_resource::Resource;

use crate::{
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
