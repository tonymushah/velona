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
pub(crate) mod components;
