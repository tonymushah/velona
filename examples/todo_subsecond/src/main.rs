use std::sync::Arc;
mod views;

use log::trace;
use velona::NewWidgetExt;
use velona::components::label;
use velona::masonry::properties::CornerRadius;
use velona::masonry::widgets::Portal;
use velona::reactive::traits::Read;
// use log::trace;
use velona::reactive::{signal::signal, traits::Update};
use velona::subsecond::{hot_value_with_memo, hot_value_with_memo_raw};
use velona::utils::{hot_view, local_effect};
use velona::{
    AnyNewWidget, WindowBuilder, collection::NewCollectionWidgetExt,
    widgets::button::NewButtonPressEventsExt,
};
use velona::{
    masonry::{
        self,
        core::Widget,
        layout::{AsUnit, Length},
        palette::css::{BEIGE, BLACK, WHITE},
        properties::{Background, BorderColor, BorderWidth, Padding},
        widgets::{Button, Flex, FlexParams, Label, Prose},
    },
    widgets::text_area::NewTextAreaExt,
};
use velona_renderer_vello::create_wgpu_context;

// trait FnTransMut<T> {
//     fn into_opt_arg_fn(self) -> impl Fn(Option<T>) -> T + 'static
//     where
//         T: 'static;
// }

// impl<T, F> FnTransMut<T> for F
// where
//     T: 'static,
//     F: Fn() -> T + 'static,
// {
//     fn into_opt_arg_fn(self) -> impl Fn(Option<T>) -> T + 'static {
//         move |_| self()
//     }
// }

fn view() -> AnyNewWidget {
    let (todos, set_todos) = signal(Vec::<Arc<str>>::new());

    local_effect(move || {
        let todo_ref = todos.read();
        trace!("todo len: {}", todo_ref.len());
        trace!("todo capacity: {}", todo_ref.capacity());
        trace!("todo rea size: {}", size_of_val(&*todo_ref))
    });

    let text = hot_value_with_memo_raw(|| String::from("Remove?"));

    Portal::new(
        Flex::column()
            .main_axis_alignment(masonry::properties::types::MainAxisAlignment::Start)
            .cross_axis_alignment(masonry::properties::types::CrossAxisAlignment::Center)
            .with_fixed(
                Prose::new("Todos")
                    .prepare()
                    .text(hot_value_with_memo(|| String::from("Some todos"))),
            )
            .with_fixed(Flex::column().prepare().collect_reactive_iter(move || {
                todos()
                    .into_iter()
                    .enumerate()
                    .map(move |(index, item)| {
                        (
                            hot_view(move || {
                                Flex::row()
                                    .cross_axis_alignment(
                                        masonry::properties::types::CrossAxisAlignment::Center,
                                    )
                                    .main_axis_alignment(
                                        masonry::properties::types::MainAxisAlignment::Start,
                                    )
                                    .with_fixed(Label::new(item.clone()).prepare())
                                    .with_fixed_spacer(Length::px(20.0))
                                    .with_fixed(
                                        Button::new(label(move || (*text)()))
                                            .prepare()
                                            .on_primary(move || {
                                                set_todos.update(|todos| {
                                                    todos.swap_remove(index);
                                                });
                                            })
                                            .with_props(BorderColor::new(BLACK))
                                            .with_props(Background::Color(BEIGE))
                                            .with_props(BorderWidth::all(Length::px(1.0)))
                                            .with_props(Padding::from_vh(4.0.px(), 12.0.px()))
                                            .with_props(CornerRadius::all(Length::px(2.0))),
                                    )
                                    .prepare()
                                    .static_propeperty(Padding::from_vh(
                                        Length::px(2.0),
                                        Length::default(),
                                    ))
                                    .erased()
                            }),
                            FlexParams::default(),
                        )
                    })
                    .collect::<Vec<_>>()
            }))
            .with_fixed(views::text_input(set_todos))
            .prepare()
            .with_props(Padding::all(Length::px(12.0)))
            .erased(),
    )
    .content_must_fill(true)
    .prepare()
    .erased()
}

#[cfg_attr(feature = "hotpath", hotpath::main)]
fn main() {
    env_logger::init();
    let g_context = create_wgpu_context(None, None);
    velona::Builder::new(move |_| {
        velona_renderer_vello::VelloWindowRenderer::new(g_context.clone())
    })
    .with_window(
        WindowBuilder::new(view)
            .with_title("Todos")
            .with_base_color(WHITE),
    )
    .run()
    .unwrap()
}
