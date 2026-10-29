use std::sync;

use clonelicious::clone;
use futures_util::{FutureExt, TryFutureExt};
use velona::{
    components::lazy_image,
    masonry::{
        self,
        core::Widget,
        peniko::{Blob, ImageBrush, ImageData, ImageSampler},
        widgets::Flex,
    },
    reactive::{computed::Memo, traits::Read},
    widgets::{View, label::IntoLabel, sized_box::IntoNewSizedBox},
};
use velona_reactive_resource::Resource;

use crate::client::{expect_client, objects::image::ImageObject};

pub fn image_metadata_flex(obj: &ImageObject, flex: Option<Flex>) -> Flex {
    // log::debug!("{:#?}", obj);
    flex.unwrap_or(Flex::column())
        .with_fixed(format!("Category: {}", obj.category).into_label().prepare())
        .with_fixed(
            format!("Author: {}", obj.attribution.artist.username)
                .into_label()
                .prepare(),
        )
        .with_fixed(
            format!("Copyright: {}", obj.attribution.copyright)
                .into_label()
                .prepare(),
        )
}

pub fn image_view(obj: &ImageObject) -> impl View {
    let urls = obj.image.clone();
    // let metadata = obj.metadata.clone();
    let client = expect_client();
    let maybe_image = Resource::new(move || {
        velona_tokio::tokio_spawn(clone!(client, urls => async move {
            let img = image::load_from_memory_with_format(
                &client
                    .read()
                    .await
                    .use_reqwest_client()
                    .get(urls.original.url.clone())
                    .send()
                    .await?
                    .bytes()
                    .await?,
                image::ImageFormat::from_extension(&urls.original.extension).ok_or(
                    anyhow::format_err!("Invalid extension {}", urls.original.extension),
                )?,
            )?;
            anyhow::Ok(img)
        }))
        .map_err(anyhow::Error::from)
        .map(|res| res.flatten())
    });

    let image_data = Memo::new(move |_| {
        maybe_image
            .read()
            .as_ref()
            .and_then(|d| d.as_ref().ok())
            .map(|data| {
                let data = data.to_rgba8();
                let width = data.width();
                let height = data.height();
                let mut data_buf = data.into_vec();
                data_buf.shrink_to_fit();
                ImageBrush {
                    image: ImageData {
                        data: Blob::new(sync::Arc::new(data_buf)),
                        format: masonry::peniko::ImageFormat::Rgba8,
                        alpha_type: masonry::peniko::ImageAlphaType::Alpha,
                        width,
                        height,
                    },
                    sampler: ImageSampler::new().with_quality(masonry::peniko::ImageQuality::Low),
                }
            })
    });

    let flex = Flex::column()
        .with_fixed(lazy_image(image_data, None))
        .with_fixed(
            (move || {
                let read = maybe_image.read();
                if let Err(err) = read.as_ref()? {
                    Some(format!("{err}").into_label().prepare())
                } else {
                    None
                }
            })
            .into_new_sized_box(),
        );
    image_metadata_flex(obj, Some(flex)).prepare()
}
