use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::Debug,
    num::NonZeroU64,
    ops::Deref,
    rc::{Rc, Weak},
    sync::atomic::{AtomicU64, Ordering},
};

// use parking_lot::RwLock;

use log::debug;
use masonry::core::{ErasedAction, WidgetId};
use reactive_graph::owner::{on_cleanup, use_context};
use send_wrapper::SendWrapper;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct HandlerId(pub(crate) NonZeroU64);

impl HandlerId {
    /// Allocates a new, unique `WidgetId`.
    ///
    /// All widgets are assigned ids automatically; you should only create
    /// an explicit id if you need to know it ahead of time, for instance
    /// if you want two sibling widgets to know each others' ids.
    ///
    /// You must ensure that a given `WidgetId` is only ever used for one
    /// widget at a time.
    pub(crate) fn next() -> Self {
        static HANDLER_ID_COUNTER: AtomicU64 = AtomicU64::new(1);
        let id = HANDLER_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self(id.try_into().unwrap())
    }

    // Returns the integer value of the `WidgetId`.
    // pub fn to_raw(self) -> u64 {
    //     self.0.into()
    // }
}

pub type HandlerFn = Box<dyn Fn(&ErasedAction)>;

#[derive(Default)]
pub(crate) struct WindowEventHandler {
    widget_handlers: HashMap<WidgetId, HashMap<HandlerId, SendWrapper<HandlerFn>>>,
}

impl WindowEventHandler {
    pub fn handle_event(&self, widget_id: WidgetId, ev: &ErasedAction) {
        let Some(handlers) = self.widget_handlers.get(&widget_id) else {
            debug!("no event handler registered for {:?}", widget_id);
            return;
        };
        handlers.values().for_each(|h| (h)(ev));
    }
    pub fn add_handler_fn(&mut self, widget_id: WidgetId, hander_fn: HandlerFn) -> HandlerId {
        let handler_id = HandlerId::next();
        self.widget_handlers
            .entry(widget_id)
            .or_default()
            .entry(handler_id)
            .insert_entry(SendWrapper::new(hander_fn));
        handler_id
    }
    pub fn remove_handler_fn(&mut self, handler_id: HandlerId) {
        self.widget_handlers.retain(|_, v| {
            v.remove(&handler_id);
            !v.is_empty()
        });
    }
    pub fn cleanup(&mut self, render_root: &masonry::app::RenderRoot) {
        self.widget_handlers
            .retain(|widget_id, _| render_root.has_widget(*widget_id));
    }
    pub(crate) fn shrink_to_fit(&mut self) {
        self.widget_handlers
            .values_mut()
            .for_each(|map| map.shrink_to_fit());
        self.widget_handlers.shrink_to_fit();
    }
}

impl Debug for WindowEventHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindowEventHandle")
            .field(
                "widget_handlers",
                &self
                    .widget_handlers
                    .iter()
                    .map(|(id, handles)| (*id, handles.keys().collect::<Vec<_>>()))
                    .collect::<HashMap<_, _>>(),
            )
            .finish()
    }
}

pub(crate) struct InternWindowEventHandler(SendWrapper<Rc<RefCell<WindowEventHandler>>>);

impl Default for InternWindowEventHandler {
    fn default() -> Self {
        Self(SendWrapper::new(Rc::new(RefCell::new(
            WindowEventHandler::default(),
        ))))
    }
}

impl Deref for InternWindowEventHandler {
    type Target = RefCell<WindowEventHandler>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl InternWindowEventHandler {
    pub fn get_weak(&self) -> WindowEventHandlerWrapper {
        WindowEventHandlerWrapper(SendWrapper::new(Rc::downgrade(&*self.0)))
    }
}

#[derive(Debug, Clone)]
pub struct WindowEventHandlerWrapper(SendWrapper<Weak<RefCell<WindowEventHandler>>>);

impl WindowEventHandlerWrapper {
    pub fn add_handler_fn(&self, widget_id: WidgetId, hander_fn: HandlerFn) -> Option<HandlerId> {
        if !self.0.valid() {
            log::error!("An window event handler was called outside the main thread");
            return None;
        }
        let arc = self.0.upgrade()?;
        Some(
            arc.try_borrow_mut()
                .ok()?
                .add_handler_fn(widget_id, hander_fn),
        )
    }
    pub fn remove_handler_fn(&self, handler_id: HandlerId) {
        if !self.0.valid() {
            log::error!("An window event handler was called outside the main thread");
            return;
        }
        let Some(arc) = self.0.upgrade() else {
            return;
        };
        let Ok(mut evs) = arc.try_borrow_mut() else {
            return;
        };
        evs.remove_handler_fn(handler_id);
    }
}

pub fn use_window_event_handler() -> Option<WindowEventHandlerWrapper> {
    use_context()
}

pub fn register_widget_action_handler(widget_id: WidgetId, hander_fn: HandlerFn) {
    let Some(handlers) = use_window_event_handler() else {
        #[cfg(debug_assertions)]
        {
            panic!("No event handlers");
        }
        #[cfg(not(debug_assertions))]
        {
            log::warn!("No event handlers");
            return;
        }
    };
    let Some(handler_id) = handlers.add_handler_fn(widget_id, hander_fn) else {
        #[cfg(debug_assertions)]
        {
            panic!("Cannot register {} event handler", widget_id)
        }
        #[cfg(not(debug_assertions))]
        {
            log::warn!("Cannot register {} event handler", widget_id);
            return;
        }
    };

    on_cleanup(move || {
        handlers.remove_handler_fn(handler_id);
    });
}
