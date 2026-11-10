use velona::{
    components::lazy_image,
    masonry::{
        self,
        core::Widget,
        dpi::PhysicalSize,
        layout::AsUnit,
        peniko::{ImageBrush, ImageSampler},
        properties::{Dimensions, Padding},
        widgets::Flex,
    },
    reactive::{callback::UnsyncCallback, computed::Memo, traits::Read},
    widgets::{View, align::IntoAlign, label::IntoLabel, sized_box::IntoNewSizedBox},
};

use crate::{
    client::objects::image::ImageObject,
    image_cache::{ImageCacheEntryKey, expect_image_cache},
};

pub fn image_metadata_flex(obj: &ImageObject, flex: Option<Flex>) -> Flex {
    // log::debug!("{:#?}", obj);
    flex.unwrap_or(Flex::column())
        .with_fixed(
            format!(
                "Rating: {}",
                match obj.rating {
                    crate::client::objects::Rating::Safe => "Safe",
                    crate::client::objects::Rating::Suggestive => "Suggestive",
                }
            )
            .into_label()
            .prepare(),
        )
        .with_fixed(format!("Category: {}", obj.category).into_label().prepare())
        .with_fixed(
            format!("Artist: {}", obj.attribution.artist.username)
                .into_label()
                .prepare(),
        )
        .with_fixed(
            format!("Copyright: {}", obj.attribution.copyright)
                .into_label()
                .prepare(),
        )
}

pub fn image_view(obj: &ImageObject) -> impl View + use<> {
    let urls = obj.image.clone();
    // let metadata = obj.metadata.clone();
    let image_cache = expect_image_cache();
    let maybe_image = image_cache
        .image_of_as_res(ImageCacheEntryKey {
            url: urls.compressed.url,
            size: Some(PhysicalSize::new(500, 500)),
        })
        .into_resource();

    let image_data = Memo::new(move |_| {
        maybe_image
            .read()
            .as_ref()
            .and_then(|d| d.as_ref().ok())
            .map(|data| ImageBrush {
                image: data.clone(),
                sampler: ImageSampler::new().with_quality(masonry::peniko::ImageQuality::High),
            })
    });

    let source = obj.anime.clone();
    let flex = Flex::column()
        .with_fixed(
            lazy_image(
                image_data,
                Some(velona::components::LazyImageOptions {
                    fallback: Some(UnsyncCallback::new(move |_| {
                        let mut to_show: String = "Loading".into();
                        match (source.character.as_ref(), source.title.as_ref()) {
                            (None, Some(title)) => {
                                to_show = format!("{to_show} something from {title}...");
                            }
                            (Some(character), None) => {
                                to_show = format!("{to_show} {character}...");
                            }
                            (Some(character), Some(title)) => {
                                to_show = format!("{to_show} {character} from {title}...");
                            }
                            (None, None) => {
                                to_show = format!("{to_show}...");
                            }
                        }
                        to_show
                            .into_label()
                            .prepare()
                            .align_centered()
                            .prepare()
                            .erased()
                    })),
                    object_fit: None,
                }),
            )
            .with_props(Dimensions::new(
                masonry::layout::Dim::Ratio(0.5),
                masonry::layout::Dim::MinContent,
            )),
        )
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
    image_metadata_flex(obj, Some(flex))
        .prepare()
        .with_props(Padding::bottom(10.px()))
}
