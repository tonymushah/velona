use velona_core::reactive::owner::on_cleanup;
use velona_core::utils::HandlerFnGeneric;
use velona_core::window::event_listener::{OnKeyboardInput, OnPointerDoSomething};
use velona_core::{
    utils::{HandlerFnGenericStatic, NoParamHandlerFn},
    window::expect_window,
};
use winit_core::event::Modifiers;

macro_rules! functions {
    ($(
        $(#[$attr:meta])*
        $name:ident => [$register:ident($handler:ty), $unregister:ident],
    )*) => {
        $(
            $(#[$attr])*
            #[track_caller]
            pub fn $name(handler: $handler) {
                let window = expect_window();

                let handler_id = match window.$register(handler) {
                    Ok(id) => id,
                    Err(e) => {
                        log::error!("{e}");
                        return;
                    }
                };

                on_cleanup(move|| {
                   if let Err(err) = window.$unregister(handler_id) {
                       log::error!("{err}");
                   }
                });
            }
        )*
    };
}

functions! {
    on_destroy => [register_on_destroy_handler(NoParamHandlerFn), remove_on_destroy_handler],
    on_focused => [register_on_focused_handler(HandlerFnGenericStatic<bool>), remove_on_focused_handler],
    on_occluded => [register_on_occluded_handler(HandlerFnGenericStatic<bool>), remove_on_occluded_handler],
    on_cursor_entered => [register_on_cursor_entered_handler(HandlerFnGeneric<OnPointerDoSomething>), remove_on_cursor_entered_handler],
    on_cursor_left => [register_on_cursor_left_handler(HandlerFnGeneric<OnPointerDoSomething>), remove_on_cursor_left_handler],
    on_theme_changed => [register_on_theme_changed_handler(HandlerFnGeneric<winit_core::window::Theme>), remove_on_theme_changed_handler],
    on_keyboard_input => [register_on_keyboard_input_handler(HandlerFnGeneric<OnKeyboardInput>), remove_on_keyboard_input_handler],
    on_modifiers_changed => [register_on_modifiers_changed_handler(HandlerFnGeneric<Modifiers>), remove_on_modifiers_changed_handler],
}
