use std::sync::Arc;

use velona::{masonry::widgets::Svg, widgets::svg::usvg};

pub const X_ICON: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/svgs/x.svg"));

pub const MAXIMIZE_ICON: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/svgs/maximize.svg"));

// const MINIMIZE_ICON: &[u8] = include_bytes!("../svgs/minimize.svg");

pub const MINUS_ICON: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/svgs/minus.svg"));

pub const PLUS_ICON: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/svgs/plus.svg"));

pub const STAR_ICON: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/svgs/star.svg"));

pub fn build_static_svg(data: &'static [u8]) -> Svg {
    Svg::new(Arc::new(
        usvg::Tree::from_data(data, &usvg::Options::default()).unwrap(),
    ))
}
