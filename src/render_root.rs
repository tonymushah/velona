use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use masonry::{
    app::{RenderRoot, RenderRootOptions, RenderRootSignal},
    core::{NewWidget, Widget, WidgetId, WidgetMut},
    widgets::SizedBox,
};
use reactive_graph::owner::use_context;
use send_wrapper::SendWrapper;

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
        let index_stack = SizedBox::empty().prepare();
        let root_widget_id = index_stack.id();
        Self {
            tree: RenderRoot::new(index_stack, signal_sink, options),
            root_widget_id,
        }
    }
    /// Use the render root root widget
    pub fn use_root_widget_mut<F, R>(&mut self, to_use: F) -> Option<R>
    where
        F: FnOnce(WidgetMut<'_, SizedBox>) -> R,
    {
        if self.tree.has_widget(self.root_widget_id) {
            self.tree
                .edit_widget(self.root_widget_id, |mut widget_mut| {
                    widget_mut.try_downcast::<SizedBox>().map(to_use)
                })
        } else {
            None
        }
    }
    /// Swap the root widget
    pub fn swap_root_widget(&mut self, new_widget: NewWidget<dyn Widget + 'static>) {
        self.use_root_widget_mut(|mut root| {
            SizedBox::set_child(&mut root, new_widget);
        });
    }
}

pub(crate) struct WindowRenderRoot {
    inner: SendWrapper<Rc<RefCell<InnerRenderRoot>>>,
}

impl WindowRenderRoot {
    pub fn new(render_root: InnerRenderRoot) -> Self {
        Self {
            inner: SendWrapper::new(Rc::new(RefCell::new(render_root))),
        }
    }
    pub fn use_render_root_ref<Ufn, R>(&self, use_fn: Ufn) -> Option<R>
    where
        Ufn: FnOnce(&RenderRoot) -> R,
    {
        if !self.inner.valid() {
            log::error!("Trying to access a render root outside the main thread");
            return None;
        }
        if let Ok(_ref) = self.inner.try_borrow() {
            Some(use_fn(&_ref.tree))
        } else {
            log::warn!("The inner render root mutable reference is used somewhere else...");
            None
        }
    }
    pub fn use_render_root_mut<Ufn, R>(&self, use_fn: Ufn) -> Option<R>
    where
        Ufn: FnOnce(&mut RenderRoot) -> R,
    {
        if !self.inner.valid() {
            log::error!("Trying to access a render root outside the main thread");
            return None;
        }
        if let Ok(mut _ref) = self.inner.try_borrow_mut() {
            Some(use_fn(&mut _ref.tree))
        } else {
            log::warn!("The inner render root mutable reference is used somewhere else...");
            None
        }
    }
    pub fn use_inner_render_root_ref<Ufn, R>(&self, use_fn: Ufn) -> Option<R>
    where
        Ufn: FnOnce(&InnerRenderRoot) -> R,
    {
        if !self.inner.valid() {
            log::error!("Trying to access a render root outside the main thread");
            return None;
        }
        if let Ok(_ref) = self.inner.try_borrow() {
            Some(use_fn(&_ref))
        } else {
            log::warn!("The inner render root mutable reference is used somewhere else...");
            None
        }
    }
    pub fn use_inner_render_root_mut<Ufn, R>(&self, use_fn: Ufn) -> Option<R>
    where
        Ufn: FnOnce(&mut InnerRenderRoot) -> R,
    {
        if !self.inner.valid() {
            log::error!("Trying to access a render root outside the main thread");
            return None;
        }
        if let Ok(mut _ref) = self.inner.try_borrow_mut() {
            Some(use_fn(&mut _ref))
        } else {
            log::warn!("The inner render root mutable reference is used somewhere else...");
            None
        }
    }
    pub fn create_weak(&self) -> WindowRenderRootRef {
        WindowRenderRootRef {
            inner: SendWrapper::new(Rc::downgrade(&self.inner)),
        }
    }
}

/// **Do not send this type outside of the main thread**.
///
/// **Sending this type to another thread causes the thread to panic**
#[derive(Debug, Clone)]
pub struct WindowRenderRootRef {
    inner: SendWrapper<Weak<RefCell<InnerRenderRoot>>>,
}

impl WindowRenderRootRef {
    /// Use the current render root widget ref.
    ///
    /// Return [`None`] if :
    /// - called outside of the main thread
    /// - the intern render root was dropped
    /// - the intern render root is already used somewhere
    pub fn use_inner_render_root_ref<Ufn, R>(&self, use_fn: Ufn) -> Option<R>
    where
        Ufn: FnOnce(&InnerRenderRoot) -> R,
    {
        if !self.inner.valid() {
            log::error!("Trying to access a render root outside the main thread");
            return None;
        }
        let Some(inner) = self.inner.upgrade() else {
            log::warn!("The render root already dropped");
            return None;
        };
        if let Ok(_ref) = inner.try_borrow() {
            Some(use_fn(&_ref))
        } else {
            log::warn!("The inner render root mutable reference is used somewhere else...");
            None
        }
    }
    /// Use the current render root widget mutable reference.
    ///
    /// Return [`None`] if :
    /// - called outside of the main thread
    /// - the intern render root was dropped
    /// - the intern render root is already used somewhere
    pub fn use_inner_render_root_mut<Ufn, R>(&self, use_fn: Ufn) -> Option<R>
    where
        Ufn: FnOnce(&mut InnerRenderRoot) -> R,
    {
        if !self.inner.valid() {
            log::error!("Trying to access a render root outside the main thread");
            return None;
        }
        let Some(inner) = self.inner.upgrade() else {
            log::warn!("The render root already dropped");
            return None;
        };
        if let Ok(mut _ref) = inner.try_borrow_mut() {
            Some(use_fn(&mut _ref))
        } else {
            log::warn!("The inner render root mutable reference is used somewhere else...");
            None
        }
    }
}

/// Get the current window render root ref
pub fn use_window_render_root_ref() -> Option<WindowRenderRootRef> {
    use_context()
}
