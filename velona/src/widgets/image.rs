use masonry::{
    core::{ArcStr, NewWidget},
    peniko::ImageBrush,
    widgets::Image,
};
use reactive_graph::effect::Effect;

use crate::NewWidgetExt;

/// A [`NewWidget<Image>`] trait extension
///
/// PS: You might not need this in most cases. Use `lazy_image` instead.
pub trait NewImageExt {
    /// make the image_data reactive
    fn image_data<F, I>(self, img: F) -> Self
    where
        F: Fn() -> I + 'static,
        I: Into<ImageBrush>;
    /// Specifies whether the image is decorative, meaning it doesn’t have meaningful content and is only for visual presentation.
    ///
    /// If `is_decorative` returns `true`, the image will be ignored by screen readers.
    fn decorative<F>(self, is_decorative: F) -> Self
    where
        F: Fn() -> bool + 'static;
    /// Sets the text that will describe the image to screen readers.
    ///
    /// Users are encouraged to set alt text for the image. If possible, the alt-text should succinctly describe what the image represents.
    ///
    /// If the image is decorative users should set alt text to "". If it’s too hard to describe through text, the alt text should be left unset. This allows accessibility clients to know that there is no accessible description of the image content.
    fn with_alt_text<F, S>(self, alt_text: F) -> Self
    where
        F: Fn() -> Option<S> + 'static,
        S: Into<ArcStr> + 'static;
}

impl NewImageExt for NewWidget<Image> {
    fn image_data<F, I>(self, img: F) -> Self
    where
        F: Fn() -> I + 'static,
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

    fn decorative<F>(self, is_decorative: F) -> Self
    where
        F: Fn() -> bool + 'static,
    {
        let wref = self.create_velona_ref().disarm();
        Effect::new(move || {
            let decorative = is_decorative();
            let _ = wref
                .edit_local_now(|mut widget_mut| {
                    Image::set_decorative(&mut widget_mut, decorative);
                })
                .inspect_err(|err| {
                    log::error!("cannot edit image data => {err}");
                });
        });
        self
    }

    fn with_alt_text<F, S>(self, alt_text: F) -> Self
    where
        F: Fn() -> Option<S> + 'static,
        S: Into<ArcStr> + 'static,
    {
        let wref = self.create_velona_ref().disarm();
        Effect::new(move || {
            let alt_text = alt_text();
            let _ = wref
                .edit_local_now(|mut widget_mut| {
                    Image::set_alt_text(&mut widget_mut, alt_text);
                })
                .inspect_err(|err| {
                    log::error!("cannot edit image data => {err}");
                });
        });
        self
    }
}
