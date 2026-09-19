use velona::{
    AnyNewWidget, WindowBuilder,
    masonry::{
        core::{NewWidget, Widget},
        palette::css::WHITE,
        widgets::{Button, Flex, Label, Portal, Prose},
    },
    reactive::{
        computed::Memo,
        traits::{Get, Read},
    },
    widgets::{NewWidgetBaseExt, button::NewButtonPressEventsExt},
};
use velona_renderer_vello::create_wgpu_context;
use velona_router::{Route, Router, components::outlet, use_navigation_controller};

fn navigate_button(text: &str, goto: &'static str) -> NewWidget<Button> {
    let navigation = use_navigation_controller();
    let current_location = navigation.current();
    let is_on_goto = Memo::new(move |_| current_location.read().path() == goto);
    Button::with_text(text)
        .prepare()
        .on_primary(move || {
            if let Err(err) = navigation.push(goto) {
                eprintln!("{err}")
            }
        })
        .disabled_reactive(move || is_on_goto.get())
}

fn main_layout() -> AnyNewWidget {
    Portal::new(
        Flex::column()
            .with_fixed(
                Flex::row()
                    .with_fixed(navigate_button("Home", "/"))
                    .with_fixed(navigate_button("Posts", "/posts"))
                    .prepare(),
            )
            .with_spacer(10.0)
            .with_fixed(outlet())
            .prepare(),
    )
    .prepare()
    .erased()
}

fn index_view() -> AnyNewWidget {
    Prose::new("asdajdam ada sa daa dasmkook sa, dass jasd as mdampdas  sad sad ")
        .prepare()
        .erased()
}

fn nothing_to_show() -> AnyNewWidget {
    Label::new("Nothing to show").prepare().erased()
}

fn main_router() -> Router {
    Router::default().route(
        Route::layout(main_layout)
            .child(Route::index(index_view))
            .child(Route::wildcard("any", nothing_to_show)),
    )
}

#[cfg_attr(feature = "hotpath", hotpath::main)]
fn main() {
    env_logger::init();
    let g_context = create_wgpu_context(None, None);
    velona::Builder::new(move |_| {
        velona_renderer_vello::VelloWindowRenderer::new(g_context.clone())
    })
    .with_window(
        WindowBuilder::new(main_router().build())
            .with_title("Router example")
            .with_base_color(WHITE),
    )
    .run()
    .unwrap()
}
