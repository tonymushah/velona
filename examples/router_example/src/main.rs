use std::sync::Arc;

use velona::{
    AnyNewWidget, WindowBuilder,
    masonry::{
        core::{NewWidget, Widget},
        layout::{AsUnit, Length},
        palette::css::{BLACK, MISTY_ROSE, WHEAT, WHITE, WHITE_SMOKE},
        properties::{Background, BorderColor, BorderWidth, CornerRadius, Padding},
        widgets::{Button, Flex, Label, Portal, Prose},
    },
    reactive::{
        computed::Memo,
        owner::{expect_context, provide_context},
        traits::{Get, Read},
    },
    scoped_styling::{ApplyScopedStyles, ApplyToNewWidget, ScopedClasses, ScopedClassesState},
    widgets::{NewWidgetBaseExt, button::NewButtonPressEventsExt},
};
use velona_renderer_vello::create_wgpu_context;
use velona_router::{Route, Router, components::outlet, use_navigation_controller};

// This is here to prevent props drilling
#[derive(Debug, Clone)]
struct NavigateButtonStyles(Arc<ScopedClasses<1>>);

impl ApplyToNewWidget for NavigateButtonStyles {
    fn apply_to_widget<W>(&self, new_widget: NewWidget<W>) -> NewWidget<W>
    where
        W: Widget + ?Sized,
    {
        new_widget.apply(&*self.0)
    }
}

trait ApplyNavigateButtonStyles {
    fn apply_navigate_button_styles(self) -> Self;
}

impl<W> ApplyNavigateButtonStyles for NewWidget<W>
where
    W: Widget + 'static,
{
    fn apply_navigate_button_styles(self) -> Self {
        // BUG The app will not show if we use `with_context`
        // It is because of effects hang in on forever in `with_context`
        //
        // with_context::<NavigateButtonStyles, Self>(|s| {
        //     println!("with context");
        //     let res = self.apply(&s.0);
        //     println!("Applyed style");
        //     res
        // })
        // .expect("The `NavigateButtonStyles` should be available in the current context")
        let styles = expect_context::<NavigateButtonStyles>();
        self.apply(&styles)
    }
}

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
        .apply_navigate_button_styles()
}

fn main_layout() -> AnyNewWidget {
    let button_styles = ScopedClasses::new(["navigation-buttons"])
        .prop(ScopedClassesState::HOVERED.disabled(false), |_| {
            Background::Color(WHEAT)
        })
        .prop(ScopedClassesState::default(), |_| {
            BorderWidth::all(Length::px(3.0))
        })
        .prop(ScopedClassesState::default(), |_| BorderColor::new(BLACK))
        .prop(ScopedClassesState::default(), |_| {
            CornerRadius::all(Length::const_px(8.0))
        })
        .prop(ScopedClassesState::default(), |_| {
            Padding::from_vh(Length::const_px(4.0), Length::const_px(12.0))
        })
        .prop(ScopedClassesState::DISABLED, |_| {
            Background::Color(MISTY_ROSE)
        })
        .prop(ScopedClassesState::ACTIVE, |_| {
            Background::Color(WHITE_SMOKE)
        });

    provide_context(NavigateButtonStyles(Arc::new(button_styles)));

    Portal::new(
        Flex::column()
            .with_fixed(
                Flex::row()
                    .with_fixed(navigate_button("Home", "/"))
                    .with_fixed_spacer(10.0.px())
                    .with_fixed(navigate_button("Posts", "/posts"))
                    .prepare(),
            )
            .with_fixed_spacer(10.0.px())
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
