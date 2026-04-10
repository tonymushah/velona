use masonry::core::{ErasedAction, NewWidget, Widget, WidgetId};
use send_wrapper::SendWrapper;
use winit::{event_loop::EventLoopProxy, window::WindowId};

use crate::window::WindowBuilder;

pub(crate) struct NewLayer(pub(crate) SendWrapper<NewWidget<dyn Widget + 'static>>);

impl From<NewWidget<dyn Widget + 'static>> for NewLayer {
    fn from(value: NewWidget<dyn Widget + 'static>) -> Self {
        Self(SendWrapper::new(value))
    }
}

pub(crate) struct RenderRootNewLayer {
    pub window_id: WindowId,
    pub layer: NewLayer,
    pub point: masonry::kurbo::Point,
}

pub(crate) struct RenderRootRemoveLayer {
    pub window_id: WindowId,
    pub widget_id: WidgetId,
}

pub(crate) struct RenderRootRepositionLayer {
    pub window_id: WindowId,
    pub widget_id: WidgetId,
    pub point: masonry::kurbo::Point,
}

#[derive(Debug)]
pub struct WidgetAction {
    pub window_id: WindowId,
    pub widget_id: WidgetId,
    pub event: SendWrapper<ErasedAction>,
}

pub(crate) enum EventLoopEvent {
    AccessKitAction(Box<accesskit_winit::Event>),
    RunTask(Box<async_task::Runnable>),
    NewLayer(Box<RenderRootNewLayer>),
    RemoveLayer(Box<RenderRootRemoveLayer>),
    RepositionLayer(Box<RenderRootRepositionLayer>),
    NewWindow(Box<WindowBuilder>),
    WidgetAction(Box<WidgetAction>),
}

pub(crate) type AppEventLoopProxy = EventLoopProxy<EventLoopEvent>;

impl From<accesskit_winit::Event> for EventLoopEvent {
    fn from(value: accesskit_winit::Event) -> Self {
        Self::AccessKitAction(Box::new(value))
    }
}

impl From<RenderRootNewLayer> for EventLoopEvent {
    fn from(value: RenderRootNewLayer) -> Self {
        Self::NewLayer(Box::new(value))
    }
}

impl From<RenderRootRemoveLayer> for EventLoopEvent {
    fn from(value: RenderRootRemoveLayer) -> Self {
        Self::RemoveLayer(Box::new(value))
    }
}

impl From<RenderRootRepositionLayer> for EventLoopEvent {
    fn from(value: RenderRootRepositionLayer) -> Self {
        Self::RepositionLayer(Box::new(value))
    }
}

impl From<WidgetAction> for EventLoopEvent {
    fn from(value: WidgetAction) -> Self {
        Self::WidgetAction(Box::new(value))
    }
}

#[cfg(test)]
mod test {
    use crate::utils::is_send_sync;

    #[test]
    fn test_if_event_is_send_sync() {
        is_send_sync::<super::EventLoopEvent>();
    }
    #[test]
    fn test_if_new_layer_is_send_sync() {
        is_send_sync::<super::NewLayer>();
    }
}
