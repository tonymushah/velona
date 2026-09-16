use std::{
    num::NonZero,
    sync::atomic::{AtomicU64, Ordering},
};

use tree_arena::TreeArena;

pub type RouteView = (); // TODO

pub struct RouteNode {
    pub(crate) segment: RouteSegment,
    pub(crate) view: RouteView,
}

#[derive(Debug)]
pub enum RouteSegment {
    Root,
    Static(String),
    Param { name: String },
    Wildcard { name: String },
}

impl RouteSegment {
    pub fn kind(&self) -> RouteSegmentKind {
        match self {
            RouteSegment::Root => RouteSegmentKind::Root,
            RouteSegment::Static(_) => RouteSegmentKind::Static,
            RouteSegment::Param { name: _ } => RouteSegmentKind::Param,
            RouteSegment::Wildcard { name: _ } => RouteSegmentKind::Wildcard,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RouteSegmentKind {
    Static,
    Param,
    Root,
    Wildcard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RouteId(pub(crate) NonZero<u64>);

impl RouteId {
    fn next() -> RouteId {
        static ROUTE_ID_COUNTER: AtomicU64 = AtomicU64::new(1);
        let id = ROUTE_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self(id.try_into().unwrap())
    }
}

pub type RouteTree = TreeArena<RouteNode>;

#[cfg(test)]
mod tests {
    use crate::route_tree::RouteSegmentKind;

    #[test]
    fn test_segment_tests() {
        assert!(RouteSegmentKind::Root < RouteSegmentKind::Static);
    }

    #[test]
    fn test_segment_kind_test() {
        let mut unsorted = [
            RouteSegmentKind::Wildcard,
            RouteSegmentKind::Param,
            RouteSegmentKind::Root,
            RouteSegmentKind::Static,
        ];
        unsorted.sort();
        assert_eq!(
            unsorted,
            [
                RouteSegmentKind::Static,
                RouteSegmentKind::Param,
                RouteSegmentKind::Root,
                RouteSegmentKind::Wildcard
            ]
        );
    }
}
