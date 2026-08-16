use std::{collections::HashMap, fmt::Debug};

// use parking_lot::RwLock;

use log::{debug, warn};
use masonry_core::core::{ErasedAction, Widget, WidgetId};
use reactive_graph::owner::on_cleanup;

use crate::{
    utils::events::{EventMap, NoParamHandler},
    window::use_window,
};

pub use crate::utils::HandlerId;

pub type HandlerFn = Box<dyn Fn(&ErasedAction) + Send>;

pub type NoParamHandlerFn = NoParamHandler;

#[derive(derive_more::Debug)]
pub struct RegisterWindowEventHandler {
    pub handler_id: HandlerId,
    pub type_: RegisterWindowEventHandlerType,
}

#[derive(derive_more::Debug)]
pub enum RegisterWindowEventHandlerType {
    Widget(WidgetId, #[debug(skip)] HandlerFn),
    OnDestroy(#[debug(skip)] NoParamHandlerFn),
}

#[derive(Debug)]
pub enum UnregisterWindowEventHandlerType {
    Widget(Option<WidgetId>),
    OnDestroy,
}

#[derive(Debug)]
pub enum HandleEvent<'a> {
    Widget {
        widget_id: WidgetId,
        action: &'a ErasedAction,
    },
    OnDestroy,
}

#[derive(Default)]
pub(crate) struct WindowEventHandlers {
    widget_handlers: HashMap<WidgetId, EventMap<HandlerFn>>,
    on_destroy_handler: EventMap<NoParamHandlerFn>,
    // TODO add on_mouseenter for widgets
    // TODO add on_mouseexit for widgets
    // TODO add on_keydown for window
    // TODO add on_keyup for window
    // TODO add on_device_event for window
}

impl WindowEventHandlers {
    pub fn handle_event(&self, ev: HandleEvent<'_>) {
        match ev {
            HandleEvent::Widget { widget_id, action } => {
                let Some(handlers) = self.widget_handlers.get(&widget_id) else {
                    debug!("no event handler registered for {:?}", widget_id);
                    return;
                };
                handlers.values().for_each(|h| (h)(action));
            }
            HandleEvent::OnDestroy => {
                self.on_destroy_handler.values().for_each(|h| (h)());
            }
        }
    }
    pub fn add_handler_fn(&mut self, handler: RegisterWindowEventHandler) {
        match handler.type_ {
            RegisterWindowEventHandlerType::Widget(widget_id, handler_fn) => {
                self.widget_handlers
                    .entry(widget_id)
                    .or_default()
                    .insert(handler.handler_id, handler_fn);
            }
            RegisterWindowEventHandlerType::OnDestroy(handler_fn) => {
                self.on_destroy_handler
                    .insert(handler.handler_id, handler_fn);
            }
        }
    }
    pub fn cleanup(&mut self, render_root: &masonry_core::app::RenderRoot) {
        self.widget_handlers
            .retain(|widget_id, _| render_root.has_widget(*widget_id));
        self.widget_handlers.retain(|_, v| !v.is_empty());
    }
    pub(crate) fn shrink_to_fit(&mut self) {
        self.widget_handlers
            .values_mut()
            .for_each(|map| map.shrink_to_fit());
        self.widget_handlers.shrink_to_fit();
        self.on_destroy_handler.shrink_to_fit();
    }
    fn remove_handler_raw(&mut self, handler_id: &HandlerId) -> bool {
        if self.remove_widget_handler(handler_id, None) {
            true
        } else {
            self.remove_on_destroy_handler(handler_id)
        }
    }
    fn find_handler_widget_id(&self, handler_id: &HandlerId) -> Option<WidgetId> {
        self.widget_handlers
            .iter()
            .find(|(_, handlers)| handlers.contains_key(handler_id))
            .map(|(w, _)| w)
            .cloned()
    }
    fn remove_widget_handler(
        &mut self,
        handler_id: &HandlerId,
        widget_id: Option<WidgetId>,
    ) -> bool {
        let Some(widget_id) = widget_id.or_else(|| self.find_handler_widget_id(handler_id)) else {
            return false;
        };
        let Some(handlers) = self.widget_handlers.get_mut(&widget_id) else {
            return false;
        };
        handlers.remove(handler_id);
        let is_handlers_empty = handlers.is_empty();
        let _ = handlers;
        if is_handlers_empty {
            self.widget_handlers.remove(&widget_id);
        }
        true
    }
    fn remove_on_destroy_handler(&mut self, handler_id: &HandlerId) -> bool {
        self.on_destroy_handler.remove(handler_id).is_some()
    }
    pub(crate) fn remove_handler(
        &mut self,
        handler_id: &HandlerId,
        type_: Option<UnregisterWindowEventHandlerType>,
    ) -> bool {
        if let Some(type_) = type_ {
            match type_ {
                UnregisterWindowEventHandlerType::Widget(widget_id) => {
                    self.remove_widget_handler(handler_id, widget_id)
                }
                UnregisterWindowEventHandlerType::OnDestroy => {
                    self.remove_on_destroy_handler(handler_id)
                }
            }
        } else {
            self.remove_handler_raw(handler_id)
        }
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

/// Register a widget action handler
/// and automatically removes it [`on_cleanup`].
///
/// This function will fail if:
/// - there is no [`WindowHandle`](crate::window::WindowHandle) in the current context (panics on debug mode, just [`log::warn!`] on non-debug)
/// - the app or the window already closed (always panics)
///
/// For a typed version, use [`register_typed_widget_action_handler`].
pub fn register_widget_action_listener(widget_id: WidgetId, handler_fn: HandlerFn) {
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
        if let Err(err) = window.remove_widget_action_handler(handler_id, widget_id) {
            log::error!("{err}");
        }
    });
}

/// Very similar to [`register_widget_action_handler`]
/// but automatically cast the [`ErasedAction`] to the [`Widget::Action`] type.
///
/// The `handler_fn` function will just not run if the cast fails.
pub fn register_typed_widget_action_listener<W: Widget + 'static, H>(
    widget_id: WidgetId,
    handler_fn: H,
) where
    H: Fn(&<W as Widget>::Action) + Send + 'static,
{
    register_widget_action_listener(
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
