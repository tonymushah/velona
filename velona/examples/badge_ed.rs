use std::{sync, time::Duration};

use derive_more::{Display, FromStr};
use enum_all_variants::AllVariants;
use image::open;
use masonry::core::DefaultProperties;
use masonry::imaging::peniko::{Blob, ImageBrush, ImageData, ImageSampler, color::AlphaColor};
use masonry::layers::SelectorMenu;
use masonry::palette::css::BISQUE;
use masonry::properties::TrackColor;
use masonry::widgets::SelectorItem;
use masonry::{
    core::Widget,
    layout::Length,
    palette::css::{BLACK, VIOLET, WHEAT, WHITE, WHITE_SMOKE},
    parley::{FontWeight, StyleProperty},
    properties::{Background, BorderColor, BorderWidth, CornerRadius, Padding},
    theme::DEFAULT_SPACER_LEN,
    widgets::{Badged, Button, Flex, Label, Selector, SizedBox, Spinner},
};
use tokio::runtime;
use velona::{
    AnyNewWidget, Builder, NewWidgetExt, WindowBuilder,
    components::{LazyImageOptions, badge_count, label, lazy_image},
    utils::memo::unsync_memo,
    widgets::{
        self, badged::NewBadgedTrait, button::NewButtonPressEventsExt, sized_box::NewSizedBoxExt,
    },
};
use velona_core::reactive::{
    callback::{Callable, UnsyncCallback},
    computed::Memo,
    effect::Effect,
    signal::{WriteSignal, arc_signal, signal},
    spawn,
    traits::{Get, Read, Set, Update},
};
use velona_renderer_vello::{VelloWindowRenderer, create_wgpu_context};
use velona_scoped_styling::{ApplyScopedStyles, ScopedClasses};

struct TowaProps {
    show: UnsyncCallback<(), bool>,
}

enum ImageState {
    Loading,
    Ready(ImageBrush),
    Error(anyhow::Error),
}

impl ImageState {
    fn ready(&self) -> Option<ImageBrush> {
        if let Self::Ready(data) = self {
            Some(data.clone())
        } else {
            None
        }
    }
}

fn towa(TowaProps { show }: TowaProps) -> AnyNewWidget {
    let show = unsync_memo(move || show.run(()));
    let (base_image, set_base_image) = arc_signal(ImageState::Loading);
    let image = unsync_memo({
        let base_image = base_image.clone();
        move || base_image.read().ready().filter(|_| show.get())
    });
    // Load the towa image into another thread.
    velona::task::spawn(async move {
        match open("assets/tokoyami-towa.webp") {
            Ok(data) => {
                let data = data.into_rgba8();
                let width = data.width();
                let height = data.height();
                let mut data_buf = data.into_vec();
                data_buf.shrink_to_fit();
                set_base_image.set(ImageState::Ready(ImageBrush {
                    image: ImageData {
                        data: Blob::new(sync::Arc::new(data_buf)),
                        format: masonry::peniko::ImageFormat::Rgba8,
                        alpha_type: masonry::peniko::ImageAlphaType::Alpha,
                        width,
                        height,
                    },
                    sampler: ImageSampler::new().with_quality(masonry::peniko::ImageQuality::High),
                }));
            }
            Err(err) => {
                set_base_image.set(ImageState::Error(anyhow::Error::new(err)));
            }
        }
    });
    lazy_image(
        move || image.get(),
        LazyImageOptions {
            fallback: Some({
                let base_image = base_image.clone();

                (move || {
                    if !show.get() {
                        return SizedBox::empty().prepare().erased();
                    }
                    match *base_image.read() {
                        ImageState::Loading => SizedBox::new(
                            Spinner::default().prepare().static_propeperty(TrackColor {
                                inactive: VIOLET,
                                active: BLACK,
                            }),
                        )
                        .height(Length::px(50.0))
                        .width(Length::px(50.0))
                        .prepare()
                        .erased(),
                        ImageState::Error(ref error) => {
                            Label::new(format!("Cannot show her webp image ;( [{error}]"))
                                .with_style(StyleProperty::FontWeight(FontWeight::new(700.0)))
                                .prepare()
                                .erased()
                        }
                        _ => SizedBox::empty().prepare().erased(),
                    }
                })
                .into()
            }),
            ..Default::default()
        }
        .into(),
    )
    .width(|| Length::px(400.0))
    .height(|| Length::px(400.0))
    .erased()
}

