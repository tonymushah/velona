//! Various [`Image`] implementations.
//!
//! The most important thing in the module is the [`NewImageExt`]
//! which is implemented for [`NewWidget<Image>`].
//!
//! _See the [widget](Image) documentation for more information_.

use masonry::{
    core::{ArcStr, NewWidget},
    peniko::ImageBrush,
    widgets::Image,
};

use crate::NewWidgetExt;

/// A [`NewWidget<Image>`] trait extension
///
/// PS: You might not need this in most cases. Use `lazy_image` instead.
#[must_use]
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
        self.use_widget_mut(
            move || img().into(),
            |mut this, img| {
                Image::set_image_data(&mut this, img);
            },
        )
    }

    fn decorative<F>(self, is_decorative: F) -> Self
    where
        F: Fn() -> bool + 'static,
    {
        self.use_widget_mut(is_decorative, |mut this, is_decorative| {
            Image::set_decorative(&mut this, is_decorative);
        })
    }

    fn with_alt_text<F, S>(self, alt_text: F) -> Self
    where
        F: Fn() -> Option<S> + 'static,
        S: Into<ArcStr> + 'static,
    {
        self.use_widget_mut(
            move || alt_text().map(Into::<ArcStr>::into),
            move |mut this, alt_text| {
                Image::set_alt_text(&mut this, alt_text);
            },
        )
    }
}
