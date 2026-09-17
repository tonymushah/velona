mod when_empty;

use std::collections::HashMap;

use tree_arena::ArenaRef;

use crate::{
    location::Location,
    route_tree::{RouteId, RouteNode, RouteSegment, RouteTree},
};

pub type Params = HashMap<String, String>;

pub struct RouteMatch {
    pub route_id: RouteId,
    pub params: Params,
}

pub struct MatchedRoutes {
    pub matches: Box<[RouteMatch]>,
}

#[derive(Debug, thiserror::Error)]
pub enum MatchError {
    #[error("The route tree root node should be a `RouteSegment`")]
    InvalidRootTreeNode,
    #[error("a route id can't be zero")]
    ZeroRouteId,
    #[error("The current location doesn't have any path segments")]
    NoPathSegments,
}

fn get_routes_child_id(
    tree: &RouteTree,
    route_id: Option<RouteId>,
) -> Result<Option<Box<[RouteId]>>, MatchError> {
    let ids = if let Some(route_id) = route_id {
        let Some(route) = tree.find(route_id.0) else {
            return Ok(None);
        };
        route
            .child_ids()
            .into_iter()
            .map(|id| {
                Ok::<_, MatchError>(RouteId(id.try_into().map_err(|_| MatchError::ZeroRouteId)?))
            })
            .collect::<Result<Box<[_]>, MatchError>>()?
    } else {
        tree.root_ids()
            .map(|id| {
                Ok::<_, MatchError>(RouteId(id.try_into().map_err(|_| MatchError::ZeroRouteId)?))
            })
            .collect::<Result<Box<[_]>, MatchError>>()?
    };
    if ids.is_empty() {
        Ok(None)
    } else {
        Ok(Some(ids))
    }
}

pub fn matches_routes(tree: &RouteTree, location: &Location) -> Result<MatchedRoutes, MatchError> {
    let mut matches = Vec::<RouteMatch>::with_capacity({
        if let Some(paths) = location.url.path_segments() {
            paths.count()
        } else {
            0
        }
    });

    let mut params = HashMap::<String, String>::new();

    let root_ids = get_routes_child_id(tree, None)?.unwrap();

    if root_ids.is_empty() {
        return Ok(MatchedRoutes {
            matches: matches.into_boxed_slice(),
        });
    }

    if let Some(paths) = location.url.path_segments() {
        let segments = paths.collect::<Box<[_]>>();

        'segments: for (index, path) in segments.iter().enumerate() {
            'lookup: while let Some(route_ids) =
                get_routes_child_id(tree, matches.iter().next_back().map(|d| d.route_id))?
            {
                let mut routes_node = route_ids
                    .iter()
                    .flat_map(|id| Some((id, tree.find(id.0)?)))
                    .collect::<Box<[_]>>();
                routes_node.sort_by_key(|(_, e)| e.item.segment.kind());

                for (id, node) in routes_node {
                    match &node.item.segment {
                        RouteSegment::Root => {
                            matches.push(RouteMatch {
                                route_id: *id,
                                params: params.clone(),
                            });
                            continue 'lookup;
                        }
                        RouteSegment::Static(spath) => {
                            if spath == *path {
                                matches.push(RouteMatch {
                                    route_id: *id,
                                    params: params.clone(),
                                });
                                continue 'segments;
                            }
                        }
                        RouteSegment::Param { name } => {
                            params.insert(name.clone(), path.to_string());
                            matches.push(RouteMatch {
                                route_id: *id,
                                params: params.clone(),
                            });
                            continue 'segments;
                        }
                        RouteSegment::Wildcard { name } => {
                            params.insert(
                                name.clone(),
                                segments.split_at(index).1.iter().fold(
                                    String::new(),
                                    |mut string, path| {
                                        if !path.is_empty() {
                                            string.push('/');
                                            string.push_str(path);
                                        }
                                        string
                                    },
                                ),
                            );
                            matches.push(RouteMatch {
                                route_id: *id,
                                params: params.clone(),
                            });
                            break 'segments;
                        }
                    }
                }
                break;
            }
        }
    } else {
        return Err(MatchError::NoPathSegments);
    }

    Ok(MatchedRoutes {
        matches: matches.into_boxed_slice(),
    })
}

#[cfg(test)]
mod tests {
    use velona_core::AnyNewWidget;

    use crate::{Route, Router, route_tree::RouteSegmentKind};

