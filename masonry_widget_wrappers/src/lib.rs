use std::{
    any::{Any, TypeId},
    fmt::Debug,
    ptr,
};

use masonry_core::{
    accesskit::{Node, Role},
    core::{
        AccessCtx, AccessEvent, ActionCtx, ChildrenIds, ComposeCtx, CursorIcon, ErasedAction,
        EventCtx, Layer, LayoutCtx, MeasureCtx, NewWidget, PaintCtx, PointerEvent, PropertiesMut,
        PropertiesRef, QueryCtx, RegisterCtx, TextEvent, Update, UpdateCtx, Widget, WidgetId,
        WidgetMut, WidgetRef,
    },
    imaging::Painter,
    kurbo::{Axis, Point, Size},
    layout::{LenReq, Length},
};
use tracing::Span;

// TODO document this
pub trait WidgetWrapper {
    type Action: Any + Debug
    where
        Self: Sized;

    fn inner(&self) -> &dyn Widget;
    fn inner_mut(&mut self) -> &mut dyn Widget;

    fn on_pointer_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        props: &mut PropertiesMut<'_>,
        event: &PointerEvent,
    ) {
        self.inner_mut().on_pointer_event(ctx, props, event);
    }

    fn on_text_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        props: &mut PropertiesMut<'_>,
        event: &TextEvent,
    ) {
        self.inner_mut().on_text_event(ctx, props, event);
    }

    fn on_access_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        props: &mut PropertiesMut<'_>,
        event: &AccessEvent,
    ) {
        self.inner_mut().on_access_event(ctx, props, event);
    }

    fn on_anim_frame(
        &mut self,
        ctx: &mut UpdateCtx<'_>,
        props: &mut PropertiesMut<'_>,
        interval: u64,
    ) {
        self.inner_mut().on_anim_frame(ctx, props, interval);
    }

    fn on_action(
        &mut self,
        ctx: &mut ActionCtx<'_>,
        props: &mut PropertiesMut<'_>,
        action: &ErasedAction,
        source: WidgetId,
    ) {
        self.inner_mut().on_action(ctx, props, action, source);
    }

    fn register_children(&mut self, ctx: &mut RegisterCtx<'_>) {
        self.inner_mut().register_children(ctx);
    }

    fn update(&mut self, ctx: &mut UpdateCtx<'_>, props: &mut PropertiesMut<'_>, event: &Update) {
        self.inner_mut().update(ctx, props, event);
    }

    fn property_changed(&mut self, ctx: &mut UpdateCtx<'_>, property_type: TypeId) {
        self.inner_mut().property_changed(ctx, property_type);
    }

    fn measure(
        &mut self,
        ctx: &mut MeasureCtx<'_>,
        props: &PropertiesRef<'_>,
        axis: Axis,
        len_req: LenReq,
        cross_length: Option<Length>,
    ) -> Length {
        self.inner_mut()
            .measure(ctx, props, axis, len_req, cross_length)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx<'_>, props: &PropertiesRef<'_>, size: Size) {
        self.inner_mut().layout(ctx, props, size);
    }

    fn compose(&mut self, ctx: &mut ComposeCtx<'_>) {
        self.inner_mut().compose(ctx);
    }

    fn pre_paint(
        &mut self,
        ctx: &mut PaintCtx<'_>,
        props: &PropertiesRef<'_>,
        painter: &mut Painter<'_>,
    ) {
        self.inner_mut().pre_paint(ctx, props, painter);
    }

    fn paint(
        &mut self,
        ctx: &mut PaintCtx<'_>,
        props: &PropertiesRef<'_>,
        painter: &mut Painter<'_>,
    ) {
        self.inner_mut().paint(ctx, props, painter);
    }

    fn post_paint(
        &mut self,
        ctx: &mut PaintCtx<'_>,
        props: &PropertiesRef<'_>,
        painter: &mut Painter<'_>,
    ) {
        self.inner_mut().paint(ctx, props, painter);
    }

    fn accessibility_role(&self) -> Role {
        self.inner().accessibility_role()
    }

    fn accessibility(
        &mut self,
        ctx: &mut AccessCtx<'_>,
        props: &PropertiesRef<'_>,
        node: &mut Node,
    ) {
        self.inner_mut().accessibility(ctx, props, node);
    }

    fn children_ids(&self) -> ChildrenIds {
        self.inner().children_ids()
    }

    fn as_layer(&mut self) -> Option<&mut dyn Layer> {
        self.inner_mut().as_layer()
    }

    fn accepts_pointer_interaction(&self) -> bool {
        self.inner().accepts_pointer_interaction()
    }

    fn propagates_pointer_interaction(&self) -> bool {
        self.inner().propagates_pointer_interaction()
    }

    fn accepts_focus(&self) -> bool {
        self.inner().accepts_focus()
    }

    fn accepts_text_input(&self) -> bool {
        self.inner().accepts_text_input()
    }

    fn make_trace_span(&self, id: WidgetId) -> Span;

    fn get_debug_text(&self) -> Option<String> {
        self.inner().get_debug_text()
    }

    fn get_cursor(&self, ctx: &QueryCtx<'_>, pos: Point) -> CursorIcon {
        self.inner().get_cursor(ctx, pos)
    }

    fn find_widget_under_pointer<'c>(
        &'c self,
        ctx: QueryCtx<'c>,
        pos: Point,
    ) -> Option<WidgetRef<'c, dyn Widget>> {
        self.inner().find_widget_under_pointer(ctx, pos)
    }
    fn prepare(self) -> Wrapper<Self>
    where
        Self: Sized,
    {
        Wrapper::new(self)
    }
    fn into_new_widget(self) -> NewWidget<Wrapper<Self>>
    where
        Self: Sized + 'static,
    {
        self.prepare().prepare()
    }
}

