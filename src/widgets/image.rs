use masonry::{core::NewWidget, peniko::ImageBrush, widgets::Image};
use reactive_graph::effect::Effect;

use crate::NewWidgetExt;

/// A [`NewWidget<Image>`] trait extension
///
/// PS: You might not need this in most cases. Use `lazy_image` instead.
pub trait NewImageExt {
    /// make the image_data reactive
    fn image_data<F, I>(self, img: F) -> Self
    where
        F: FnMut() -> I + 'static,
        I: Into<ImageBrush>;
}

impl NewImageExt for NewWidget<Image> {
    fn image_data<F, I>(self, mut img: F) -> Self
    where
        F: FnMut() -> I + 'static,
        I: Into<ImageBrush>,
    {
        let wref = self.create_velona_ref().disarm();
        Effect::new(move || {
            let new_image = img();
            let _ = wref
                .edit_local_now(|mut widget_mut| {
                    Image::set_image_data(&mut widget_mut, new_image);
                })
                .inspect_err(|err| {
                    log::error!("cannot edit image data => {err}");
                });
        });
        self
    }
}
