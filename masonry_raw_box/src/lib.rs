// Copyright 2026 the Velona Authors
// SPDX-License-Identifier: Apache-2.0

use std::any::TypeId;

use accesskit::{Node, Role};
use masonry_core::accesskit;
use tracing::{Span, trace_span};

use masonry_core::core::{
    AccessCtx, ChildrenIds, LayoutCtx, MeasureCtx, NewWidget, NoAction, PaintCtx, PropertiesRef,
    RegisterCtx, UpdateCtx, Widget, WidgetId, WidgetMut, WidgetPod,
};
use masonry_core::imaging::Painter;
use masonry_core::kurbo::{Axis, Point, Size};
use masonry_core::layout::{LayoutSize, LenReq, Length};

/// A very simple and bare bone box.
///
/// When the children is nothing, it renders nothing _not even padding or background_.
/// When the children is set, it renders it only renders the children of it.
///
/// _Most of you might never need this, but useful for rendering frameworks root widgets?_.
pub struct RawBox {
    child: Option<WidgetPod<dyn Widget>>,
}

// --- MARK: BUILDERS
impl RawBox {
    /// Creates container with child, and both width and height unset.
    pub fn new(child: NewWidget<impl Widget + ?Sized>) -> Self {
        Self {
            child: Some(child.erased().to_pod()),
        }
    }

    /// Creates container without a child, and both width and height unset.
    ///
    /// In this state it will render no content but will still render its border and padding.
    #[doc(alias = "null")]
    pub fn empty() -> Self {
        Self { child: None }
    }
}

// --- MARK: WIDGETMUT
impl RawBox {
    /// Replaces the child widget with a new one.
    pub fn set_child(this: &mut WidgetMut<'_, Self>, child: NewWidget<impl Widget + ?Sized>) {
        if let Some(child) = this.widget.child.take() {
            this.ctx.remove_child(child);
        }
        this.widget.child = Some(child.erased().to_pod());
        this.ctx.children_changed();
    }

    /// Removes the child widget.
    ///
    /// (If this widget has no child, this method does nothing.)
    pub fn remove_child(this: &mut WidgetMut<'_, Self>) {
        if let Some(child) = this.widget.child.take() {
            this.ctx.remove_child(child);
        }
    }

    /// Returns mutable reference to the child widget, if any.
    pub fn child_mut<'t>(this: &'t mut WidgetMut<'_, Self>) -> Option<WidgetMut<'t, dyn Widget>> {
        let child = this.widget.child.as_mut()?;
        Some(this.ctx.get_mut(child))
    }
}

// --- MARK: IMPL WIDGET
impl Widget for RawBox {
    type Action = NoAction;

    fn register_children(&mut self, ctx: &mut RegisterCtx<'_>) {
        if let Some(ref mut child) = self.child {
            ctx.register_child(child);
        }
    }

