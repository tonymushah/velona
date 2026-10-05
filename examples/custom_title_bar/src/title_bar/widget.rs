use std::any::TypeId;

use accesskit::{Node, Role};
use masonry::accesskit;
use masonry_raw_box::RawBox;
use tracing::{Span, trace_span};

use masonry::core::{
    AccessCtx, ChildrenIds, LayoutCtx, MeasureCtx, NewWidget, NoAction, PaintCtx, PropertiesRef,
    RegisterCtx, UpdateCtx, Widget, WidgetId,
};
use masonry::imaging::Painter;
use masonry::kurbo::{Axis, Size};
use masonry::layout::{LenReq, Length};
use velona::masonry;

/// A very simple and bare bone box.
///
/// When the children is nothing, it renders nothing _not even padding or background_.
/// When the children is set, it renders it only renders the children of it.
///
/// _Most of you might never need this, but useful for rendering frameworks root widgets?_.
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

// --- MARK: WIDGETMUT
impl TitleBarContainer {}

// --- MARK: IMPL WIDGET
impl Widget for TitleBarContainer {
    type Action = NoAction;

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

    fn register_children(&mut self, ctx: &mut RegisterCtx<'_>) {
        self.inner.register_children(ctx);
    }

    fn property_changed(&mut self, ctx: &mut UpdateCtx<'_>, property_type: TypeId) {
        self.inner.property_changed(ctx, property_type);
    }

    fn measure(
        &mut self,
        ctx: &mut MeasureCtx<'_>,
        props: &PropertiesRef<'_>,
        axis: Axis,
        len_req: LenReq,
        cross_length: Option<Length>,
    ) -> Length {
        self.inner.measure(ctx, props, axis, len_req, cross_length)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx<'_>, props: &PropertiesRef<'_>, size: Size) {
        self.inner.layout(ctx, props, size);
    }

    fn paint(
        &mut self,
        ctx: &mut PaintCtx<'_>,
        props: &PropertiesRef<'_>,
        painter: &mut Painter<'_>,
    ) {
        self.inner.paint(ctx, props, painter);
    }

    fn accessibility_role(&self) -> Role {
        self.inner.accessibility_role()
    }

    fn accessibility(
        &mut self,
        ctx: &mut AccessCtx<'_>,
        props: &PropertiesRef<'_>,
        node: &mut Node,
    ) {
        self.inner.accessibility(ctx, props, node);
    }

    fn children_ids(&self) -> ChildrenIds {
        self.inner.children_ids()
    }

    fn make_trace_span(&self, id: WidgetId) -> Span {
        trace_span!("TitleBarContainer", id = id.trace())
    }
}