#[derive(Debug, Clone, Copy, Display, FromStr, AllVariants)]
enum BadgePlacement {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl From<BadgePlacement> for widgets::masonry_widgets::BadgePlacement {
    fn from(value: BadgePlacement) -> Self {
        match value {
            BadgePlacement::TopLeft => widgets::masonry_widgets::BadgePlacement::TopLeft,
            BadgePlacement::TopRight => widgets::masonry_widgets::BadgePlacement::TopRight,
            BadgePlacement::BottomLeft => widgets::masonry_widgets::BadgePlacement::BottomLeft,
            BadgePlacement::BottomRight => widgets::masonry_widgets::BadgePlacement::BottomRight,
        }
    }
}

fn badge_placement(set_placement: WriteSignal<BadgePlacement>) -> AnyNewWidget {
    Flex::row()
        .with_fixed(Label::new("Placement: ").prepare())
        .with_fixed_spacer(DEFAULT_SPACER_LEN)
        .with_fixed(
            SizedBox::new(
                Selector::new(
                    BadgePlacement::all_variants()
                        .iter()
                        .map(|d| d.to_string())
                        .collect(),
                )
                .prepare()
                .on_action(move |changes| {
                    if let Ok(align) = changes.selected_content.parse::<BadgePlacement>() {
                        set_placement.set(align);
                    }
                })
                .static_propeperty(BorderColor::new(BLACK))
                .static_propeperty(BorderWidth::all(Length::px(3.0)))
                .static_propeperty(CornerRadius::all(Length::px(8.0)))
                .static_propeperty(Padding::from_vh(Length::px(4.0), Length::px(8.0)))
                .static_propeperty(Background::Color(WHITE_SMOKE)),
            )
            .prepare(),
        )
        .main_axis_alignment(masonry::properties::types::MainAxisAlignment::Center)
        .prepare()
        .erased()
}

fn main_view() -> AnyNewWidget {
    let (count, set_count) = signal(0u32);
    // We use a memo for this to avoid unnecessary rerenders
    let is_zero = Memo::new(move |_| count.get() == 0);

    let (show_towa, should_show_towa) = arc_signal(false);
    // We "remove" the towa image after 2 second
    {
        let show_towa = show_towa.clone();
        let should_show_towa = should_show_towa.clone();
        Effect::new(move || {
            let should_show_towa = should_show_towa.clone();
            if *show_towa.read() {
                spawn(async move {
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    should_show_towa.set(false);
                });
            }
        });
    }

    let (placement, set_placement) = signal(BadgePlacement::TopRight);
    // let (badge_color, set_bagde_color) = signal()

    let badge_style = ScopedClasses::new(["badge"])
        .prop(Default::default(), |_| Background::Color(BISQUE))
        .prop(Default::default(), |_| {
            Padding::from_vh(Length::px(2.5), Length::px(5.0))
        })
        .prop(Default::default(), |_| BorderColor::new(BLACK))
        .prop(Default::default(), |_| BorderWidth::all(Length::px(1.0)))
        .prop(Default::default(), |_| CornerRadius::all(Length::px(3.0)));
    Flex::column()
        .main_axis_alignment(masonry::properties::types::MainAxisAlignment::Center)
        // The actual badge
        .with_fixed(
            Badged::new_optional(
                Button::new(label(move || {
                    if is_zero.get() {
                        "Click!"
                    } else {
                        "Increment!"
                    }
                }))
                .prepare()
                .on_primary(move || {
                    set_count.update(|count| *count += 1);
                })
                .static_propeperty(Background::Color(WHEAT))
                .static_propeperty(BorderColor::new(BLACK))
                .static_propeperty(BorderWidth::all(Length::px(3.0)))
                .static_propeperty(Padding::from_vh(Length::px(4.0), Length::px(8.0)))
                .static_propeperty(CornerRadius::all(Length::px(3.0))),
                None,
            )
            .prepare()
            .badge(move || {
                if !is_zero.get() {
                    Some(
                        badge_count(move || count.get())
                            .apply(&badge_style)
                            .erased(),
                    )
                } else {
                    None
                }
            })
            .badge_placement(move || placement.get().into()),
        )
        .with_fixed_spacer(Length::px(10.0))
        // the badge placement
        .with_fixed(badge_placement(set_placement))
        .with_fixed_spacer(Length::px(10.0))
        // reset button
        .with_fixed(
            // place a Neuron activiation meme here
            Button::with_text("Tokoyami Towa Reset :)")
                .prepare()
                .on_primary(move || {
                    set_count.set(0);
                    should_show_towa.set(true);
                })
                // Good luck on figuring what it is :)
                .static_propeperty(Background::Color(AlphaColor::from_rgb8(
                    0xff_u8, 0x78_u8, 0xff_u8,
                )))
                .static_propeperty(BorderColor::new(BLACK))
                .static_propeperty(BorderWidth::all(Length::px(3.0)))
                .static_propeperty(Padding::from_vh(Length::px(4.0), Length::px(8.0))),
        )
        // Towa!!!
        .with_fixed(towa(TowaProps {
            show: (move || show_towa.get()).into(),
        }))
        .prepare()
        .erased()
}

#[cfg_attr(feature = "hotpath-run", hotpath::main)]
fn main() {
    let runtime = runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let g_context = create_wgpu_context(None, None);
    Builder::new(move |_| VelloWindowRenderer::new(g_context.clone()))
        .with_spawn_fn({
            let handle = runtime.handle().clone();
            move |fut| {
                handle.spawn(fut);
            }
        })
        .with_window(WindowBuilder::new(main_view).with_base_color(WHITE))
        .with_default_properties(default_properties())
        .run()
        .unwrap();
}

fn default_properties() -> DefaultProperties {
    let mut p = DefaultProperties::default();
    p.insert::<SelectorMenu, _>(Background::Color(WHITE_SMOKE));
    p.insert::<SelectorMenu, _>(BorderWidth::all(Length::px(1.0)));
    p.insert::<SelectorMenu, _>(BorderColor::new(BLACK));
    p.insert::<SelectorItem, _>(Padding::from_vh(Length::px(2.5), Length::px(5.0)));
    p
}
