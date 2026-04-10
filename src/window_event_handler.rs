use std::{
    collections::HashMap,
    fmt::Debug,
    ops::Deref,
    sync::{Arc, Weak},
};

use parking_lot::RwLock;

use log::debug;
use masonry::core::{ErasedAction, WidgetId};
use reactive_graph::owner::{on_cleanup, use_context};
use send_wrapper::SendWrapper;
use uuid::Uuid;

pub type HandlerFn = Box<dyn Fn(&ErasedAction)>;

#[derive(Default)]
pub struct WindowEventHandler {
    widget_handlers: HashMap<WidgetId, HashMap<Uuid, SendWrapper<HandlerFn>>>,
}

impl WindowEventHandler {
    pub fn handle_event(&self, widget_id: WidgetId, ev: &ErasedAction) {
        let Some(handlers) = self.widget_handlers.get(&widget_id) else {
            debug!("no event handler registered for {:?}", widget_id);
            return;
        };
        handlers.values().for_each(|h| (h)(ev));
    }
    pub fn add_handler_fn(&mut self, widget_id: WidgetId, hander_fn: HandlerFn) -> Uuid {
        let handler_id = Uuid::new_v4();
        self.widget_handlers
            .entry(widget_id)
            .or_default()
            .entry(handler_id)
            .insert_entry(SendWrapper::new(hander_fn));
        handler_id
    }
    pub fn remove_handler_fn(&mut self, handler_id: Uuid) {
        self.widget_handlers.retain(|_, v| {
            v.remove(&handler_id);
            !v.is_empty()
        });
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

#[derive(Default)]
pub(crate) struct InternWindowEventHandler(Arc<RwLock<WindowEventHandler>>);

impl Deref for InternWindowEventHandler {
    type Target = RwLock<WindowEventHandler>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl InternWindowEventHandler {
    pub fn get_weak(&self) -> WindowEventHandlerWrapper {
        WindowEventHandlerWrapper(Arc::downgrade(&self.0))
    }
}

#[derive(Debug, Clone)]
pub struct WindowEventHandlerWrapper(Weak<RwLock<WindowEventHandler>>);

impl WindowEventHandlerWrapper {
    pub fn add_handler_fn(&self, widget_id: WidgetId, hander_fn: HandlerFn) -> Option<Uuid> {
        let arc = self.0.upgrade()?;
        Some(arc.write().add_handler_fn(widget_id, hander_fn))
    }
    pub fn remove_handler_fn(&self, handler_id: Uuid) {
        let Some(arc) = self.0.upgrade() else {
            return;
        };
        arc.write().remove_handler_fn(handler_id);
    }
}

pub fn use_window_event_handler() -> Option<WindowEventHandlerWrapper> {
    use_context()
}

pub fn register_window_event_handler(widget_id: WidgetId, hander_fn: HandlerFn) {
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
