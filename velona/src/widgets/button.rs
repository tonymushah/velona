use masonry::{
    core::{NewWidget, PointerButton},
    widgets::Button,
};

#[cfg(doc)]
use masonry::widgets::ButtonPress;

use crate::NewWidgetExt;

fn register_btn_ev<H>(
    btn: NewWidget<Button>,
    ptr_btn: Option<PointerButton>,
    handler: H,
) -> NewWidget<Button>
where
    H: Fn() + 'static,
{
    btn.on(move |ev| {
        if ev.button == ptr_btn {
            handler()
        }
    })
}

macro_rules! btn_ev_trait {
    ($($ptr_ev:expr => {
        $(#[$attr:meta])* $ev_method:ident
    },)*) => {
        /// A useful wrapper trait for handling [`ButtonPress::button`] event easily
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
        /// Primary button, commonly the left mouse button, touch contact, pen contact.
        on_primary
    },
    Some(PointerButton::Secondary)  => {
        /// Secondary button, commonly the right mouse button, pen barrel button.
        on_secondary
    },
    Some(PointerButton::Auxiliary)  => {
        /// Auxiliary button, commonly the middle mouse button.
        on_auxiliary
    },

    Some(PointerButton::X1)  => {
        /// X1 (back) Mouse.
        on_x1
    },
    Some(PointerButton::X2)  => {
        /// X2 (forward) Mouse.
        on_x2
    },

    Some(PointerButton::PenEraser)  => {
        /// Pen erase button.
        on_pen_eraser
    },

    Some(PointerButton::B7)  => {
        /// Button 7.
        on_b7
    },
    Some(PointerButton::B8)  => {
        /// Button 8.
        on_b8
    },
    Some(PointerButton::B9)  => {
        /// Button 9.
        on_b9
    },

    Some(PointerButton::B10)  => {
        /// Button 10.
        on_b10
    },
    Some(PointerButton::B11)  => {
        /// Button 11.
        on_b11
    },
    Some(PointerButton::B12)  => {
        /// Button 12.
        on_b12
    },
    Some(PointerButton::B13)  => {
        /// Button 13.
        on_b13
    },
    Some(PointerButton::B14)  => {
        /// Button 14.
        on_b14
    },
    Some(PointerButton::B15)  => {
        /// Button 15.
        on_b15
    },
    Some(PointerButton::B16)  => {
        /// Button 16.
        on_b16
    },
    Some(PointerButton::B17)  => {
        /// Button 17.
        on_b17
    },
    Some(PointerButton::B18)  => {
        /// Button 18.
        on_b18
    },
    Some(PointerButton::B19)  => {
        /// Button 19.
        on_b19
    },

    Some(PointerButton::B20)  => {
        /// Button 20.
        on_b20
    },
    Some(PointerButton::B21)  => {
        /// Button 21.
        on_b21
    },
    Some(PointerButton::B22)  => {
        /// Button 22.
        on_b22
    },
    Some(PointerButton::B23)  => {
        /// Button 23.
        on_b23
    },
    Some(PointerButton::B24)  => {
        /// Button 24.
        on_b24
    },
    Some(PointerButton::B25)  => {
        /// Button 25.
        on_b25
    },
    Some(PointerButton::B26)  => {
        /// Button 26.
        on_b26
    },
    Some(PointerButton::B27)  => {
        /// Button 27.
        on_b27
    },
    Some(PointerButton::B28)  => {
        /// Button 28.
        on_b28
    },
    Some(PointerButton::B29)  => {
        /// Button 29.
        on_b29
    },

    Some(PointerButton::B30)  => {
        /// Button 30.
        on_b30
    },
    Some(PointerButton::B31)  => {
        /// Button 31.
        on_b31
    },
    Some(PointerButton::B32)  => {
        /// Button 32.
        on_b32
    },
);
