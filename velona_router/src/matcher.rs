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
    if let Some(route_id) = route_id {
        let Some(route) = tree.find(route_id.0) else {
            return Ok(None);
        };
        Ok(Some(
            route
                .child_ids()
                .into_iter()
                .map(|id| {
                    Ok::<_, MatchError>(RouteId(
                        id.try_into().map_err(|_| MatchError::ZeroRouteId)?,
                    ))
                })
                .collect::<Result<Box<[_]>, MatchError>>()?,
        ))
    } else {
        Ok(Some(
            tree.root_ids()
                .map(|id| {
                    Ok::<_, MatchError>(RouteId(
                        id.try_into().map_err(|_| MatchError::ZeroRouteId)?,
                    ))
                })
                .collect::<Result<Box<[_]>, MatchError>>()?,
        ))
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

        let url_path = location.url.path();

        let mut current_path_stack = String::with_capacity(url_path.len());

        'segments: for (index, path) in segments.iter().enumerate() {
            let is_at_end = (index + 1) >= segments.len();
            current_path_stack.push_str(path);
            current_path_stack.push('/');

            if path.is_empty() && is_at_end {
                if matches.is_empty()
                    && index == 0
                    && !when_empty::root_resolving(
                        tree,
                        &mut matches,
                        &mut params,
                        &root_ids,
                        url_path,
                        &current_path_stack,
                    )
                {
                    break;
                }
                when_empty::resolve_root_recursive(
                    tree,
                    &mut matches,
                    &mut params,
                    url_path,
                    &current_path_stack,
                )?;
            } else if !path.is_empty() {
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
                                    url_path.replacen(&current_path_stack, "", 1).clone(),
                                );
                                matches.push(RouteMatch {
                                    route_id: *id,
                                    params: params.clone(),
                                });
                                break 'segments;
                            }
                        }
                    }
                }
            }
        }
    } else {
        return Err(MatchError::NoPathSegments);
    }

    Ok(MatchedRoutes {
        matches: matches.into_boxed_slice(),
    })
}
