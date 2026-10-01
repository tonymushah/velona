use bitflags::bitflags;
use dpi::PhysicalPosition;
use winit_core::event::FingerId;

bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct TouchStateFlag: u8 {
        const ENTERED = 1;
        const PRESSED = 1 << 1;
        // We don't need the MOVED flag here since it is unrelevant
        const RELEASED = 1 << 2;
        // const LEFT = 1 << 3;
    }
}

#[derive(Debug)]
pub struct TouchState {
    pub finger_id: FingerId,
    pub state: TouchStateFlag,
    pub last_position: PhysicalPosition<f64>,
}

impl TouchState {
    pub fn entered(finger_id: FingerId, last_position: PhysicalPosition<f64>) -> Self {
        Self {
            finger_id,
            state: TouchStateFlag::ENTERED,
            last_position,
        }
    }
}
