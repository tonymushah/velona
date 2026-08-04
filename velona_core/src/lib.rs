#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod app;
pub mod collection;
pub(crate) mod convert_winit_event;
pub mod error;
pub mod manager;
pub mod render_root;
pub mod task;
// #[cfg(feature = "testing")]
// #[cfg_attr(docsrs, doc(feature = "testing"))]
// pub mod testing;
pub mod utils;
pub mod widget_ref;
pub mod widgets;
pub mod window;
pub mod window_event_handler;
// TODO add `layers` module

#[doc(inline)]
pub use reactive_graph as reactive;

pub use masonry_core;

use masonry_core::core::{NewWidget, Widget};

pub use app::Builder;
pub use manager::Manager;
pub use widgets::NewWidgetExt;
pub use window::builder::WindowBuilder;
pub use window::renderer::WindowRendererFactory;

pub type AnyNewWidget = NewWidget<dyn Widget>;
