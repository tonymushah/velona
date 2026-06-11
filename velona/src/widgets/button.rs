use masonry::{
    core::{NewWidget, PointerButton},
    widgets::Button,
};
use reactive_graph::effect::Effect;

use crate::{AnyNewWidget, NewWidgetExt, utils::ConsumeResult};

/// A [new](NewWidget) [`Button`] exenstion trait
pub trait NewButton {
    /// Make the button [child](Button::set_child) reactive.
    fn child<C>(self, child: C) -> Self
    where
        C: Fn() -> AnyNewWidget + 'static;
}

impl NewButton for NewWidget<Button> {
    fn child<C>(self, child: C) -> Self
    where
        C: Fn() -> AnyNewWidget + 'static,
    {
        let v_ref = self.create_velona_ref();
        Effect::new(move || {
            let new_widget = child();
            v_ref
                .edit_local_now(|mut this| {
                    Button::set_child(&mut this, new_widget);
                })
                .consume_with_log_err();
        });
        self
    }
}

fn register_btn_ev<H>(
    btn: NewWidget<Button>,
    ptr_btn: Option<PointerButton>,
    handler: H,
) -> NewWidget<Button>
where
    H: Fn() + 'static,
{
    btn.register_handler(move |ev| {
        if ev.button == ptr_btn {
            handler()
        }
    })
}

macro_rules! btn_ev_trait {
    ($($ptr_ev:expr => {
        $(#[$attr:meta])* $ev_method:ident
    },)*) => {
        /// A useful wrapper trait for handling [`ButtonPress:button`] event easily
        pub trait NewButtonPressEventsExt {
            $(
                $(#[$attr])*
                fn $ev_method<F>(self, handler: F) -> Self
                    where F: Fn() + 'static;
            )*
        }

        impl NewButtonPressEventsExt for NewWidget<Button> {
            $(
                fn $ev_method<F>(self, handler: F) -> Self
                    where F: Fn()+ 'static
                {
                    register_btn_ev(self, $ptr_ev, handler)
                }
            )*
        }
    };
}

btn_ev_trait!(
    None => {
        /// The [`ButtonPress::button`] can be `None` when using for example the keyboard or a touch screen.
        on_nothing_press
    },

    Some(PointerButton::Primary)  => {
        on_primary
    },
    Some(PointerButton::Secondary)  => {
        on_secondary
    },
    Some(PointerButton::Auxiliary)  => {
        on_auxiliary
    },

    Some(PointerButton::X1)  => {
        on_x1
    },
    Some(PointerButton::X2)  => {
        on_x2
    },

    Some(PointerButton::PenEraser)  => {
        on_pen_eraser
    },

    Some(PointerButton::B7)  => {
        on_b7
    },
    Some(PointerButton::B8)  => {
        on_b8
    },
    Some(PointerButton::B9)  => {
        on_b9
    },

    Some(PointerButton::B10)  => {
        on_b10
    },
    Some(PointerButton::B11)  => {
        on_b11
    },
    Some(PointerButton::B12)  => {
        on_b12
    },
    Some(PointerButton::B13)  => {
        on_b13
    },
    Some(PointerButton::B14)  => {
        on_b14
    },
    Some(PointerButton::B15)  => {
        on_b15
    },
    Some(PointerButton::B16)  => {
        on_b16
    },
    Some(PointerButton::B17)  => {
        on_b17
    },
    Some(PointerButton::B18)  => {
        on_b18
    },
    Some(PointerButton::B19)  => {
        on_b19
    },

    Some(PointerButton::B20)  => {
        on_b20
    },
    Some(PointerButton::B21)  => {
        on_b21
    },
    Some(PointerButton::B22)  => {
        on_b22
    },
    Some(PointerButton::B23)  => {
        on_b23
    },
    Some(PointerButton::B24)  => {
        on_b24
    },
    Some(PointerButton::B25)  => {
        on_b25
    },
    Some(PointerButton::B26)  => {
        on_b26
    },
    Some(PointerButton::B27)  => {
        on_b27
    },
    Some(PointerButton::B28)  => {
        on_b28
    },
    Some(PointerButton::B29)  => {
        on_b29
    },

    Some(PointerButton::B30)  => {
        on_b30
    },
    Some(PointerButton::B31)  => {
        on_b31
    },
    Some(PointerButton::B32)  => {
        on_b32
    },
);
