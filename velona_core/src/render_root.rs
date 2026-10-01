use masonry_core::{
    app::{RenderRoot, RenderRootOptions, RenderRootSignal},
    core::{NewWidget, Widget, WidgetId, WidgetMut},
};
use masonry_raw_box::RawBox;

/// The current render root
pub struct InnerRenderRoot {
    /// the root itself
    pub tree: RenderRoot,
    root_widget_id: WidgetId,
}

impl InnerRenderRoot {
    /// Create a new render root
    pub(crate) fn new(
        signal_sink: impl FnMut(RenderRootSignal) + 'static,
        options: RenderRootOptions,
    ) -> Self {
        let index_stack = RawBox::empty().prepare();
        let root_widget_id = index_stack.id();
        Self {
            tree: RenderRoot::new(index_stack, signal_sink, options),
            root_widget_id,
        }
    }
    /// Use the render root root widget
    pub fn use_root_widget_mut<F, R>(&mut self, to_use: F) -> Option<R>
    where
        F: FnOnce(WidgetMut<'_, RawBox>) -> R,
    {
        if self.tree.has_widget(self.root_widget_id) {
            self.tree
                .edit_widget(self.root_widget_id, |mut widget_mut| {
                    widget_mut.try_downcast::<RawBox>().map(to_use)
                })
        } else {
            None
        }
    }
    /// Swap the root widget
    pub fn swap_root_widget(&mut self, new_widget: NewWidget<dyn Widget + 'static>) {
        self.use_root_widget_mut(|mut root| {
            RawBox::set_child(&mut root, new_widget);
        });
    }
}
