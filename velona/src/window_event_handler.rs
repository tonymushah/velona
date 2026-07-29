use std::{
    collections::HashMap,
    fmt::Debug,
    num::NonZeroU64,
    sync::atomic::{AtomicU64, Ordering},
};

// use parking_lot::RwLock;

use log::{debug, warn};
use masonry::core::{ErasedAction, Widget, WidgetId};
use reactive_graph::owner::on_cleanup;

use crate::window::use_window;

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
    pub fn cleanup(&mut self, render_root: &masonry::app::RenderRoot) {
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

// pub(crate) struct InternWindowEventHandler(SendWrapper<Rc<RefCell<WindowEventHandler>>>);

// impl Default for InternWindowEventHandler {
//     fn default() -> Self {
//         Self(SendWrapper::new(Rc::new(RefCell::new(
//             WindowEventHandler::default(),
//         ))))
//     }
// }

// impl Deref for InternWindowEventHandler {
//     type Target = RefCell<WindowEventHandler>;
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl InternWindowEventHandler {
//     pub fn get_weak(&self) -> WindowEventHandlerWrapper {
//         WindowEventHandlerWrapper(SendWrapper::new(Rc::downgrade(&*self.0)))
//     }
// }

// #[derive(Debug, Clone)]
// pub struct WindowEventHandlerWrapper(SendWrapper<Weak<RefCell<WindowEventHandler>>>);

// impl WindowEventHandlerWrapper {
//     pub fn add_handler_fn(&self, widget_id: WidgetId, hander_fn: HandlerFn) -> Option<HandlerId> {
//         if !self.0.valid() {
//             log::error!("An window event handler was called outside the main thread");
//             return None;
//         }
//         let arc = self.0.upgrade()?;
//         Some(
//             arc.try_borrow_mut()
//                 .ok()?
//                 .add_handler_fn(widget_id, hander_fn),
//         )
//     }
//     pub fn remove_handler_fn(&self, handler_id: HandlerId) {
//         if !self.0.valid() {
//             log::error!("An window event handler was called outside the main thread");
//             return;
//         }
//         let Some(arc) = self.0.upgrade() else {
//             return;
//         };
//         let Ok(mut evs) = arc.try_borrow_mut() else {
//             return;
//         };
//         evs.remove_handler_fn(handler_id);
//     }
// }

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
