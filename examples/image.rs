use std::{process, sync};

use masonry::{
    core::Widget,
    palette::css::WHITE,
    peniko::{Blob, ImageBrush, ImageData, ImageSampler},
    properties::types::Length,
    widgets::{Align, Flex, Label, SizedBox, Spinner},
};
use reactive_graph::{
    signal::arc_signal,
    traits::{Get, Set},
};
use velona::{
    AnyNewWidget, Builder, WindowBuilder,
    components::{LazyImageOptions, lazy_image},
};

fn new_view() -> AnyNewWidget {
    let (image_data, set_image_data) = arc_signal(None::<ImageBrush>);
    reactive_graph::spawn(async move {
        // TODO Fix buffer overflow panic
        match image::open("assets/image1.png") {
            Ok(data) => {
                let data = data.into_rgba8();
                let width = data.width();
                let height = data.height();
                let mut data_buf = data.into_vec();
                data_buf.shrink_to_fit();
                set_image_data.set(Some(ImageBrush {
                    image: ImageData {
                        data: Blob::new(sync::Arc::new(data_buf)),
                        format: masonry::peniko::ImageFormat::Rgba8,
                        alpha_type: masonry::peniko::ImageAlphaType::Alpha,
                        width,
                        height,
                    },
                    sampler: ImageSampler::new().with_quality(masonry::peniko::ImageQuality::High),
                }));
                // println!("Runned shit");
            }
            Err(err) => {
                log::error!("Cannot load image {err}");
            }
        }
    });
    Align::centered(
        Flex::column()
            .with_child(Label::new("SOme image").with_auto_id().erased())
            .with_child(lazy_image(
                move || image_data.get(),
                Some(LazyImageOptions {
                    fallback: Some(
                        (|| {
                            SizedBox::new(Spinner::new().with_auto_id())
                                .width(Length::px(100.0))
                                .height(Length::px(100.0))
                                .with_auto_id()
                                .erased()
                        })
                        .into(),
                    ),
                    ..Default::default()
                }),
            ))
            .with_gap(Length::px(8.0))
            .with_auto_id()
            .erased(),
    )
    .with_auto_id()
    .erased()
}

fn main() {
    env_logger::init();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let runtime_handle = runtime.handle().clone();
    if let Err(err) = Builder::default()
        .spawn_fn(move |fut| {
            runtime_handle.spawn(fut);
        })
        .window(
            WindowBuilder::new(new_view)
                .with_title("Image")
                .base_color(WHITE),
        )
        .run()
    {
        eprintln!("{err}");
        process::exit(1)
    }
}
