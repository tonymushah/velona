use velona_core::{
    masonry_core::core::{NewWidget, Widget},
    window::{WindowHandle, use_window},
};

mod classes;
mod propstack;

pub use classes::{ScopedClasses, ScopedClassesState};
pub use propstack::ScopedPropstack;

pub(crate) fn use_window_local() -> WindowHandle {
    use_window().expect("Cannot find current window handle in the current context")
}

pub trait ApplyToNewWidget {
    fn apply_to_widget<W>(&self, new_widget: NewWidget<W>) -> NewWidget<W>
    where
        W: Widget + 'static;
}

pub trait ApplyScopedStyles {
    fn apply<A>(self, styles: &A) -> Self
    where
        A: ApplyToNewWidget;
}

impl<W> ApplyScopedStyles for NewWidget<W>
where
    W: Widget + 'static,
{
    fn apply<A>(self, styles: &A) -> Self
    where
        A: ApplyToNewWidget,
    {
        styles.apply_to_widget(self)
    }
}