#[non_exhaustive]
pub struct Wrapper<W: ?Sized> {
    pub wrapper: Box<W>,
}

impl<W> Wrapper<W> {
    pub fn new(wrapper: W) -> Wrapper<W> {
        Wrapper {
            wrapper: Box::new(wrapper),
        }
    }
}

impl<W> Wrapper<W>
where
    W: WidgetWrapper + 'static,
{
    pub fn use_inner<U, R>(this: &mut WidgetMut<'_, Self>, to_use_fn: U) -> R
    where
        U: FnOnce(&mut WidgetMut<'_, dyn Widget>) -> R,
    {
        let inner_mut = ptr::from_mut(this.widget.wrapper.inner_mut());
        let mut ref_mut = this.downcast::<dyn Widget>();
        // SAFETY: This is guaranties to always work since `inner_mut` doesn't escape the current scope
        ref_mut.widget = unsafe { inner_mut.as_mut_unchecked() };
        to_use_fn(&mut ref_mut)
    }
}

impl<W> Widget for Wrapper<W>
where
    W: WidgetWrapper + 'static,
{
    type Action = W::Action;
    fn on_pointer_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        props: &mut PropertiesMut<'_>,
        event: &PointerEvent,
    ) {
        self.wrapper.on_pointer_event(ctx, props, event);
    }

    fn on_text_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        props: &mut PropertiesMut<'_>,
        event: &TextEvent,
    ) {
        self.wrapper.on_text_event(ctx, props, event);
    }

    fn on_access_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        props: &mut PropertiesMut<'_>,
        event: &AccessEvent,
    ) {
        self.wrapper.on_access_event(ctx, props, event);
    }

    fn on_anim_frame(
        &mut self,
        ctx: &mut UpdateCtx<'_>,
        props: &mut PropertiesMut<'_>,
        interval: u64,
    ) {
        self.wrapper.on_anim_frame(ctx, props, interval);
    }

    fn on_action(
        &mut self,
        ctx: &mut ActionCtx<'_>,
        props: &mut PropertiesMut<'_>,
        action: &ErasedAction,
        source: WidgetId,
    ) {
        self.wrapper.on_action(ctx, props, action, source);
    }

    fn update(&mut self, ctx: &mut UpdateCtx<'_>, props: &mut PropertiesMut<'_>, event: &Update) {
        self.wrapper.update(ctx, props, event);
    }

    fn property_changed(&mut self, ctx: &mut UpdateCtx<'_>, property_type: TypeId) {
        self.wrapper.property_changed(ctx, property_type);
    }

    fn compose(&mut self, ctx: &mut ComposeCtx<'_>) {
        self.wrapper.compose(ctx);
    }

    fn pre_paint(
        &mut self,
        ctx: &mut PaintCtx<'_>,
        props: &PropertiesRef<'_>,
        painter: &mut Painter<'_>,
    ) {
        self.wrapper.pre_paint(ctx, props, painter);
    }

    fn post_paint(
        &mut self,
        ctx: &mut PaintCtx<'_>,
        props: &PropertiesRef<'_>,
        painter: &mut Painter<'_>,
    ) {
        self.wrapper.post_paint(ctx, props, painter);
    }

    fn as_layer(&mut self) -> Option<&mut dyn Layer> {
        self.wrapper.as_layer()
    }

    fn accepts_pointer_interaction(&self) -> bool {
        self.wrapper.accepts_pointer_interaction()
    }

    fn propagates_pointer_interaction(&self) -> bool {
        self.wrapper.propagates_pointer_interaction()
    }

    fn accepts_focus(&self) -> bool {
        self.wrapper.accepts_focus()
    }

    fn accepts_text_input(&self) -> bool {
        self.wrapper.accepts_text_input()
    }

    fn make_trace_span(&self, id: WidgetId) -> Span {
        self.wrapper.make_trace_span(id)
    }

    fn get_debug_text(&self) -> Option<String> {
        self.wrapper.get_debug_text()
    }

    fn get_cursor(&self, ctx: &QueryCtx<'_>, pos: Point) -> CursorIcon {
        self.wrapper.get_cursor(ctx, pos)
    }

    fn find_widget_under_pointer<'c>(
        &'c self,
        ctx: QueryCtx<'c>,
        pos: Point,
    ) -> Option<WidgetRef<'c, dyn Widget>> {
        self.wrapper.find_widget_under_pointer(ctx, pos)
    }

    fn register_children(&mut self, ctx: &mut RegisterCtx<'_>) {
        self.wrapper.register_children(ctx);
    }

    fn measure(
        &mut self,
        ctx: &mut MeasureCtx<'_>,
        props: &PropertiesRef<'_>,
        axis: Axis,
        len_req: LenReq,
        cross_length: Option<Length>,
    ) -> Length {
        self.wrapper
            .measure(ctx, props, axis, len_req, cross_length)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx<'_>, props: &PropertiesRef<'_>, size: Size) {
        self.wrapper.layout(ctx, props, size);
    }

    fn paint(
        &mut self,
        ctx: &mut PaintCtx<'_>,
        props: &PropertiesRef<'_>,
        painter: &mut Painter<'_>,
    ) {
        self.wrapper.paint(ctx, props, painter);
    }

    fn accessibility_role(&self) -> Role {
        self.wrapper.accessibility_role()
    }

    fn accessibility(
        &mut self,
        ctx: &mut AccessCtx<'_>,
        props: &PropertiesRef<'_>,
        node: &mut Node,
    ) {
        self.wrapper.accessibility(ctx, props, node);
    }

    fn children_ids(&self) -> ChildrenIds {
        self.wrapper.children_ids()
    }
}

