use velona_core::masonry_core::core::{Property, Selector};

use crate::{ApplyScopedStyles, ApplyToNewWidget, ApplyToWidgetMut, ScopedPropstack};

#[derive(Debug)]
pub struct ScopedClasses<const N: usize> {
    classes: [&'static str; N],
    prop_stack: ScopedPropstack,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopedClassesState {
    pub is_hovered: Option<bool>,
    pub is_active: Option<bool>,
    pub is_disabled: Option<bool>,
    pub has_focus_target: Option<bool>,
}

impl From<ScopedClassesState> for Selector {
    fn from(value: ScopedClassesState) -> Self {
        let selector = Self::default();
        value.apply_to_selector(selector)
    }
}

impl ScopedClassesState {
    pub fn hovered(mut self, is_hovered: bool) -> Self {
        self.is_hovered = Some(is_hovered);
        self
    }
    pub fn active(mut self, is_active: bool) -> Self {
        self.is_active = Some(is_active);
        self
    }
    pub fn disabled(mut self, is_disabled: bool) -> Self {
        self.is_disabled = Some(is_disabled);
        self
    }
    pub fn focus(mut self, has_focus_target: bool) -> Self {
        self.has_focus_target = Some(has_focus_target);
        self
    }
    pub fn apply_to_selector(self, mut selector: Selector) -> Selector {
        if let Some(hovered) = self.is_hovered {
            selector = selector.with_hovered(hovered);
        }
        if let Some(active) = self.is_active {
            selector = selector.with_active(active);
        }
        if let Some(disabled) = self.is_disabled {
            selector = selector.with_disabled(disabled);
        }
        if let Some(focus) = self.has_focus_target {
            selector = selector.with_focused(focus);
        }
        selector
    }
    pub fn into_selector_with_classes(self, classes: &[&str]) -> Selector {
        self.apply_to_selector(Selector::classes(classes))
    }
}

impl<const N: usize> ScopedClasses<N> {
    pub fn new(classes: [&'static str; N]) -> Self {
        Self {
            classes,
            prop_stack: ScopedPropstack::default(),
        }
    }
    pub fn prop_opt<P, Pfn>(mut self, state: ScopedClassesState, prop: Pfn) -> Self
    where
        P: Property,
        Pfn: Fn(Option<P>) -> Option<P> + 'static,
    {
        self.prop_stack = self
            .prop_stack
            .prop_opt(state.into_selector_with_classes(&self.classes), prop);
        self
    }
    pub fn prop<P, Pfn>(self, state: ScopedClassesState, prop: Pfn) -> Self
    where
        P: Property,
        Pfn: Fn(Option<P>) -> P + 'static,
    {
        self.prop_opt(state, move |old| Some(prop(old)))
    }
    pub fn propstack_ref(&self) -> &ScopedPropstack {
        &self.prop_stack
    }
}

impl<const N: usize> ApplyToNewWidget for ScopedClasses<N> {
    fn apply_to_widget<W>(
        &self,
        new_widget: velona_core::masonry_core::core::NewWidget<W>,
    ) -> velona_core::masonry_core::core::NewWidget<W>
    where
        W: velona_core::masonry_core::core::Widget + ?Sized,
    {
        new_widget
            .with_classes(self.classes.into_iter().map(String::from))
            .apply(self.propstack_ref())
    }
}

impl<const N: usize> ApplyToWidgetMut for ScopedClasses<N> {
    fn apply_to_widget_mut<W>(&self, mut widget_mut: velona_core::masonry_core::core::WidgetMut<W>)
    where
        W: velona_core::masonry_core::core::Widget + ?Sized,
    {
        for class in self.classes {
            widget_mut.ctx.add_class(class);
        }
        self.prop_stack.apply_to_widget_mut(widget_mut);
    }
}
