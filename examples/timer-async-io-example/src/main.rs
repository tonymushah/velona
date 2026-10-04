use std::time::Duration;

use async_io::Timer;
use futures_util::StreamExt;
use velona::{
    NewWidgetExt, VelonaAppExt, WindowBuilder,
    masonry::{
        core::Widget,
        layout::AsUnit,
        palette::css::{BLACK, BLUE, DARK_BLUE, WHEAT, WHITE},
        properties::{
            Background, BorderColor, BorderWidth, CornerRadius, Dimensions, Padding, ThumbColor,
            ThumbRadius, TrackColor, TrackThickness,
            types::{CrossAxisAlignment, MainAxisAlignment},
        },
        widgets::{Flex, Slider},
    },
    reactive::{
        computed::Memo,
        effect::Effect,
        signal::signal,
        traits::{GetUntracked, Read, SignalOrFn, Update},
    },
    task::spawn_local_scoped_with_cancellation,
    widgets::{
        View,
        button::{IntoButton, NewButtonPressEventsExt},
        label::IntoNewLabel,
        portal::IntoPortal,
        sized_box::IntoSizedBox,
        slider::NewSliderExt,
    },
};
use velona_renderer_vello_hybrid::create_wgpu_context;

fn view() -> impl View {
    let (period, set_period) = signal(Duration::from_millis(500));

    let period_millis = Memo::new(move |_| {
        let a: f64 = period.read().as_millis() as _;
        a
    });

    let (time_elapsed, set_time_elapsed) = signal(Duration::default());
    let (enabled, set_enabled) = signal(true);
    use_interval(
        period,
        move |period| {
            set_time_elapsed.try_maybe_update(|write| {
                if let Some(new_duration) = write.checked_add(period) {
                    *write = new_duration;
                    (true, ())
                } else {
                    (false, ())
                }
            });
        },
        enabled,
    );
    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .main_axis_alignment(MainAxisAlignment::Center)
        .with_fixed(
            (move || {
                let Some(time_elapsed_read) = time_elapsed.try_read() else {
                    return String::from("Nothing");
                };
                format!(
                    "Time elapsed: {}.{:0>3}s",
                    time_elapsed_read.as_secs(),
                    time_elapsed_read
                        .as_millis()
                        .checked_rem(1000)
                        .unwrap_or_default()
                )
            })
            .into_new_label(),
        )
        .with_fixed(
            (move || format!("Refresh time: {}ms", period.read().as_millis())).into_new_label(),
        )
        .with_fixed_spacer(10.0.px())
        .with_fixed(
            Slider::new(10.0, 2000.0, period_millis.get_untracked())
                .prepare()
                .value(period_millis)
                .on_action(move |change| {
                    set_period.update(move |period| {
                        let new_period = Duration::from_millis(change.value as _);

                        *period = new_period;
                    });
                })
                .with_props((
                    TrackThickness(5.px()),
                    TrackColor {
                        active: BLUE,
                        inactive: DARK_BLUE,
                    },
                    ThumbColor(WHITE),
                    ThumbRadius(10.px()),
                ))
                .into_sized_box()
                .prepare()
                .with_props(Dimensions::fixed(200.px(), 50.px())),
        )
        .with_fixed_spacer(5.0.px())
        .with_fixed(
            (move || {
                if enabled() { "Pause" } else { "Unpause" }
            })
            .into_new_label()
            .into_button()
            .prepare()
            .on_primary(move || {
                set_enabled.update(|enabled| *enabled = !*enabled);
            })
            .with_props((
                Background::Color(WHEAT),
                Padding::from_vh(4.0.px(), 12.0.px()),
                BorderWidth::all(2.0.px()),
                BorderColor::new(BLACK),
                CornerRadius::all(8.0.px()),
            )),
        )
        .prepare()
        .into_portal()
        .content_must_fill(true)
        .prepare()
}

fn use_interval<I, F, E>(interval_period: I, run_fn: F, enabled: E)
where
    I: SignalOrFn<Output = Duration> + 'static,
    F: Fn(Duration) + Clone + Send + Sync + 'static,
    E: SignalOrFn<Output = bool> + 'static,
{
    Effect::new(move || {
        let run_fn = run_fn.clone();
        if enabled.run() {
            let interval_period = interval_period.run();

            spawn_local_scoped_with_cancellation(async move {
                let mut timer = Timer::interval(interval_period);
                let mut start = std::time::Instant::now();
                while let Some(_end) = timer.next().await {
                    run_fn(_end - start);
                    start = std::time::Instant::now();
                }
            });
        }
    });
}

#[cfg_attr(feature = "hotpath", hotpath::main)]
fn main() {
    env_logger::init();

    let g_context = create_wgpu_context(None, None);
    velona::Builder::new(move |_| {
        velona_renderer_vello_hybrid::VelloHybridWindowRenderer::new(g_context.clone())
    })
    .with_window(
        WindowBuilder::new(view)
            .with_title("Timer")
            .with_base_color(WHITE),
    )
    .build()
    .run()
    .unwrap()
}
