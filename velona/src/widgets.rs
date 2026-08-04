//! Various [`widgets`](masonry::widgets) extenstions
//!
//! TODO **custom implementation**
//!
//! - [x] [`Align`](masonry::widgets::Align)
//! - [x] [`Badge`](masonry::widgets::Badge)
//! - [x] [`Badged`](masonry_widgets::Badged)
//! - [x] [`Button`](masonry::widgets::Button)
//! - [x] [`Canvas`](masonry::widgets::Canvas)
//! - [x] [`Checkbox`](masonry::widgets::Checkbox) in [`checkbox`]
//! - [x] [`CollapsePanel`](masonry::widgets::CollapsePanel)
//! - [x] [`DisclosureButton`](masonry::widgets::DisclosureButton)
//! - [x] [`Divider`](masonry::widgets::Divider)
//! - [x] [`Flex`](masonry::widgets::Flex)
//! - [x] [`Grid`](masonry::widgets::Grid)
//! - [x] [`Image`](masonry::widgets::Image) in [`image`]
//! - [x] [`IndexedStack`](masonry::widgets::IndexedStack)
//! - [x] [`Label`](masonry::widgets::Label) in [`label`]
//! - [x] [`Pagination`](masonry::widgets::Pagination)
//! - [x] [`Passthrough`](masonry::widgets::Passthrough)
//! - [x] [`Portal`](masonry::widgets::Portal)
//! - [x] [`ProgressBar`](masonry::widgets::ProgressBar)
//! - [x] [`Prose`](masonry::widgets::Prose)
//! - [x] [`Radio`](masonry::widgets::RadioButton)
//! - [x] [`ResizeObserver`](masonry::widgets::ResizeObserver)
//! - [x] [`ScrollBar`](masonry::widgets::ScrollBar)
//! - [x] [`Selector`](masonry::widgets::Selector)
//! - [x] [`SelectorItem`](masonry::widgets::SelectorItem)
//! - [x] [`SizedBox`](masonry::widgets::SizedBox)
//! - [x] [`Slider`](masonry::widgets::Slider)
//! - [x] [`Spinner`](masonry::widgets::Spinner)
//! - [x] [`Split`](masonry::widgets::Split)
//! - [x] [`StepInput`](masonry::widgets::StepInput)
//! - [x] [`Svg`](masonry::widgets::Svg)
//! - [x] [`Switch`](masonry::widgets::Switch)
//! - [x] [`TextArea`](masonry::widgets::TextArea)
//! - [x] [`TextInput`](masonry::widgets::TextInput)
//! - [x] [`VariableLabel`](masonry::widgets::VariableLabel)
//! - [x] [`VirtualScroll`](masonry::widgets::VirtualScroll)
//! - [x] [`ZStack`](masonry::widgets::ZStack)
// TODO add [new](NewWidget) for `New*Ext` widget trait doc comments
// TODO add doc comment for each module
pub mod align;
pub mod badged;
pub mod button;
pub mod canvas;
pub mod checkbox;
pub mod collapse_panel;
pub mod disclosure_button;
pub mod divider;
pub mod flex;
pub mod grid;
pub mod image;
pub mod indexed_stack;
pub mod label;
pub mod pagination;
pub mod portal;
pub mod progress;
pub mod prose;
pub mod radio;
pub mod resize_observer;
pub mod scrollbar;
pub mod selector;
pub mod selector_item;
pub mod sized_box;
pub mod slider;
pub mod split;
pub mod step_input;
pub mod svg;
pub mod switch;
pub mod text_area;
pub mod text_input;
pub mod variable_label;
pub mod virtual_scroll;
pub mod zstack;

#[doc(inline)]
pub use velona_core::widgets::*;
pub use velona_core_child::{
    ReactiveSingleChildExt, ReactiveSingleTypedChildExt, SingleChildWidget, TypedSingleChildWidget,
};

pub use masonry::widgets as masonry_widgets;