#[cfg(test)]
mod tests {
    use masonry_core::core::{DefaultProperties, NoAction};
    use masonry_raw_box::RawBox;
    use tracing::trace_span;

    use super::*;

    struct SomeBoxWrapper(RawBox);
    impl WidgetWrapper for SomeBoxWrapper {
        type Action
            = NoAction
        where
            Self: Sized;
        fn inner(&self) -> &dyn Widget {
            &self.0
        }
        fn inner_mut(&mut self) -> &mut dyn Widget {
            &mut self.0
        }

        fn make_trace_span(&self, id: WidgetId) -> Span {
            trace_span!("SomeBoxWrapper", id = id.trace())
        }
    }

    #[test]
    fn test_casting() {
        let mut testing = masonry_testing::TestHarness::create(
            DefaultProperties::new(),
            SomeBoxWrapper(RawBox::empty()).into_new_widget(),
        );
        // Double testing cause why not :)
        testing.edit_root_widget(|mut root| {
            Wrapper::use_inner(&mut root, |maybe_raw_box| {
                let mut raw_box = maybe_raw_box.downcast::<RawBox>();
                assert!(
                    RawBox::child_mut(&mut raw_box).is_none(),
                    "The RawBox shouldn't have no child"
                );
                RawBox::set_child(&mut raw_box, RawBox::empty().prepare());
            })
        });

        testing.edit_root_widget(|mut root| {
            Wrapper::use_inner(&mut root, |maybe_raw_box| {
                let mut raw_box = maybe_raw_box.downcast::<RawBox>();
                assert!(
                    RawBox::child_mut(&mut raw_box).is_some(),
                    "The RawBox should have a child now"
                )
            })
        });
    }
}
