#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod app;
pub mod components;
pub(crate) mod convert_winit_event;
pub mod error;
pub mod manager;
pub mod render_root;
pub(crate) mod utils;
pub mod widget_ref;
pub mod widgets;
pub mod window;
pub(crate) mod window_event_handler;

use masonry::core::{NewWidget, Widget};
pub use reactive_graph;

pub use app::Builder;
pub use manager::Manager;
pub use widgets::NewWidgetExt;
pub use window::builder::WindowBuilder;

pub type AnyNewWidget = NewWidget<dyn Widget>;
