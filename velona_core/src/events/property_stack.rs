use futures_channel::oneshot;
use masonry_core::core::{PropertyStack, PropertyStackId};
use winit::window::WindowId;

#[derive(Debug)]
pub struct PropertyStackMethods {
    pub window_id: WindowId,
    pub type_: PropertyStackMethodsType,
}

#[derive(derive_more::Debug)]
pub enum PropertyStackMethodsType {
    Add {
        stack: PropertyStack,
        sender: oneshot::Sender<PropertyStackId>,
    },
    Replace {
        id: PropertyStackId,
        #[debug(ignore)]
        stack: PropertyStack,
    },
    Remove {
        id: PropertyStackId,
    },
    IsPresent {
        id: PropertyStackId,
        sender: oneshot::Sender<bool>,
    },
}
