use velona_core::{
    masonry_core::core::{NewWidget, PropertySet, PropertyStack, Selector, Widget},
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
        W: Widget + ?Sized;
}

pub trait ApplyScopedStyles {
    fn apply<A>(self, styles: &A) -> Self
    where
        A: ApplyToNewWidget;
}

impl<W> ApplyScopedStyles for NewWidget<W>
where
    W: Widget + ?Sized,
{
    fn apply<A>(self, styles: &A) -> Self
    where
        A: ApplyToNewWidget,
    {
        styles.apply_to_widget(self)
    }
}

pub(crate) trait PropertyStackUtils {
    fn has_selector(&self, selector: &Selector) -> bool;
    fn get_last_selector_property_set_mut(
        &mut self,
        selector: &Selector,
    ) -> Option<&mut PropertySet>;
}

impl PropertyStackUtils for PropertyStack {
    fn has_selector(&self, selector: &Selector) -> bool {
        self.get_layers().iter().any(|(sel, _)| sel == selector)
    }
    fn get_last_selector_property_set_mut(
        &mut self,
        selector: &Selector,
    ) -> Option<&mut PropertySet> {
        let index = self
            .get_layers()
            .iter()
            .enumerate()
            .rev()
            .find_map(|(index, (selector_in, _))| (selector_in == selector).then_some(index))?;
        self.get_layer_mut(index)
    }
}
