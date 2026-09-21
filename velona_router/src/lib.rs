// current path, query, hash, history
pub(crate) mod location;
// route patterns and URL matching
pub(crate) mod matcher;
// route definitions and nesting
pub(crate) mod route_tree;
// push, replace, back, forward
pub(crate) mod navigation;
// active matches and rendering
pub(crate) mod runtime;
// Router, Routes, Route, Outlet, Link
mod builder;
pub mod components;

pub use builder::{Route, Router};
pub use navigation::NavigationController;
pub use runtime::use_navigation_controller;
pub use runtime::use_params;