    use super::*;

    fn view() -> AnyNewWidget {
        todo!()
    }

    #[test]
    fn test_root_matching() {
        let router = Router::default().route(Route::root(view).child(Route::static_("aaa", view)));

        let location = Location::default();
        let matches = matches_routes(&router.tree, &location).unwrap();

        for (index, node) in matches.matches.iter().enumerate() {
            match index {
                0 => {
                    let node = router.tree.find(node.route_id.0).unwrap();
                    assert_eq!(node.item.segment.kind(), RouteSegmentKind::Root);
                }
                _ => {
                    unreachable!()
                }
            }
        }
    }
    #[test]
    fn test_root_matching_nested() {
        let router = Router::default().route(
            Route::root(view)
                .child(Route::static_("aaa", view))
                .child(Route::root(view).child(Route::root(view))),
        );

        let location = Location::default();

        let matches = matches_routes(&router.tree, &location).unwrap();

        assert_eq!(matches.matches.len(), 3);

        for (index, node) in matches.matches.iter().enumerate() {
            match index {
                0..=2 => {
                    let node = router.tree.find(node.route_id.0).unwrap();
                    assert_eq!(node.item.segment.kind(), RouteSegmentKind::Root);
                }
                _ => {
                    unreachable!()
                }
            }
        }
    }
    #[test]
    fn test_root_matching_nested_wild_card() {
        let router =
            Router::default().route(Route::root(view).child(Route::static_("aaa", view)).child(
                Route::root(view).child(Route::root(view).child(Route::wildcard("any", view))),
            ));

        let location = Location::default();
        let matches = matches_routes(&router.tree, &location).unwrap();

        assert_eq!(matches.matches.len(), 4);

        for (index, node) in matches.matches.iter().enumerate() {
            match index {
                0..=2 => {
                    let node = router.tree.find(node.route_id.0).unwrap();
                    assert_eq!(node.item.segment.kind(), RouteSegmentKind::Root);
                }
                3 => {
                    let tnode = router.tree.find(node.route_id.0).unwrap();
                    assert_eq!(tnode.item.segment.kind(), RouteSegmentKind::Wildcard);
                    assert_eq!(node.params.get("any").map(|a| a.as_str()), Some(""));
                }
                _ => {
                    unreachable!()
                }
            }
        }
    }

    #[test]
    fn test_matching_nested_static() {
        let router = Router::default()
            .route(
                Route::root(view).child(
                    Route::static_("user", view)
                        .child(Route::static_("posts", view))
                        .child(Route::params("id", view))
                        .child(Route::wildcard("any", view)),
                ),
            )
            .route(Route::wildcard("any", view));

        let mut location = Location::default();
        location.goto("users/").unwrap();

        let matches = matches_routes(&router.tree, &location).unwrap();

        assert_eq!(matches.matches.len(), 1);
        for (index, node) in matches.matches.iter().enumerate() {
            match index {
                0 => {
                    let node = router.tree.find(node.route_id.0).unwrap();
                    assert_eq!(node.item.segment.kind(), RouteSegmentKind::Root);
                }
                _ => {
                    unreachable!()
                }
            }
        }
    }

    #[test]
    fn test_matching_nested_static_wildcard() {
        let router = Router::default()
            .route(
                Route::root(view)
                    .child(
                        Route::static_("user", view)
                            .child(Route::static_("posts", view))
                            .child(Route::params("id", view))
                            .child(Route::wildcard("any", view)),
                    )
                    .child(Route::wildcard("any", view)),
            )
            .route(Route::wildcard("any", view));

        let mut location = Location::default();
        location.goto("users/").unwrap();

        let matches = matches_routes(&router.tree, &location).unwrap();

        assert_eq!(matches.matches.len(), 2);
        for (index, node) in matches.matches.iter().enumerate() {
            match index {
                0 => {
                    let node = router.tree.find(node.route_id.0).unwrap();
                    assert_eq!(node.item.segment.kind(), RouteSegmentKind::Root);
                }
                1 => {
                    let tnode = router.tree.find(node.route_id.0).unwrap();
                    assert_eq!(tnode.item.segment.kind(), RouteSegmentKind::Wildcard);
                    assert_eq!(node.params.get("any").map(|a| a.as_str()), Some("/users"));
                }
                _ => {
                    unreachable!()
                }
            }
        }
    }
}
