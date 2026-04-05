use masonry::{
    core::{NewWidget, Properties, Widget},
    palette::css,
    properties::ContentColor,
    widgets::Label,
};
use velona::window::WindowBuilder;

fn view() -> NewWidget<dyn Widget + 'static> {
    Label::new("Text")
        .with_props(Properties::one(ContentColor::new(css::WHITE)))
        .erased()
}

fn main() {
    env_logger::init();
    velona::app::Builder::default()
        .window(WindowBuilder::new(view).with_title("aaaaaa"))
        .run()
        .unwrap()
}
