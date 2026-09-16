pub type RouteView = (); // TODO

pub struct RouteNode {
    pub(crate) segment: RouteSegment,
    pub(crate) children: Vec<RouteNode>,
    pub(crate) view: RouteView,
}

pub enum RouteSegment {
    Root,
    Static(String),
    Param { name: String },
    OptionalParam { name: String },
    Wildcard { name: String },
}
