use velona_core::AnyNewWidget;

use crate::route_tree::{RouteId, RouteNode, RouteSegment, RouteTree};

#[derive(Debug, Default)]
pub struct Router {
    pub(crate) tree: RouteTree,
}

impl Router {
    pub fn route(mut self, route: Route) -> Self {
        let mut stack = vec![route];

        while let Some(route) = stack.pop() {
            if let Some(parent_id) = route.parent_id {
                let Some(mut parent_node) = self.tree.find_mut(parent_id.0) else {
                    unreachable!("The parent child node should be always available")
                };
                parent_node.children.insert(route.id.0, route.node);
            } else {
                self.tree.roots_mut().insert(route.id.0, route.node);
            }
            let mut childs = route.childs;
            stack.append(&mut childs);
        }

        self
    }
}

pub struct Route {
    node: RouteNode,
    id: RouteId,
    childs: Vec<Route>,
    parent_id: Option<RouteId>,
}

impl Route {
    pub fn root<V>(view: V) -> Self
    where
        V: Fn() -> AnyNewWidget + Send + 'static,
    {
        Self {
            node: RouteNode {
                segment: RouteSegment::Root,
                view: Box::new(view),
            },
            id: RouteId::next(),
            childs: Default::default(),
            parent_id: None,
        }
    }
    pub fn static_<V>(path: &str, view: V) -> Self
    where
        V: Fn() -> AnyNewWidget + Send + 'static,
    {
        Self {
            node: RouteNode {
                segment: RouteSegment::Static(path.into()),
                view: Box::new(view),
            },
            id: RouteId::next(),
            childs: Default::default(),
            parent_id: None,
        }
    }
    pub fn params<V>(name: &str, view: V) -> Self
    where
        V: Fn() -> AnyNewWidget + Send + 'static,
    {
        Self {
            node: RouteNode {
                segment: RouteSegment::Param { name: name.into() },
                view: Box::new(view),
            },
            id: RouteId::next(),
            childs: Default::default(),
            parent_id: None,
        }
    }
    pub fn wildcard<V>(name: &str, view: V) -> Self
    where
        V: Fn() -> AnyNewWidget + Send + 'static,
    {
        Self {
            node: RouteNode {
                segment: RouteSegment::Wildcard { name: name.into() },
                view: Box::new(view),
            },
            id: RouteId::next(),
            childs: Default::default(),
            parent_id: None,
        }
    }
}

impl Route {
    pub fn child(mut self, mut child: Route) -> Self {
        child.parent_id = Some(self.id);
        self.childs.push(child);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn my_view() -> AnyNewWidget {
        todo!()
    }

    #[test]
    fn test_build_router() {
        let router = Router::default().route(
            Route::root(my_view)
                .child(Route::static_("posts", my_view))
                .child(Route::static_("users", my_view).child(Route::params("something", my_view)))
                .child(Route::wildcard("any", my_view)),
        );
        assert_eq!(router.tree.len(), 5);
        assert_eq!(router.tree.root_ids().count(), 1);
        assert_eq!(
            router
                .tree
                .roots()
                .item(router.tree.root_ids().next().unwrap())
                .unwrap()
                .child_ids()
                .into_iter()
                .count(),
            3
        );
        let child_routes = router
            .tree
            .roots()
            .item(router.tree.root_ids().next().unwrap())
            .unwrap()
            .child_ids()
            .into_iter()
            .flat_map(|id| router.tree.find(id))
            .collect::<Box<[_]>>();

        assert!(child_routes.iter().any(|node| {
            if let RouteSegment::Static(name) = &node.item.segment {
                name == "posts"
            } else {
                false
            }
        }));
        assert!(child_routes.iter().any(|node| {
            if let RouteSegment::Static(name) = &node.item.segment {
                name == "users"
            } else {
                false
            }
        }));
        assert!(child_routes.iter().any(|node| {
            if let RouteSegment::Wildcard { name } = &node.item.segment {
                name == "any"
            } else {
                false
            }
        }))
    }
}
