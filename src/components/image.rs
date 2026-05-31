use masonry::{
    core::Widget,
    peniko::ImageBrush,
    properties::ObjectFit,
    widgets::{Image, SizedBox},
};
use reactive_graph::{
    callback::{Callable, UnsyncCallback},
    effect::Effect,
};

use crate::{
    AnyNewWidget, NewWidgetExt,
    widget_ref::{EditWidgetLocalError, VelonaWidgetRef},
};

#[derive(Default)]
// We use a callback here for generics simplicity
pub struct LazyImageOptions {
    /// A fallback widget when the image is missing or loading
    pub fallback: Option<UnsyncCallback<(), AnyNewWidget>>,
    /// The image [`object fit`](ObjectFit)
    pub object_fit: Option<UnsyncCallback<(), ObjectFit>>,
}

fn change_box_child_element(
    isr: &VelonaWidgetRef<SizedBox>,
    maybe_element: Option<AnyNewWidget>,
) -> Result<(), EditWidgetLocalError> {
    isr.edit_local_now(|mut this| {
        if let Some(element) = maybe_element {
            SizedBox::set_child(&mut this, element);
        } else {
            SizedBox::remove_child(&mut this);
        }
    })
    .inspect_err(|e| log::error!("Unable to edit SizedBox => {e}"))
}

/// This component should be the way you should show images in Velona.
///
/// If image_data return [`None`], it means that your image is not available yet
/// and will show a [`fallback`](LazyImageOptions::fallback) if provided
// TODO add add example
pub fn lazy_image<Ifn, I>(mut image_data: Ifn, options: Option<LazyImageOptions>) -> AnyNewWidget
where
    Ifn: FnMut() -> Option<I> + 'static,
    I: Into<ImageBrush>,
{
    let options = options.unwrap_or_default();
    let LazyImageOptions {
        fallback: maybe_fallback,
        object_fit,
    } = options;
    let s_box = SizedBox::empty().with_auto_id();
    let s_box_ref = s_box.create_velona_ref();
    Effect::new(
        move |maybe_current_image: Option<Option<VelonaWidgetRef<Image>>>| {
            // We are reusing the Widget Ref here from the last effect run to prevent reallocating a new widget Image everytime
            let maybe_current_image_ref = maybe_current_image.flatten();
            // Run the image data function
            let image_data = image_data().map(|i| i.into());
            let Some(image_data) = image_data else {
                // check if there is a fallback element
                if let Some(fallback) = maybe_fallback {
                    let s_box_ref = s_box_ref.clone();
                    // We warp the fallback inside a fallback to p
                    // I could have used another SizedBox but it would be more efficient that way
                    Effect::new(move || {
                        // I know that i should have used `run` but i want to be safe here.
                        let maybe_fallback = fallback.try_run(());
                        let _ = change_box_child_element(&s_box_ref, maybe_fallback);
                    });
                } else {
                    // Remove set nothing if there is not call back.
                    let _ = change_box_child_element(&s_box_ref, None);
                }
                return None;
            };
            let image_ref = if let Some(current_image_ref) = maybe_current_image_ref {
                // If there is already some image showing we just update it.
                let _ = current_image_ref
                    .edit_local_now(|mut this| {
                        Image::set_image_data(&mut this, image_data);
                    })
                    .inspect_err(|e| log::error!("Cannot set image data => {e}"));
                current_image_ref
            } else {
                // If not, we create a new one
                let image = Image::new(image_data).with_auto_id();
                let image_ref = image.create_velona_ref();
                // swap the sized box element
                let _ = change_box_child_element(&s_box_ref, Some(image.erased()));
                image_ref
            };
            // Change the object fit reactivly in another effect for effiency
            if let Some(object_fit) = object_fit {
                let image_ref = image_ref.clone();
                Effect::new(move || {
                    let object_fit = object_fit.try_run(());
                    if let Some(object_fit) = object_fit {
                        let _ = image_ref
                            .edit_local_now(|mut this| {
                                this.insert_prop(object_fit);
                            })
                            .inspect_err(|e| log::error!("Cannot set image object fit => {e}"));
                    }
                });
            }
            Some(image_ref)
        },
    );
    s_box.erased()
}

// TODO add image from path
// TODO image from [`image::DynamicImage`]
