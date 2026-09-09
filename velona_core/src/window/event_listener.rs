use std::{collections::HashMap, fmt::Debug};

// use parking_lot::RwLock;

use log::{debug, warn};
use masonry_core::core::{ErasedAction, Widget, WidgetId};
use reactive_graph::owner::{Owner, on_cleanup};
use winit::event::{DeviceId, KeyEvent, Modifiers, WindowEvent};

use crate::{
    utils::events::{EventMap, NoParamHandler},
    window::use_window,
};

pub use crate::utils::HandlerId;

pub type HandlerFn = HandlerFnGeneric<ErasedAction>;

pub type HandlerFnGeneric<T> = Box<dyn Fn(&T) + Send>;

pub type HandlerFnGenericStatic<T> = Box<dyn Fn(T) + Send>;

pub type NoParamHandlerFn = NoParamHandler;

#[derive(derive_more::Debug)]
pub struct RegisterWindowEventHandler {
    pub handler_id: HandlerId,
    pub type_: RegisterWindowEventHandlerType,
}

#[derive(Debug)]
#[non_exhaustive]
pub struct OnKeyboardInput {
    pub device_id: DeviceId,
    pub event: KeyEvent,
    pub is_synthetic: bool,
}

#[derive(derive_more::Debug)]
pub enum RegisterWindowEventHandlerType {
    Widget(WidgetId, #[debug(skip)] HandlerFn),
    OnDestroy(#[debug(skip)] NoParamHandlerFn),
    OnFocused(#[debug(skip)] HandlerFnGenericStatic<bool>),
    // breaking changes for this one see #130
    // OnDropFile(#[debug(skip)] HandlerFnGeneric<>)
    OnOccluded(#[debug(skip)] HandlerFnGenericStatic<bool>),
    OnCursorEntered(#[debug(skip)] HandlerFnGeneric<DeviceId>),
    OnCursorLeft(#[debug(skip)] HandlerFnGeneric<DeviceId>),
    OnThemeChanged(#[debug(skip)] HandlerFnGeneric<winit::window::Theme>),
    OnKeyboardInput(#[debug(skip)] HandlerFnGeneric<OnKeyboardInput>),
    OnModifiersChanged(#[debug(skip)] HandlerFnGeneric<Modifiers>),
}

#[derive(Debug)]
pub enum UnregisterWindowEventHandlerType {
    Widget(Option<WidgetId>),
    OnDestroy,
    OnFocused,
    OnOccluded,
    OnCursorEntered,
    OnCursorLeft,
    OnThemeChanged,
    OnKeyboardInput,
    OnModifiersChanged,
}

#[derive(Debug)]
pub enum HandleEvent<'a> {
    Widget {
        widget_id: WidgetId,
        action: &'a ErasedAction,
    },
    WindowEvent(&'a WindowEvent),
}

#[derive(Default)]
pub(crate) struct WindowEventHandlers {
    widget_handlers: HashMap<WidgetId, EventMap<HandlerFn>>,
    on_destroy_handler: EventMap<NoParamHandlerFn>,
    on_focused_handler: EventMap<HandlerFnGenericStatic<bool>>,
    on_occluded_handler: EventMap<HandlerFnGenericStatic<bool>>,
    on_cursor_entered_handler: EventMap<HandlerFnGeneric<DeviceId>>,
    on_cursor_left_handler: EventMap<HandlerFnGeneric<DeviceId>>,
    on_theme_changed_handler: EventMap<HandlerFnGeneric<winit::window::Theme>>,
    on_keyboard_input_handler: EventMap<HandlerFnGeneric<OnKeyboardInput>>,
    on_modifiers_changed_handler: EventMap<HandlerFnGeneric<Modifiers>>,
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
            HandleEvent::WindowEvent(event) => match event {
                WindowEvent::Destroyed => {
                    self.on_destroy_handler.values().for_each(|h| (h)());
                }
                WindowEvent::Focused(is_focused) => {
                    self.on_focused_handler
                        .values()
                        .for_each(|h| (h)(*is_focused));
                }
                WindowEvent::Occluded(is_occluded) => {
                    self.on_occluded_handler
                        .values()
                        .for_each(|h| (h)(*is_occluded));
                }
                WindowEvent::CursorEntered { device_id } => {
                    self.on_cursor_entered_handler
                        .values()
                        .for_each(|h| (h)(device_id));
                }
                WindowEvent::CursorLeft { device_id } => {
                    self.on_cursor_left_handler
                        .values()
                        .for_each(|h| (h)(device_id));
                }
                WindowEvent::ThemeChanged(theme) => {
                    self.on_theme_changed_handler
                        .values()
                        .for_each(|h| (h)(theme));
                }
                WindowEvent::KeyboardInput {
                    device_id,
                    event,
                    is_synthetic,
                } => {
                    let to_send = OnKeyboardInput {
                        device_id: *device_id,
                        event: event.clone(),
                        is_synthetic: *is_synthetic,
                    };
                    self.on_keyboard_input_handler
                        .values()
                        .for_each(|h| (h)(&to_send));
                }
                WindowEvent::ModifiersChanged(modifiers) => {
                    self.on_modifiers_changed_handler
                        .values()
                        .for_each(|h| (h)(modifiers));
                }
                _ => {}
            },
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
            RegisterWindowEventHandlerType::OnFocused(handler_fn) => {
                self.on_focused_handler
                    .insert(handler.handler_id, handler_fn);
            }
            RegisterWindowEventHandlerType::OnOccluded(handler_fn) => {
                self.on_occluded_handler
                    .insert(handler.handler_id, handler_fn);
            }
            RegisterWindowEventHandlerType::OnCursorEntered(handler_fn) => {
                self.on_cursor_entered_handler
                    .insert(handler.handler_id, handler_fn);
            }
            RegisterWindowEventHandlerType::OnCursorLeft(handler_fn) => {
                self.on_cursor_left_handler
                    .insert(handler.handler_id, handler_fn);
            }
            RegisterWindowEventHandlerType::OnThemeChanged(handler_fn) => {
                self.on_theme_changed_handler
                    .insert(handler.handler_id, handler_fn);
            }
            RegisterWindowEventHandlerType::OnKeyboardInput(handler_fn) => {
                self.on_keyboard_input_handler
                    .insert(handler.handler_id, handler_fn);
            }
            RegisterWindowEventHandlerType::OnModifiersChanged(handler_fn) => {
                self.on_modifiers_changed_handler
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

    fn find_handler_widget_id(&self, handler_id: &HandlerId) -> Option<WidgetId> {
        self.widget_handlers
            .iter()
            .find(|(_, handlers)| handlers.contains_key(handler_id))
            .map(|(w, _)| w)
            .cloned()
    }

    pub fn remove_handler(
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
                UnregisterWindowEventHandlerType::OnFocused => {
                    self.remove_on_focused_handler(handler_id)
                }
                UnregisterWindowEventHandlerType::OnOccluded => {
                    self.remove_on_occluded_handler(handler_id)
                }
                UnregisterWindowEventHandlerType::OnCursorEntered => {
                    self.remove_on_cursor_entered_handler(handler_id)
                }
                UnregisterWindowEventHandlerType::OnCursorLeft => {
                    self.remove_on_cursor_left_handler(handler_id)
                }
                UnregisterWindowEventHandlerType::OnThemeChanged => {
                    self.remove_on_theme_changed_handler(handler_id)
                }
                UnregisterWindowEventHandlerType::OnKeyboardInput => {
                    self.remove_on_keyboard_input_handler(handler_id)
                }
                UnregisterWindowEventHandlerType::OnModifiersChanged => {
                    self.remove_on_modifiers_changed_handler(handler_id)
                }
            }
        } else {
            self.remove_handler_raw(handler_id)
        }
    }
}

impl WindowEventHandlers {
    fn remove_handler_raw(&mut self, handler_id: &HandlerId) -> bool {
        macro_rules! handle_remove {
            ($($func:ident, )*) => {
                $(
                    self.$func(handler_id) ||
                )* false
            };
        }
        // TODO add the others
        handle_remove!(
            remove_widget_handler_none,
            remove_on_destroy_handler,
            remove_on_focused_handler,
            remove_on_occluded_handler,
            remove_on_cursor_entered_handler,
            remove_on_theme_changed_handler,
            remove_on_keyboard_input_handler,
            remove_on_modifiers_changed_handler,
        )
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
    fn remove_widget_handler_none(&mut self, handler_id: &HandlerId) -> bool {
        self.remove_widget_handler(handler_id, None)
    }
    fn remove_on_destroy_handler(&mut self, handler_id: &HandlerId) -> bool {
        self.on_destroy_handler.remove(handler_id).is_some()
    }
    fn remove_on_focused_handler(&mut self, handler_id: &HandlerId) -> bool {
        self.on_focused_handler.remove(handler_id).is_some()
    }
    fn remove_on_occluded_handler(&mut self, handler_id: &HandlerId) -> bool {
        self.on_occluded_handler.remove(handler_id).is_some()
    }
    fn remove_on_cursor_entered_handler(&mut self, handler_id: &HandlerId) -> bool {
        self.on_cursor_entered_handler.remove(handler_id).is_some()
    }
    fn remove_on_cursor_left_handler(&mut self, handler_id: &HandlerId) -> bool {
        self.on_cursor_left_handler.remove(handler_id).is_some()
    }
    fn remove_on_theme_changed_handler(&mut self, handler_id: &HandlerId) -> bool {
        self.on_theme_changed_handler.remove(handler_id).is_some()
    }
    fn remove_on_keyboard_input_handler(&mut self, handler_id: &HandlerId) -> bool {
        self.on_keyboard_input_handler.remove(handler_id).is_some()
    }
    fn remove_on_modifiers_changed_handler(&mut self, handler_id: &HandlerId) -> bool {
        self.on_modifiers_changed_handler
            .remove(handler_id)
            .is_some()
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
            // TODO add more attributes
            .finish_non_exhaustive()
    }
}

/// Register a widget action handler
/// and automatically removes it [`on_cleanup`].
///
/// This function will fail if:
/// - there is no [`WindowHandle`](crate::window::WindowHandle) in the current context (panics on debug mode, just [`log::warn!`] on non-debug)
/// - the app or the window already closed (always panics)
///
/// For a typed version, use [`register_typed_widget_action_listener`].
pub fn register_widget_action_listener(widget_id: WidgetId, mut handler_fn: HandlerFn) {
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
    if let Some(current) = Owner::current() {
        let to_send = current.child();
        handler_fn = Box::new(move |e| {
            to_send.with(|| {
                handler_fn(e);
            })
        })
    }
    let handler_id = window
        .register_action_handler(widget_id, handler_fn)
        .unwrap();

    on_cleanup(move || {
        if let Err(err) = window.remove_widget_action_handler(handler_id, widget_id) {
            log::error!("{err}");
        }
    });
}

/// Very similar to [`register_widget_action_listener`]
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

// TODO add tests
