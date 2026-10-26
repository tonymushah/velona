use velona_core::masonry_core::core::{FromDynWidget, Property};

use crate::{ApplyScopedStyles, ApplyToNewWidget, ApplyToWidgetMut, ScopedPropstack};

#[derive(Debug)]
pub struct ScopedClasses<const N: usize> {
    classes: [&'static str; N],
    prop_stack: ScopedPropstack,
}

mod state;

pub use state::ScopedClassesState;

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
    pub fn static_prop<P>(self, state: ScopedClassesState, prop: P) -> Self
    where
        P: Property + Clone,
    {
        self.prop(state, move |_| prop.clone())
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
        W: velona_core::masonry_core::core::Widget + FromDynWidget + ?Sized,
    {
        new_widget
            .with_classes(self.classes.into_iter().map(String::from))
            .apply(self.propstack_ref())
    }
}

impl<const N: usize> ApplyToWidgetMut for ScopedClasses<N> {
    fn apply_to_widget_mut<W>(&self, mut widget_mut: velona_core::masonry_core::core::WidgetMut<W>)
    where
        W: velona_core::masonry_core::core::Widget + FromDynWidget + ?Sized,
    {
        for class in self.classes {
            widget_mut.ctx.add_class(class);
        }
        self.prop_stack.apply_to_widget_mut(widget_mut);
    }
}
