use std::{process, sync};

use masonry::{
    core::Widget,
    layout::Length,
    palette::css::WHITE,
    peniko::{Blob, ImageBrush, ImageData, ImageSampler},
    widgets::{Flex, Label, SizedBox, Spinner},
};
use reactive_graph::{
    computed::Memo,
    signal::signal,
    traits::{Get, Read, Set},
};
use velona::{
    AnyNewWidget, Builder, WindowBuilder,
    components::{LazyImageOptions, lazy_image},
};

enum ImageState {
    Loading,
    Ready(ImageBrush),
    Error(anyhow::Error),
}

fn new_view() -> AnyNewWidget {
    let (image_data, set_image_data) = signal(ImageState::Loading);
    reactive_graph::spawn(async move {
        // TODO Fix buffer overflow panic
        match image::open("assets/image1.png") {
            Ok(data) => {
                let data = data.into_rgba8();
                let width = data.width();
                let height = data.height();
                let mut data_buf = data.into_vec();
                data_buf.shrink_to_fit();
                set_image_data.set(ImageState::Ready(ImageBrush {
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
                set_image_data.set(ImageState::Error(anyhow::Error::from(err)));
            }
        }
    });
    let image_ready = Memo::new(move |_| {
        if let ImageState::Ready(ref img) = *image_data.read() {
            Some(img.clone())
        } else {
            None
        }
    });
    Flex::column()
        .with_fixed(Label::new("SOme image").prepare().erased())
        .with_fixed_spacer(Length::px(8.0))
        .with_fixed(lazy_image(
            move || image_ready.get(),
            Some(LazyImageOptions {
                fallback: Some(
                    (move || match *image_data.read() {
                        ImageState::Loading => SizedBox::new(Spinner::new().prepare())
                            .width(Length::px(100.0))
                            .height(Length::px(100.0))
                            .prepare()
                            .erased(),
                        ImageState::Ready(ref _a) => SizedBox::empty().prepare().erased(),
                        ImageState::Error(ref error) => {
                            Label::new(format!("Cannot load image: {error}"))
                                .prepare()
                                .erased()
                        }
                    })
                    .into(),
                ),
                // object_fit: Some((|| ObjectFit::Stretch).into())
                ..Default::default()
            }),
        ))
        .main_axis_alignment(masonry::properties::types::MainAxisAlignment::Center)
        .prepare()
        .erased()
}

#[cfg_attr(feature = "hotpath", hotpath::main)]
fn main() {
    env_logger::init();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let runtime_handle = runtime.handle().clone();
    if let Err(err) = Builder::new(|_| velona_renderer_vello::VelloWindowRenderer::new())
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
