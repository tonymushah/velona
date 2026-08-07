// use log::trace;
use masonry::{
    core::Widget,
    layout::{AsUnit, Length},
    palette::css::{BEIGE, BLACK, WHITE},
    properties::{Background, BorderColor, BorderWidth, Padding},
    widgets::{Button, Flex, FlexParams, Label, Prose, TextInput},
};
use velona::{
    AnyNewWidget,
    WindowBuilder,
    collection::NewCollectionWidgetExt,
    // reactive::{effect::Effect, traits::Read},
    widgets::{button::NewButtonPressEventsExt, text_input::NewTextInputActionExt},
};
use velona_core::reactive::{signal::signal, traits::Update};
use velona_renderer_vello::create_wgpu_context;

fn view() -> AnyNewWidget {
    let (todos, set_todos) = signal(Vec::<String>::new());

    // Effect::new(move || {
    //     let todo_ref = todos.read();
    //     trace!("todo len: {}", todo_ref.len());
    //     trace!("todo capacity: {}", todo_ref.capacity());
    //     trace!("todo real size: {}", size_of_val(&*todo_ref))
    // });

    Flex::column()
        .main_axis_alignment(masonry::properties::types::MainAxisAlignment::Center)
        .cross_axis_alignment(masonry::properties::types::CrossAxisAlignment::Center)
        .with_fixed(Prose::new("Todos").prepare())
        .with_fixed(Flex::column().prepare().collect_reactive_iter(move || {
            todos().into_iter().enumerate().map(move |(index, item)| {
                (
                    Flex::row()
                        .cross_axis_alignment(
                            masonry::properties::types::CrossAxisAlignment::Center,
                        )
                        .main_axis_alignment(masonry::properties::types::MainAxisAlignment::Center)
                        .with_fixed(Label::new(item).prepare())
                        .with_fixed_spacer(Length::px(20.0))
                        .with_fixed(
                            Button::with_text("Remove")
                                .prepare()
                                .on_primary(move || {
                                    set_todos.update(|todos| {
                                        todos.swap_remove(index);
                                    });
                                })
                                .with_props(BorderColor::new(BLACK))
                                .with_props(Background::Color(BEIGE))
                                .with_props(BorderWidth::all(Length::px(1.0)))
                                .with_props(Padding::from_vh(4.0.px(), 12.0.px())),
                        )
                        .prepare()
                        .erased(),
                    FlexParams::default(),
                )
            })
        }))
        .with_fixed(
            TextInput::new("")
                .with_placeholder("Put something...")
                .prepare()
                .with_props(BorderColor::new(BLACK))
                .with_props(BorderWidth::all(Length::px(1.0)))
                .with_props(Padding::from_vh(Length::px(4.0), Length::px(12.0)))
                .on_text_action(move |a| match a {
                    masonry::widgets::TextAction::Changed(e) => log::trace!("Changed input {e}"),
                    masonry::widgets::TextAction::Entered(e) => {
                        set_todos.update(|todos| todos.push(e.clone()))
                    }
                    _ => {}
                }),
        )
        .prepare()
        .with_props(Padding::all(Length::px(12.0)))
        .erased()
}

#[cfg_attr(feature = "hotpath", hotpath::main)]
fn main() {
    env_logger::init();
    let g_context = create_wgpu_context(None, None);
    velona::Builder::new(move |_| {
        velona_renderer_vello::VelloWindowRenderer::new(g_context.clone())
    })
    .window(
        WindowBuilder::new(view)
            .with_title("Todos")
            .base_color(WHITE),
    )
    .run()
    .unwrap()
}
