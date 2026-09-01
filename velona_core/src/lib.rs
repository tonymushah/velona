#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod app;
pub mod collection;
pub mod error;
pub(crate) mod events;
pub mod manager;
pub mod render_root;
pub mod subsecond;
pub mod task;
pub mod utils;
pub mod widget_ref;
pub mod widgets;
pub mod window;
// TODO add `layers` module

#[doc(inline)]
pub use reactive_graph as reactive;

pub use masonry_core;

use masonry_core::core::{NewWidget, Widget};

pub use app::Builder;
pub use manager::Manager;
pub use widgets::{NewWidgetBaseExt, NewWidgetExt};
pub use window::builder::WindowBuilder;
pub use window::renderer::WindowRendererFactory;

pub type AnyNewWidget = NewWidget<dyn Widget>;
