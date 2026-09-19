use std::collections::HashMap;

use crate::{
    location::LocationState,
    route_tree::{RouteId, RouteSegment, RouteTree},
};

pub type Params = HashMap<String, String>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RouteMatch {
    pub route_id: RouteId,
    pub params: Params,
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct MatchedRoutes {
    pub matches: Box<[RouteMatch]>,
}

#[derive(Debug, thiserror::Error)]
pub enum MatchError {
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
        let Some(route) = tree.find(route_id) else {
            return Ok(None);
        };
        route
            .child_ids()
            .into_iter()
            .map(|id| {
                Ok::<_, MatchError>(RouteId::new(
                    id.try_into().map_err(|_| MatchError::ZeroRouteId)?,
                ))
            })
            .collect::<Result<Box<[_]>, MatchError>>()?
    } else {
        tree.root_ids()
            .map(|id| {
                Ok::<_, MatchError>(RouteId::new(
                    id.try_into().map_err(|_| MatchError::ZeroRouteId)?,
                ))
            })
            .collect::<Result<Box<[_]>, MatchError>>()?
    };
    if ids.is_empty() {
        Ok(None)
    } else {
        Ok(Some(ids))
    }
}

pub fn matches_routes(
    tree: &RouteTree,
    location: &LocationState,
) -> Result<MatchedRoutes, MatchError> {
    if let Some(paths) = location.url.path_segments() {
        let mut segments = paths.collect::<Vec<_>>();

        if segments.last().is_some_and(|s| !s.is_empty()) {
            segments.push("");
        }

        let segments = segments.into_boxed_slice();

        let mut matches = Vec::<RouteMatch>::with_capacity(segments.len());

        let mut params = HashMap::<String, String>::new();

        let root_ids = get_routes_child_id(tree, None)?.unwrap();

        if root_ids.is_empty() {
            return Ok(MatchedRoutes {
                matches: matches.into_boxed_slice(),
            });
        }

        'segments: for (index, path) in segments.iter().enumerate() {
            'lookup: while let Some(route_ids) =
                get_routes_child_id(tree, matches.iter().next_back().map(|d| d.route_id))?
            {
                let mut routes_node = route_ids
                    .iter()
                    .flat_map(|id| Some((id, tree.find(*id)?)))
                    .collect::<Box<[_]>>();
                routes_node.sort_by_key(|(_, e)| e.item.segment.kind());

                for (id, node) in routes_node {
                    match &node.item.segment {
                        RouteSegment::Index => {
                            if path.is_empty() {
                                matches.push(RouteMatch {
                                    route_id: *id,
                                    params: params.clone(),
                                });
                                break 'segments;
                            }
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
                        RouteSegment::Layout => {
                            matches.push(RouteMatch {
                                route_id: *id,
                                params: params.clone(),
                            });
                            continue 'lookup;
                        }
                    }
                }
                break;
            }
        }
        Ok(MatchedRoutes {
            matches: matches.into_boxed_slice(),
        })
    } else {
        Err(MatchError::NoPathSegments)
    }
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
        let router = router_root_1();

        let location = LocationState::default();

        let matches = matches_routes(&router.tree, &location).unwrap();

        assert_eq!(matches.matches.len(), 1);

        for (index, node) in matches.matches.iter().enumerate() {
            match index {
                0 => {
                    let node = router.tree.find(node.route_id).unwrap();
                    assert_eq!(node.item.segment.kind(), RouteSegmentKind::Layout);
                }
                _ => {
                    unreachable!()
                }
            }
        }
    }

    fn router_root_1() -> Router {
        Router::default().route(Route::layout(view).child(Route::static_("aaa", view)))
    }

    #[test]
    fn test_root_matching_nested() {
        let router = router_nested_1();

        let location = LocationState::default();

        let matches = matches_routes(&router.tree, &location).unwrap();

        assert_eq!(matches.matches.len(), 3);

        for (index, node) in matches.matches.iter().enumerate() {
            match index {
                0..=1 => {
                    let node = router.tree.find(node.route_id).unwrap();
                    assert_eq!(node.item.segment.kind(), RouteSegmentKind::Layout);
                }
                2 => {
                    let node = router.tree.find(node.route_id).unwrap();
                    assert_eq!(node.item.segment.kind(), RouteSegmentKind::Index);
                }
                _ => {
                    unreachable!()
                }
            }
        }
    }

    fn router_nested_1() -> Router {
        Router::default().route(
            Route::layout(view)
                .child(Route::static_("aaa", view))
                .child(Route::layout(view).child(Route::index(view))),
        )
    }

    #[test]
    fn test_root_matching_nested_wild_card() {
        let router = router_nested_wild_card_1();

        let location = LocationState::default();
        let matches = matches_routes(&router.tree, &location).unwrap();

        assert_eq!(matches.matches.len(), 4);

        for (index, node) in matches.matches.iter().enumerate() {
            match index {
                0..=2 => {
                    let node = router.tree.find(node.route_id).unwrap();
                    assert_eq!(node.item.segment.kind(), RouteSegmentKind::Layout);
                }
                3 => {
                    let tnode = router.tree.find(node.route_id).unwrap();
                    assert_eq!(tnode.item.segment.kind(), RouteSegmentKind::Wildcard);
                    assert_eq!(node.params.get("any").map(|a| a.as_str()), Some(""));
                }
                _ => {
                    unreachable!()
                }
            }
        }
    }

    fn router_nested_wild_card_1() -> Router {
        Router::default().route(
            Route::layout(view)
                .child(Route::static_("aaa", view))
                .child(
                    Route::layout(view)
                        .child(Route::layout(view).child(Route::wildcard("any", view))),
                ),
        )
    }

    #[test]
    fn test_matching_nested_static() {
        let router = router_nested_static_1();

        let mut location = LocationState::default();
        location.goto("users/").unwrap();

        let matches = matches_routes(&router.tree, &location).unwrap();

        assert_eq!(matches.matches.len(), 1);
        for (index, node) in matches.matches.iter().enumerate() {
            match index {
                0 => {
                    let node = router.tree.find(node.route_id).unwrap();
                    assert_eq!(node.item.segment.kind(), RouteSegmentKind::Layout);
                }
                _ => {
                    unreachable!()
                }
            }
        }
    }

    fn router_nested_static_1() -> Router {
        Router::default()
            .route(
                Route::layout(view).child(
                    Route::static_("user", view)
                        .child(Route::static_("posts", view))
                        .child(Route::params("id", view))
                        .child(Route::wildcard("any", view)),
                ),
            )
            .route(Route::wildcard("any", view))
    }

    #[test]
    fn test_matching_nested_static_wildcard() {
        let router = router_static_wildcard_1();

        let mut location = LocationState::default();
        location.goto("users/").unwrap();

        let matches = matches_routes(&router.tree, &location).unwrap();

        assert_eq!(matches.matches.len(), 2);
        for (index, node) in matches.matches.iter().enumerate() {
            match index {
                0 => {
                    let node = router.tree.find(node.route_id).unwrap();
                    assert_eq!(node.item.segment.kind(), RouteSegmentKind::Layout);
                }
                1 => {
                    let tnode = router.tree.find(node.route_id).unwrap();
                    assert_eq!(tnode.item.segment.kind(), RouteSegmentKind::Wildcard);
                    assert_eq!(node.params.get("any").map(|a| a.as_str()), Some("/users"));
                }
                _ => {
                    unreachable!()
                }
            }
        }
    }

    fn router_static_wildcard_1() -> Router {
        Router::default()
            .route(
                Route::layout(view)
                    .child(
                        Route::static_("user", view)
                            .child(Route::static_("posts", view))
                            .child(Route::params("id", view))
                            .child(Route::wildcard("any", view)),
                    )
                    .child(Route::wildcard("any", view)),
            )
            .route(Route::wildcard("any", view))
    }

    #[test]
    // route : /user/posts
    fn test_matching_nested_static_static() {
        let router = router_user_posts_1();

        let mut location = LocationState::default();
        location.goto("user/posts").unwrap();

        {
            let matches = matches_routes(&router.tree, &location).unwrap();

            assert_eq!(matches.matches.len(), 3);
            for (index, node) in matches.matches.iter().enumerate() {
                match index {
                    0 => {
                        let node = router.tree.find(node.route_id).unwrap();
                        assert_eq!(node.item.segment.kind(), RouteSegmentKind::Layout);
                    }
                    1 => {
                        let tnode = router.tree.find(node.route_id).unwrap();
                        assert_eq!(tnode.item.segment.kind(), RouteSegmentKind::Static);
                        assert!(if let RouteSegment::Static(s) = &tnode.item.segment {
                            s == "user"
                        } else {
                            false
                        });
                    }
                    2 => {
                        let tnode = router.tree.find(node.route_id).unwrap();
                        assert_eq!(tnode.item.segment.kind(), RouteSegmentKind::Static);
                        assert!(if let RouteSegment::Static(s) = &tnode.item.segment {
                            s == "posts"
                        } else {
                            false
                        });
                    }
                    _ => {
                        unreachable!()
                    }
                }
            }
        }
    }

    #[test]
    // route : /user/1
    fn test_matching_nested_static_params() {
        let router = router_user_posts_1();

        let mut location = LocationState::default();
        location.goto("user/1").unwrap();

        {
            let matches = matches_routes(&router.tree, &location).unwrap();

            assert_eq!(matches.matches.len(), 3);
            for (index, node) in matches.matches.iter().enumerate() {
                match index {
                    0 => {
                        let node = router.tree.find(node.route_id).unwrap();
                        assert_eq!(node.item.segment.kind(), RouteSegmentKind::Layout);
                    }
                    1 => {
                        let tnode = router.tree.find(node.route_id).unwrap();
                        assert_eq!(tnode.item.segment.kind(), RouteSegmentKind::Static);
                        assert!(if let RouteSegment::Static(s) = &tnode.item.segment {
                            s == "user"
                        } else {
                            false
                        });
                    }
                    2 => {
                        let tnode = router.tree.find(node.route_id).unwrap();
                        assert_eq!(tnode.item.segment.kind(), RouteSegmentKind::Param);

                        assert!(if let RouteSegment::Param { name } = &tnode.item.segment {
                            name == "id"
                        } else {
                            false
                        });

                        assert_eq!(node.params.get("id").map(String::as_str), Some("1"));
                    }
                    _ => {
                        unreachable!()
                    }
                }
            }
        }
    }

    fn router_user_posts_1() -> Router {
        Router::default()
            .route(
                Route::layout(view)
                    .child(
                        Route::static_("user", view)
                            .child(Route::static_("posts", view))
                            .child(Route::params("id", view))
                            .child(Route::wildcard("any", view)),
                    )
                    .child(Route::wildcard("any", view)),
            )
            .route(Route::wildcard("any", view))
    }

    #[test]
    // route : /user/1
    fn test_matching_nested_static_params_root() {
        let router = router_user_posts_2();

        let mut location = LocationState::default();
        location.goto("user/1").unwrap();

        {
            let matches = matches_routes(&router.tree, &location).unwrap();

            assert_eq!(matches.matches.len(), 4);
            for (index, node) in matches.matches.iter().enumerate() {
                match index {
                    0 => {
                        let node = router.tree.find(node.route_id).unwrap();
                        assert_eq!(node.item.segment.kind(), RouteSegmentKind::Layout);
                    }
                    1 => {
                        let tnode = router.tree.find(node.route_id).unwrap();
                        assert_eq!(tnode.item.segment.kind(), RouteSegmentKind::Static);
                        assert!(if let RouteSegment::Static(s) = &tnode.item.segment {
                            s == "user"
                        } else {
                            false
                        });
                    }
                    2 => {
                        let tnode = router.tree.find(node.route_id).unwrap();
                        assert_eq!(tnode.item.segment.kind(), RouteSegmentKind::Param);

                        assert!(if let RouteSegment::Param { name } = &tnode.item.segment {
                            name == "id"
                        } else {
                            false
                        });

                        assert_eq!(node.params.get("id").map(String::as_str), Some("1"));
                    }
                    3 => {
                        let node = router.tree.find(node.route_id).unwrap();
                        assert_eq!(node.item.segment.kind(), RouteSegmentKind::Index);
                    }
                    _ => {
                        unreachable!()
                    }
                }
            }
        }
    }

    #[test]
    // route : /user/2
    fn test_matching_nested_static_params_root_2() {
        let router = router_user_posts_2();

        let mut location = LocationState::default();
        location.goto("user/2").unwrap();

        {
            let matches = matches_routes(&router.tree, &location).unwrap();

            assert_eq!(matches.matches.len(), 4);
            for (index, node) in matches.matches.iter().enumerate() {
                match index {
                    0 => {
                        let node = router.tree.find(node.route_id).unwrap();
                        assert_eq!(node.item.segment.kind(), RouteSegmentKind::Layout);
                    }
                    1 => {
                        let tnode = router.tree.find(node.route_id).unwrap();
                        assert_eq!(tnode.item.segment.kind(), RouteSegmentKind::Static);
                        assert!(if let RouteSegment::Static(s) = &tnode.item.segment {
                            s == "user"
                        } else {
                            false
                        });
                    }
                    2 => {
                        let tnode = router.tree.find(node.route_id).unwrap();
                        assert_eq!(tnode.item.segment.kind(), RouteSegmentKind::Param);

                        assert!(if let RouteSegment::Param { name } = &tnode.item.segment {
                            name == "id"
                        } else {
                            false
                        });

                        assert_eq!(node.params.get("id").map(String::as_str), Some("2"));
                    }
                    3 => {
                        let node = router.tree.find(node.route_id).unwrap();
                        assert_eq!(node.item.segment.kind(), RouteSegmentKind::Index);
                    }
                    _ => {
                        unreachable!()
                    }
                }
            }
        }
    }

    #[test]
    // route : /user/2/aaaaa
    fn test_matching_nested_static_params_root_2_aaaa() {
        let router = router_user_posts_2();

        let mut location = LocationState::default();
        location.goto("user/2/aaaa").unwrap();

        {
            let matches = matches_routes(&router.tree, &location).unwrap();

            assert_eq!(matches.matches.len(), 4);
            for (index, node) in matches.matches.iter().enumerate() {
                match index {
                    0 => {
                        let node = router.tree.find(node.route_id).unwrap();
                        assert_eq!(node.item.segment.kind(), RouteSegmentKind::Layout);
                    }
                    1 => {
                        let tnode = router.tree.find(node.route_id).unwrap();
                        assert_eq!(tnode.item.segment.kind(), RouteSegmentKind::Static);
                        assert!(if let RouteSegment::Static(s) = &tnode.item.segment {
                            s == "user"
                        } else {
                            false
                        });
                    }
                    2 => {
                        let tnode = router.tree.find(node.route_id).unwrap();
                        assert_eq!(tnode.item.segment.kind(), RouteSegmentKind::Param);

                        assert!(if let RouteSegment::Param { name } = &tnode.item.segment {
                            name == "id"
                        } else {
                            false
                        });

                        assert_eq!(node.params.get("id").map(String::as_str), Some("2"));
                    }
                    3 => {
                        let tnode = router.tree.find(node.route_id).unwrap();
                        assert_eq!(tnode.item.segment.kind(), RouteSegmentKind::Wildcard);
                        assert_eq!(node.params.get("any").map(|a| a.as_str()), Some("/aaaa"));
                    }
                    _ => {
                        unreachable!()
                    }
                }
            }
        }
    }

    fn router_user_posts_2() -> Router {
        Router::default()
            .route(
                Route::layout(view)
                    .child(
                        Route::static_("user", view)
                            .child(Route::static_("posts", view))
                            .child(
                                Route::params("id", view)
                                    .child(Route::index(view))
                                    .child(Route::static_("followers", view))
                                    .child(Route::wildcard("any", view)),
                            )
                            .child(Route::wildcard("any", view)),
                    )
                    .child(Route::wildcard("any", view)),
            )
            .route(Route::wildcard("any", view))
    }
}
