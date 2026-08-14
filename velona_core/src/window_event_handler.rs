use std::{collections::HashMap, fmt::Debug};

// use parking_lot::RwLock;

use log::{debug, warn};
use masonry_core::core::{ErasedAction, Widget, WidgetId};
use reactive_graph::owner::on_cleanup;

use crate::window::use_window;

pub use crate::utils::HandlerId;

pub type HandlerFn = Box<dyn Fn(&ErasedAction) + Send>;

pub type NoParamHandlerFn = Box<dyn Fn() + Send>;

#[derive(Default)]
pub(crate) struct WindowEventHandlers {
    widget_handlers: HashMap<WidgetId, HashMap<HandlerId, HandlerFn>>,
    on_destroy_handler: HashMap<HandlerId, NoParamHandlerFn>,
    // TODO add on_mouseenter for widgets
    // TODO add on_mouseexit for widgets
    // TODO add on_keydown for window
    // TODO add on_keyup for window
    // TODO add on_device_event for window
}

impl WindowEventHandlers {
    pub fn handle_widget_action(&self, widget_id: WidgetId, ev: &ErasedAction) {
        let Some(handlers) = self.widget_handlers.get(&widget_id) else {
            debug!("no event handler registered for {:?}", widget_id);
            return;
        };
        handlers.values().for_each(|h| (h)(ev));
    }
    pub fn add_widget_action_handler_fn(
        &mut self,
        handler_id: HandlerId,
        widget_id: WidgetId,
        hander_fn: HandlerFn,
    ) {
        self.widget_handlers
            .entry(widget_id)
            .or_default()
            .entry(handler_id)
            .insert_entry(hander_fn);
    }
    pub fn remove_handler_fn(&mut self, handler_id: HandlerId) -> bool {
        let mut removed = false;
        self.widget_handlers.retain(|_, v| {
            removed = v.remove(&handler_id).is_some();
            !v.is_empty()
        });
        removed
    }
    pub fn cleanup(&mut self, render_root: &masonry_core::app::RenderRoot) {
        self.widget_handlers
            .retain(|widget_id, _| render_root.has_widget(*widget_id));
    }
    pub(crate) fn shrink_to_fit(&mut self) {
        self.widget_handlers
            .values_mut()
            .for_each(|map| map.shrink_to_fit());
        self.widget_handlers.shrink_to_fit();
        self.on_destroy_handler.shrink_to_fit();
    }
    pub fn add_on_destroy_handler(&mut self, handler_id: HandlerId, hander_fn: NoParamHandlerFn) {
        self.on_destroy_handler.insert(handler_id, hander_fn);
    }
}

impl Drop for WindowEventHandlers {
    fn drop(&mut self) {
        for (_, handler) in self.on_destroy_handler.drain() {
            handler();
        }
    }
}

impl Debug for WindowEventHandlers {
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

// TODO add documentation
pub fn register_widget_action_handler(widget_id: WidgetId, handler_fn: HandlerFn) {
    let Some(window) = use_window() else {
        #[cfg(debug_assertions)]
        {
            panic!("No window handle found in the current context");
        }
        #[cfg(not(debug_assertions))]
        {
            log::warn!("No window handle found in the current context");
            return;
        }
    };
    let handler_id = window
        .register_action_handler(widget_id, handler_fn)
        .unwrap();

    on_cleanup(move || {
        if let Err(err) = window.remove_handler(handler_id) {
            log::error!("{err}");
        }
    });
}

// TODO add documentation
pub fn register_typed_widget_action_handler<W: Widget + 'static, H>(
    widget_id: WidgetId,
    handler_fn: H,
) where
    H: Fn(&<W as Widget>::Action) + Send + 'static,
{
    register_widget_action_handler(
        widget_id,
        Box::new(move |ev| {
            let Some(ev) = ev.downcast_ref::<W::Action>() else {
                warn!("Cannot cast action");
                return;
            };
            handler_fn(ev);
        }),
    );
}
