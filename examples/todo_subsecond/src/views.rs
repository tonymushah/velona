use std::sync::Arc;

use velona::{
    masonry::{
        self,
        core::Widget,
        layout::Length,
        palette::css::BLACK,
        properties::{BorderColor, BorderWidth, Padding},
        widgets::TextInput,
    },
    reactive::traits::Update,
    widgets::text_input::NewTextInputActionExt,
};

pub fn text_input(
    set_todos: velona::reactive::signal::WriteSignal<Vec<Arc<str>>>,
) -> masonry::core::NewWidget<dyn Widget + 'static> {
    TextInput::new("")
        .with_placeholder("Put something...")
        .prepare()
        .with_props(BorderColor::new(BLACK))
        .with_props(BorderWidth::all(Length::px(1.0)))
        .with_props(Padding::from_vh(Length::px(4.0), Length::px(12.0)))
        .on_text_action(move |a| match a {
            masonry::widgets::TextAction::Changed(e) => log::trace!("Changed input {e}"),
            masonry::widgets::TextAction::Entered(todo) => {
                set_todos.update(|todos| {
                    if let Some(already) = todos.iter().find(|inside| &***inside == todo).cloned() {
                        todos.push(already);
                    } else {
                        todos.push(todo.as_str().into());
                    }
                });
            }
            _ => {}
        })
        .erased()
}
