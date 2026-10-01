use masonry_core::core::ErasedAction;
use winit_core::window::WindowId;

#[derive(Debug, Default)]
#[non_exhaustive]
pub enum ManagerErasedActionOrigin {
    #[default]
    App,
    Window(WindowId),
}

#[derive(derive_more::Debug)]
#[non_exhaustive]
pub struct ManagerErasedAction {
    pub action: ErasedAction,
    pub origin: ManagerErasedActionOrigin,
}
