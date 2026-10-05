use masonry_raw_box::RawBox;
use masonry_widget_wrappers::WidgetWrapper;
use tracing::{Span, trace_span};

use masonry::core::{NewWidget, NoAction, Widget, WidgetId};
use velona::masonry;

pub struct TitleBarContainer {
    inner: RawBox,
}

// --- MARK: BUILDERS
impl TitleBarContainer {
    pub fn new(child: NewWidget<impl Widget + ?Sized>) -> Self {
        Self {
            inner: RawBox::new(child),
        }
    }
}

impl WidgetWrapper for TitleBarContainer {
    type Action
        = NoAction
    where
        Self: Sized;
    fn inner(&self) -> &dyn Widget {
        &self.inner
    }
    fn inner_mut(&mut self) -> &mut dyn Widget {
        &mut self.inner
    }
    fn make_trace_span(&self, id: WidgetId) -> Span {
        trace_span!("TitleBarContainer", id = id.trace())
    }
    fn on_pointer_event(
        &mut self,
        ctx: &mut masonry::core::EventCtx<'_>,
        _props: &mut masonry::core::PropertiesMut<'_>,
        event: &masonry::core::PointerEvent,
    ) {
        if event.is_primary_pointer() {
            ctx.drag_window();
        }
    }
}
