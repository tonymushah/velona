use tree_arena::TreeArena;

use crate::route_tree::RouteNode;
use crate::route_tree::RouteSegment;

use crate::route_tree::RouteId;

use super::get_routes_child_id;

use super::MatchError;

use std::collections::HashMap;

use super::RouteMatch;

pub(crate) fn resolve_root_recursive(
    tree: &TreeArena<RouteNode>,
    matches: &mut Vec<RouteMatch>,
    params: &mut HashMap<String, String>,
    url_path: &str,
    current_path_stack: &str,
) -> Result<(), MatchError> {
    while let Some(last_match_route) = matches.iter().next_back() {
        let last_match_route = last_match_route.route_id;
        let Some(route_ids) = get_routes_child_id(tree, Some(last_match_route))? else {
            break;
        };
        if !root_resolving(
            tree,
            matches,
            params,
            &route_ids,
            url_path,
            current_path_stack,
        ) {
            break;
        }
    }
    Ok(())
}

pub(crate) fn root_resolving(
    tree: &TreeArena<RouteNode>,
    matches: &mut Vec<RouteMatch>,
    params: &mut HashMap<String, String>,
    root_ids: &[RouteId],
    url_path: &str,
    current_path_stack: &str,
) -> bool {
    let mut routes = root_ids
        .iter()
        .flat_map(|id| tree.roots().item(id.0))
        .collect::<Box<[_]>>();
    routes.sort_by_key(|a| a.item.segment.kind());
    for node in routes {
        match &node.item.segment {
            RouteSegment::Root => {
                matches.push(RouteMatch {
                    route_id: RouteId(node.id().try_into().unwrap()),
                    params: params.clone(),
                });
                return true;
            }
            RouteSegment::Wildcard { name } => {
                params.insert(
                    name.clone(),
                    url_path.replacen(current_path_stack, "", 1).clone(),
                );

                matches.push(RouteMatch {
                    route_id: RouteId(node.id().try_into().unwrap()),
                    params: params.clone(),
                });
            }
            _ => {}
        }
    }
    false
}
