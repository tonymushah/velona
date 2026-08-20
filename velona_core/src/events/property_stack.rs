use futures_channel::oneshot;
use masonry_core::core::{PropertyStack, PropertyStackId, PropertyStackMut};
use winit::window::WindowId;

pub type EditFn = Box<dyn FnOnce(&mut PropertyStackMut<'_>) + Send + 'static>;

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
    Edit {
        id: PropertyStackId,
        #[debug(ignore)]
        edit_fn: EditFn,
    },
    Remove {
        id: PropertyStackId,
    },
    IsPresent {
        id: PropertyStackId,
        sender: oneshot::Sender<bool>,
    },
}
