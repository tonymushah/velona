use masonry::{
    core::Widget,
    widgets::{Button, Flex, FlexParams, Label, Prose, TextInput},
};
use reactive_graph::{signal::signal, traits::Update};
use velona::{
    AnyNewWidget, WindowBuilder,
    collection::NewCollectionWidgetExt,
    widgets::{button::NewButtonPressEventsExt, text_input::NewTextInputActionExt},
};

fn view() -> AnyNewWidget {
    let (todos, set_todos) = signal(Vec::<String>::new());

    Flex::column()
        .main_axis_alignment(masonry::properties::types::MainAxisAlignment::Center)
        .cross_axis_alignment(masonry::properties::types::CrossAxisAlignment::Start)
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
                        .with_fixed(Button::with_text("Remove").prepare().on_primary(move || {
                            set_todos.update(|todos| {
                                todos.swap_remove(index);
                            });
                        }))
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
                .on_text_action(move |a| match a {
                    masonry::widgets::TextAction::Changed(e) => log::trace!("Changed input {e}"),
                    masonry::widgets::TextAction::Entered(e) => {
                        set_todos.update(|todos| todos.push(e.clone()))
                    }
                }),
        )
        .prepare()
        .erased()
}

fn main() {
    velona::Builder::new(|_| velona_renderer_vello::VelloWindowRenderer::new())
        .window(WindowBuilder::new(view).with_title("Todos"))
        .run()
        .unwrap()
}
