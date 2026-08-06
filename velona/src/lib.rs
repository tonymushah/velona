#![cfg_attr(docsrs, feature(doc_cfg))]

#[doc(inline)]
pub use velona_core::app;
#[doc(inline)]
pub use velona_core::collection;
pub mod components;
#[doc(inline)]
pub use velona_core::error;
#[doc(inline)]
pub use velona_core::manager;
#[doc(inline)]
pub use velona_core::render_root;
#[doc(inline)]
pub use velona_core::task;
pub mod utils;
#[doc(inline)]
pub use velona_core::widget_ref;
#[doc(inline)]
pub use velona_core::window;
#[doc(inline)]
pub use velona_core::window_event_handler;
pub mod widgets;
// TODO add `layers` module

#[doc(inline)]
pub use velona_core::reactive;

pub use masonry;

use masonry::core::{NewWidget, Widget};

pub use app::Builder;
pub use manager::Manager;
pub use widgets::NewWidgetExt;
pub use window::builder::WindowBuilder;
pub use window::renderer::WindowRendererFactory;

pub type AnyNewWidget = NewWidget<dyn Widget>;
