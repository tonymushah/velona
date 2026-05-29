use masonry::{
    core::{NewWidget, Widget},
    palette::css::WHITE,
    widgets::{Align, Checkbox, Flex},
};
use reactive_graph::{
    signal::signal,
    traits::{Get, Set},
};
use velona::{
    AnyNewWidget, Builder, NewWidgetExt, WindowBuilder, widgets::checkbox::NewCheckboxExt,
};

fn view() -> AnyNewWidget {
    let (checked, set_checked) = signal(false);
    Align::centered(
        Flex::column()
            .with_child(
                <NewWidget<Checkbox> as NewCheckboxExt>::new(
                    move || checked.get(),
                    move || {
                        if checked.get() {
                            "Checkbox checked"
                        } else {
                            "Checkbox not checked"
                        }
                    },
                )
                .register_handler(move |event| {
                    set_checked.set(event.0);
                }),
            )
            .with_auto_id(),
    )
    .with_auto_id()
    .erased()
}

fn main() {
    Builder::default()
        .window(
            WindowBuilder::new(view)
                .with_title("Checkbox")
                .base_color(WHITE),
        )
        .run()
        .unwrap();
}
