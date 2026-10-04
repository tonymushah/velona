#![cfg_attr(docsrs, feature(doc_cfg))]

#[doc(inline)]
pub use velona_core::app;
#[doc(inline)]
pub use velona_core::collection;
#[doc(inline)]
pub use velona_core::error;
#[doc(inline)]
pub use velona_core::manager;
#[doc(inline)]
pub use velona_core::render_root;
#[doc(inline)]
pub use velona_core::task;
#[doc(inline)]
pub use velona_masonry_widgets::components;
pub mod utils;
#[doc(inline)]
pub use velona_core::widget_ref;
#[doc(inline)]
pub use velona_core::window;
use velona_renderer::WindowRenderer;
#[doc(inline)]
pub use velona_scoped_styling as scoped_styling;
pub mod widgets;
#[doc(inline)]
#[cfg(feature = "subsecond")]
#[cfg_attr(docsrs, doc(feature = "subsecond"))]
pub use velona_core::subsecond;
// TODO add `layers` module

#[doc(inline)]
pub use velona_core::reactive;

pub use masonry;

use masonry::core::{NewWidget, Widget};

pub use app::Builder;
pub use manager::Manager;
pub use widgets::NewWidgetExt;
pub use window::WindowBuilder;
pub use window::WindowRendererFactory;
use winit::event_loop;
use winit::event_loop::EventLoop;

pub type AnyNewWidget = NewWidget<dyn Widget>;

pub trait VelonaAppExt {
    fn run_in_event_loop(self, event_loop: EventLoop) -> Result<(), winit::error::EventLoopError>;
    fn run(self) -> Result<(), winit::error::EventLoopError>
    where
        Self: Sized,
    {
        self.run_in_event_loop(event_loop::EventLoopBuilder::default().build()?)
    }
}

impl<V> VelonaAppExt for velona_core::app::App<V>
where
    V: WindowRenderer + 'static,
{
    fn run_in_event_loop(self, event_loop: EventLoop) -> Result<(), winit::error::EventLoopError> {
        event_loop.run_app(self)
    }
}