    fn property_changed(&mut self, _ctx: &mut UpdateCtx<'_>, _property_type: TypeId) {}

    fn measure(
        &mut self,
        ctx: &mut MeasureCtx<'_>,
        _props: &PropertiesRef<'_>,
        axis: Axis,
        len_req: LenReq,
        cross_length: Option<Length>,
    ) -> Length {
        if let Some(child) = self.child.as_mut() {
            let cross = axis.cross();

            let auto_length = len_req.into();
            let context_size = LayoutSize::maybe(cross, cross_length);

            ctx.compute_length(child, auto_length, context_size, axis, cross_length)
        } else {
            Length::ZERO
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx<'_>, _props: &PropertiesRef<'_>, size: Size) {
        let Some(child) = self.child.as_mut() else {
            // No child, so no layout work beyond resetting baselines
            ctx.clear_baselines();
            return;
        };

        ctx.run_layout(child, size);
        ctx.place_child(child, Point::ORIGIN);
        ctx.derive_baselines(child);
    }

    fn paint(
        &mut self,
        _ctx: &mut PaintCtx<'_>,
        _props: &PropertiesRef<'_>,
        _painter: &mut Painter<'_>,
    ) {
    }

    fn accessibility_role(&self) -> Role {
        Role::GenericContainer
    }

    fn accessibility(
        &mut self,
        _ctx: &mut AccessCtx<'_>,
        _props: &PropertiesRef<'_>,
        _node: &mut Node,
    ) {
    }

    fn children_ids(&self) -> ChildrenIds {
        if let Some(child) = &self.child {
            ChildrenIds::from_slice(&[child.id()])
        } else {
            ChildrenIds::from_slice(&[])
        }
    }

    fn make_trace_span(&self, id: WidgetId) -> Span {
        trace_span!("RawBox", id = id.trace())
    }
}

// --- MARK: TESTS
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::core::PropertySet;
//     use crate::layout::{AsUnit, UnitPoint};
//     use crate::palette;
//     use crate::properties::types::Gradient;
//     use crate::properties::{Background, BorderColor, CornerRadius};
//     use crate::testing::{TestHarness, assert_failing_render_snapshot, assert_render_snapshot};
//     use crate::theme::test_property_set;
//     use crate::widgets::Label;

//     // TODO - Add WidgetMut tests

//     #[test]
//     fn empty_box() {
//         let mut box_props = PropertySet::new();
//         box_props.insert(BorderColor::new(palette::css::BLUE));
//         box_props.insert(BorderWidth::all(5.px()));
//         box_props.insert(CornerRadius::all(5.px()));

//         let widget = RawBox::empty()
//             .width(20.px())
//             .height(20.px())
//             .prepare()
//             .with_props(box_props);

//         let mut harness = TestHarness::create_with_size(test_property_set(), widget, (100, 100));

//         assert_render_snapshot!(harness, "sized_box_empty_box");
//     }

//     #[test]
//     fn label_box_no_size() {
//         let mut box_props = PropertySet::new();
//         box_props.insert(BorderColor::new(palette::css::BLUE));
//         box_props.insert(BorderWidth::all(5.px()));
//         box_props.insert(CornerRadius::all(5.px()));

//         let widget = RawBox::new(Label::new("hello").prepare())
//             .prepare()
//             .with_props(box_props);

//         let mut harness = TestHarness::create_with_size(test_property_set(), widget, (100, 100));

//         assert_render_snapshot!(harness, "sized_box_label_box_no_size");
//     }

//     #[test]
//     fn label_box_with_size() {
//         let mut box_props = PropertySet::new();
//         box_props.insert(BorderColor::new(palette::css::BLUE));
//         box_props.insert(BorderWidth::all(5.px()));
//         box_props.insert(CornerRadius::all(5.px()));

//         let widget = RawBox::new(Label::new("hello").prepare())
//             .width(20.px())
//             .height(20.px())
//             .prepare()
//             .with_props(box_props);

//         let mut harness = TestHarness::create_with_size(test_property_set(), widget, (100, 100));

//         assert_render_snapshot!(harness, "sized_box_label_box_with_size");
//     }

//     #[test]
//     fn label_box_with_padding() {
//         let mut box_props = PropertySet::new();
//         box_props.insert(BorderColor::new(palette::css::BLUE));
//         box_props.insert(BorderWidth::all(5.px()));
//         box_props.insert(CornerRadius::all(5.px()));
//         box_props.insert(Padding::from_vh(15.px(), 10.px()));

//         let widget = RawBox::new(Label::new("hello").prepare())
//             .prepare()
//             .with_props(box_props);

//         let mut harness = TestHarness::create_with_size(test_property_set(), widget, (100, 100));

//         assert_render_snapshot!(harness, "sized_box_label_box_with_padding");
//     }

//     #[test]
//     fn label_box_with_solid_background() {
//         let mut box_props = PropertySet::new();
//         box_props.insert(Background::Color(palette::css::PLUM));

//         let widget = RawBox::new(Label::new("hello").prepare())
//             .width(20.px())
//             .height(20.px())
//             .prepare()
//             .with_props(box_props);

//         let mut harness = TestHarness::create_with_size(test_property_set(), widget, (100, 100));

//         assert_render_snapshot!(harness, "sized_box_label_box_with_solid_background");
//     }

//     #[test]
//     fn empty_box_with_gradient_background() {
//         let mut box_props = PropertySet::new();

//         let gradient = Gradient::new_linear(2.0).with_stops([
//             palette::css::WHITE,
//             palette::css::BLACK,
//             palette::css::RED,
//             palette::css::GREEN,
//             palette::css::WHITE,
//         ]);
//         box_props.insert(Background::Gradient(gradient));
//         box_props.insert(BorderColor::new(palette::css::LIGHT_SKY_BLUE));
//         box_props.insert(BorderWidth::all(5.px()));
//         box_props.insert(CornerRadius::all(10.px()));

//         let widget = RawBox::empty()
//             .width(20.px())
//             .height(20.px())
//             .prepare()
//             .with_props(box_props);

//         let mut harness = TestHarness::create_with_size(test_property_set(), widget, (100, 100));

//         assert_render_snapshot!(harness, "sized_box_empty_box_with_gradient_background");
//     }

//     #[test]
//     fn radial_gradient_background() {
//         let mut box_props = PropertySet::new();

//         let gradient = Gradient::new_radial(UnitPoint::CENTER).with_stops([
//             palette::css::WHITE,
//             palette::css::BLACK,
//             palette::css::RED,
//             palette::css::GREEN,
//             palette::css::WHITE,
//         ]);
//         box_props.insert(Background::Gradient(gradient));
//         box_props.insert(BorderColor::new(palette::css::LIGHT_SKY_BLUE));
//         box_props.insert(BorderWidth::all(5.px()));
//         box_props.insert(CornerRadius::all(10.px()));

//         let widget = RawBox::empty()
//             .width(20.px())
//             .height(20.px())
//             .prepare()
//             .with_props(box_props);

//         let mut harness = TestHarness::create_with_size(test_property_set(), widget, (100, 100));

//         assert_render_snapshot!(harness, "sized_box_radial_gradient_background");
//     }

//     #[test]
//     fn sweep_gradient_background() {
//         let mut box_props = PropertySet::new();

//         let gradient = Gradient::new_full_sweep(UnitPoint::CENTER, 0.).with_stops([
//             palette::css::WHITE,
//             palette::css::BLACK,
//             palette::css::RED,
//             palette::css::GREEN,
//             palette::css::WHITE,
//         ]);
//         box_props.insert(Background::Gradient(gradient));
//         box_props.insert(BorderColor::new(palette::css::LIGHT_SKY_BLUE));
//         box_props.insert(BorderWidth::all(5.px()));
//         box_props.insert(CornerRadius::all(10.px()));

//         let widget = RawBox::empty()
//             .width(20.px())
//             .height(20.px())
//             .prepare()
//             .with_props(box_props);

//         let mut harness = TestHarness::create_with_size(test_property_set(), widget, (100, 100));

//         assert_render_snapshot!(harness, "sized_box_sweep_gradient_background");
//     }

//     #[test]
//     fn label_box_with_padding_and_background() {
//         let mut box_props = PropertySet::new();
//         box_props.insert(Background::Color(palette::css::PLUM));
//         box_props.insert(BorderColor::new(palette::css::LIGHT_SKY_BLUE));
//         box_props.insert(BorderWidth::all(5.px()));
//         box_props.insert(Padding::all(25.px()));

//         let widget = RawBox::new(Label::new("hello").prepare())
//             .width(20.px())
//             .height(20.px())
//             .prepare()
//             .with_props(box_props);

//         let mut harness = TestHarness::create_with_size(test_property_set(), widget, (100, 100));

//         assert_render_snapshot!(harness, "sized_box_label_box_with_background_and_padding");
//     }

//     // --- MARK: INVALID SCREENSHOT TESTS

//     #[test]
//     fn invalid_screenshot() {
//         // Copy-pasted from empty_box
//         let mut box_props = PropertySet::new();
//         box_props.insert(BorderColor::new(palette::css::BLUE));
//         box_props.insert(BorderWidth::all(5.px()));
//         box_props.insert(CornerRadius::all(5.px()));

//         // This is the difference
//         box_props.insert(BorderWidth::all(5.2.px()));

//         let widget = RawBox::empty()
//             .width(20.px())
//             .height(20.px())
//             .prepare()
//             .with_props(box_props);

//         let mut harness = TestHarness::create_with_size(test_property_set(), widget, (100, 100));

//         assert_failing_render_snapshot!(harness, "sized_box_empty_box");
//     }
// }
