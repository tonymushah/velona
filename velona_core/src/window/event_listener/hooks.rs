use log::warn;
use masonry_core::core::{Widget, WidgetId};
use reactive_graph::owner::{Owner, on_cleanup};
use winit::event::{DeviceId, Modifiers};

use crate::window::{
    WindowHandle,
    event_listener::{HandlerFnGeneric, HandlerFnGenericStatic, NoParamHandlerFn, OnKeyboardInput},
    use_window,
};

use super::HandlerFn;

#[track_caller]
fn use_window_with_panic() -> WindowHandle {
    let Some(window) = use_window() else {
        #[cfg(debug_assertions)]
        {
            panic!(
                "No window handle found in the current context at {}",
                std::panic::Location::caller()
            );
        }
        #[cfg(not(debug_assertions))]
        {
            log::warn!("No window handle found in the current context");
            return;
        }
    };
    window
}

#[track_caller]
/// Register a widget action handler
/// and automatically removes it [`on_cleanup`].
///
/// This function will fail if:
/// - there is no [`WindowHandle`](crate::window::WindowHandle) in the current context (panics on debug mode, just [`log::warn!`] on non-debug)
/// - the app or the window already closed (always panics)
///
/// For a typed version, use [`register_typed_widget_action_listener`].
pub fn register_widget_action_listener(widget_id: WidgetId, mut handler_fn: HandlerFn) {
    let window = use_window_with_panic();
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

#[track_caller]
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

#[track_caller]
pub fn register_on_window_destroy_listener(mut handler_fn: NoParamHandlerFn) {
    let window = use_window_with_panic();
    if let Some(current) = Owner::current() {
        let to_send = current.child();
        handler_fn = Box::new(move || {
            to_send.with(|| {
                handler_fn();
            })
        })
    }
    let handler_id = window.register_on_destroy_handler(handler_fn).unwrap();

    on_cleanup(move || {
        if let Err(err) = window.remove_on_destroy_handler(handler_id) {
            log::error!("{err}");
        }
    });
}

#[track_caller]
pub fn register_on_window_focused_listener(mut handler_fn: HandlerFnGenericStatic<bool>) {
    let window = use_window_with_panic();
    if let Some(current) = Owner::current() {
        let to_send = current.child();
        handler_fn = Box::new(move |e| {
            to_send.with(|| {
                handler_fn(e);
            })
        })
    }
    let handler_id = window.register_on_focused_handler(handler_fn).unwrap();

    on_cleanup(move || {
        if let Err(err) = window.remove_on_focused_handler(handler_id) {
            log::error!("{err}");
        }
    });
}

#[track_caller]
pub fn register_on_window_occluded_listener(mut handler_fn: HandlerFnGenericStatic<bool>) {
    let window = use_window_with_panic();
    if let Some(current) = Owner::current() {
        let to_send = current.child();
        handler_fn = Box::new(move |e| {
            to_send.with(|| {
                handler_fn(e);
            })
        })
    }
    let handler_id = window.register_on_occluded_handler(handler_fn).unwrap();

    on_cleanup(move || {
        if let Err(err) = window.remove_on_occluded_handler(handler_id) {
            log::error!("{err}");
        }
    });
}

#[track_caller]
pub fn register_on_window_cursor_entered_listener(mut handler_fn: HandlerFnGeneric<DeviceId>) {
    let window = use_window_with_panic();
    if let Some(current) = Owner::current() {
        let to_send = current.child();
        handler_fn = Box::new(move |e| {
            to_send.with(|| {
                handler_fn(e);
            })
        })
    }
    let handler_id = window
        .register_on_cursor_entered_handler(handler_fn)
        .unwrap();

    on_cleanup(move || {
        if let Err(err) = window.remove_on_cursor_entered_handler(handler_id) {
            log::error!("{err}");
        }
    });
}

#[track_caller]
pub fn register_on_window_cursor_left_listener(mut handler_fn: HandlerFnGeneric<DeviceId>) {
    let window = use_window_with_panic();
    if let Some(current) = Owner::current() {
        let to_send = current.child();
        handler_fn = Box::new(move |e| {
            to_send.with(|| {
                handler_fn(e);
            })
        })
    }
    let handler_id = window.register_on_cursor_left_handler(handler_fn).unwrap();

    on_cleanup(move || {
        if let Err(err) = window.remove_on_cursor_left_handler(handler_id) {
            log::error!("{err}");
        }
    });
}

#[track_caller]
pub fn register_on_theme_changed_listener(mut handler_fn: HandlerFnGeneric<winit::window::Theme>) {
    let window = use_window_with_panic();
    if let Some(current) = Owner::current() {
        let to_send = current.child();
        handler_fn = Box::new(move |e| {
            to_send.with(|| {
                handler_fn(e);
            })
        })
    }
    let handler_id = window
        .register_on_theme_changed_handler(handler_fn)
        .unwrap();

    on_cleanup(move || {
        if let Err(err) = window.remove_on_theme_changed_handler(handler_id) {
            log::error!("{err}");
        }
    });
}

#[track_caller]
pub fn register_on_window_keyboard_input_listener(
    mut handler_fn: HandlerFnGeneric<OnKeyboardInput>,
) {
    let window = use_window_with_panic();
    if let Some(current) = Owner::current() {
        let to_send = current.child();
        handler_fn = Box::new(move |e| {
            to_send.with(|| {
                handler_fn(e);
            })
        })
    }
    let handler_id = window
        .register_on_keyboard_input_handler(handler_fn)
        .unwrap();

    on_cleanup(move || {
        if let Err(err) = window.remove_on_keyboard_input_handler(handler_id) {
            log::error!("{err}");
        }
    });
}

#[track_caller]
pub fn register_on_window_modifiers_changed_listener(mut handler_fn: HandlerFnGeneric<Modifiers>) {
    let window = use_window_with_panic();
    if let Some(current) = Owner::current() {
        let to_send = current.child();
        handler_fn = Box::new(move |e| {
            to_send.with(|| {
                handler_fn(e);
            })
        })
    }
    let handler_id = window
        .register_on_modifiers_changed_handler(handler_fn)
        .unwrap();

    on_cleanup(move || {
        if let Err(err) = window.remove_on_modifiers_changed_handler(handler_id) {
            log::error!("{err}");
        }
    });
}
