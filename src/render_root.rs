use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use masonry::{
    app::{RenderRoot, RenderRootOptions, RenderRootSignal},
    core::{NewWidget, Widget, WidgetId, WidgetMut},
    widgets::IndexedStack,
};
use reactive_graph::owner::use_context;
use send_wrapper::SendWrapper;

pub struct InnerRenderRoot {
    pub tree: RenderRoot,
    pub root_widget_id: WidgetId,
}

impl InnerRenderRoot {
    pub fn new(
        signal_sink: impl FnMut(RenderRootSignal) + 'static,
        options: RenderRootOptions,
    ) -> Self {
        let index_stack = IndexedStack::new().with_auto_id();
        let root_widget_id = index_stack.id();
        Self {
            tree: RenderRoot::new(index_stack, signal_sink, options),
            root_widget_id,
        }
    }
    pub fn use_root_widget_mut<F, R>(&mut self, to_use: F) -> Option<R>
    where
        F: FnOnce(WidgetMut<'_, IndexedStack>) -> R,
    {
        if self.tree.has_widget(self.root_widget_id) {
            self.tree
                .edit_widget(self.root_widget_id, |mut widget_mut| {
                    if let Some(w) = widget_mut.try_downcast::<IndexedStack>() {
                        Some(to_use(w))
                    } else {
                        None
                    }
                })
        } else {
            None
        }
    }
    pub fn swap_root_widget(&mut self, new_widget: NewWidget<dyn Widget + 'static>) {
        self.use_root_widget_mut(|mut root| {
            if root.widget.is_empty() {
                IndexedStack::add_child(&mut root, new_widget);
            } else {
                IndexedStack::remove_child(&mut root, 0);
                IndexedStack::add_child(&mut root, new_widget);
            }
        });
    }
}

pub struct WindowRenderRoot {
    inner: SendWrapper<Rc<RefCell<InnerRenderRoot>>>,
}

impl WindowRenderRoot {
    pub fn new(render_root: InnerRenderRoot) -> Self {
        Self {
            inner: SendWrapper::new(Rc::new(RefCell::new(render_root))),
        }
    }
    pub fn use_inner_render_root_ref<Ufn, R>(&self, use_fn: Ufn) -> Option<R>
    where
        Ufn: FnOnce(&InnerRenderRoot) -> R,
    {
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
        if let Ok(mut _ref) = self.inner.try_borrow_mut() {
            Some(use_fn(&mut _ref))
        } else {
            log::warn!("The inner render root mutable reference is used somewhere else...");
            None
        }
    }
    pub fn create_weak(&self) -> WeakWindowRenderRoot {
        WeakWindowRenderRoot {
            inner: SendWrapper::new(Rc::downgrade(&self.inner)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct WeakWindowRenderRoot {
    inner: SendWrapper<Weak<RefCell<InnerRenderRoot>>>,
}

impl WeakWindowRenderRoot {
    pub fn use_inner_render_root_ref<Ufn, R>(&self, use_fn: Ufn) -> Option<R>
    where
        Ufn: FnOnce(&InnerRenderRoot) -> R,
    {
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
    pub fn use_inner_render_root_mut<Ufn, R>(&self, use_fn: Ufn) -> Option<R>
    where
        Ufn: FnOnce(&mut InnerRenderRoot) -> R,
    {
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

pub fn use_weak_render_root() -> Option<WeakWindowRenderRoot> {
    use_context()
}
