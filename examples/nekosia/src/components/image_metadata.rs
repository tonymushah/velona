use velona::{
    masonry::{core::Widget, widgets::Flex},
    widgets::label::IntoLabel,
};

use crate::client::objects::image::ImageObject;

pub fn image_metadata_flex(obj: &ImageObject, flex: Option<Flex>) -> Flex {
    // log::debug!("{:#?}", obj);
    flex.unwrap_or(Flex::column())
        .with_fixed(format!("Category: {}", obj.category).into_label().prepare())
}
