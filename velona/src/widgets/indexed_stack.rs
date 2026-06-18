use masonry::{core::NewWidget, widgets::IndexedStack};

use crate::NewWidgetExt;

/// A [new](NewWidget) [`IndexedStack`] extension trait.
pub trait NewIndexedStackExt {
    /// [Set the active child](IndexedStack::set_active_child) reactively.
    fn active_child<A>(self, idx: A) -> Self
    where
        A: Fn() -> usize + 'static;
}

impl NewIndexedStackExt for NewWidget<IndexedStack> {
    fn active_child<A>(self, idx: A) -> Self
    where
        A: Fn() -> usize + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            IndexedStack::set_active_child(&mut this, idx());
        })
    }
}
