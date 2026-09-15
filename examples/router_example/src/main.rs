use velona::{
    AnyNewWidget, Builder, WindowBuilder,
    masonry::{
        core::{NewWidget, Widget},
        layout::Length,
        palette::css::{BLACK, WHEAT, WHITE_SMOKE},
        peniko::color::AlphaColor,
        properties::{Background, BorderColor, BorderWidth},
        theme::DEFAULT_SPACER_LEN,
        widgets::{Button, Flex, Label, Portal},
    },
    scoped_styling::{ApplyScopedStyles, ScopedClasses, ScopedClassesState},
    widgets::{NewWidgetBaseExt, button::NewButtonPressEventsExt},
};
use velona_renderer_vello::create_wgpu_context;
use velona_router::{RouterBuilder, except_router_ctx};

fn nav_buttons(
    styles: &ScopedClasses<1>,
    title: &'static str,
    goto: &'static str,
) -> NewWidget<Button> {
    let router = except_router_ctx();
    Button::with_text(title)
        .prepare()
        .apply(styles)
        .disabled_reactive({
            let router = router.clone();
            move || router.current_route() == goto
        })
        .on_primary({
            let router = router.clone();
            move || {
                router.goto(goto);
            }
        })
}

fn layout(child: NewWidget<impl Widget>) -> AnyNewWidget {
    let styles = button_styles();
    Portal::new(
        Flex::column()
            .with_fixed(
                Flex::row()
                    .with_fixed(nav_buttons(&styles, "Home", "/"))
                    .with_fixed_spacer(DEFAULT_SPACER_LEN)
                    .with_fixed(nav_buttons(&styles, "Posts", "/posts"))
                    .with_fixed_spacer(DEFAULT_SPACER_LEN)
                    .with_fixed(nav_buttons(&styles, "About", "/about"))
                    .prepare(),
            )
            .with_fixed(child)
            .prepare(),
    )
    .prepare()
    .erased()
}

fn button_styles() -> ScopedClasses<1> {
    ScopedClasses::new(["nav-buttons"])
        .prop(ScopedClassesState::default(), |_| Background::Color(WHEAT))
        .prop(ScopedClassesState::default(), |_| {
            BorderWidth::all(Length::px(10.0))
        })
        .prop(ScopedClassesState::default().disabled(true), |_| {
            Background::Color(AlphaColor::from_rgb8(200, 200, 200))
        })
        .prop(ScopedClassesState::default(), |_| BorderColor::new(BLACK))
        .prop(ScopedClassesState::default().hovered(true), |_| {
            Background::Color(WHITE_SMOKE)
        })
}

fn main_view() -> AnyNewWidget {
    layout(
        Flex::column()
            .with_fixed(Label::new("Hello There").prepare())
            .prepare(),
    )
}

fn main_routes() -> impl FnOnce() -> AnyNewWidget + Send + 'static {
    RouterBuilder::default().route("/", main_view).build()
}

fn main() {
    let wgpu_ctx = create_wgpu_context(None, None);
    Builder::new(move |_| velona_renderer_vello::VelloWindowRenderer::new(wgpu_ctx.clone()))
        .with_window(WindowBuilder::new(main_routes()).with_title("Router example"))
        .run()
        .unwrap();
}
