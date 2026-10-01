use std::{ops::Deref, sync::Arc};

use masonry_raw_box::RawBox;
use velona_core::{
    AnyNewWidget, NewWidgetExt,
    masonry_core::core::Widget,
    reactive::{
        computed::{ArcMemo, Memo},
        effect::Effect,
        owner::{expect_context, provide_context},
        send_wrapper_ext::SendOption,
        traits::Read,
    },
    utils::ConsumeResult,
};

#[derive(Debug, Clone, Copy)]
pub struct RouteParams(Memo<Params>);

impl Deref for RouteParams {
    type Target = Memo<Params>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

use crate::{
    NavigationController,
    matcher::{MatchedRoutes, Params, matches_routes},
    route_tree::{RouteId, RouteTree},
};

pub struct RouterState {
    tree: Arc<RouteTree>,
    navigation: NavigationController,
    matches: ArcMemo<MatchedRoutes>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ChildRoute {
    pub route_id: Memo<Option<RouteId>>,
    pub params: RouteParams,
    pub index: usize,
}

impl RouterState {
    pub fn new(tree: RouteTree, navigation: NavigationController) -> Self {
        let tree = Arc::new(tree);
        Self {
            tree: tree.clone(),
            navigation: navigation.clone(),
            matches: ArcMemo::new(move |_| {
                matches_routes(&tree, &navigation.state.read()).unwrap_or_default()
            }),
        }
    }
    pub fn build_view(self) -> impl Fn() -> AnyNewWidget + Send {
        move || {
            provide_context(self.navigation.clone());

            provide_context(self.tree.clone());

            provide_context(self.matches.clone());

            let _box = RawBox::empty().prepare();

            let box_ref = _box.create_velona_ref();
            {
                let index = 0;

                let (route_id, params, child_route) = route_show_triad(index, &self.matches);

                provide_context(params);

                provide_context(child_route);

                Effect::new(move |_| {
                    update_raw_box(&box_ref, &route_id);
                });
            }

            _box.erased()
        }
    }
}

#[track_caller]
pub(crate) fn update_raw_box(
    box_ref: &velona_core::widget_ref::VelonaWidgetRef<RawBox>,
    route_id: &Memo<Option<RouteId>>,
) {
    let tree = use_route_tree();
    if let Some(route_id) = route_id()
        && let Some(route_node) = {
            // This is fine because the route is memoized

            tree.find(route_id)
        }
    {
        let widget = SendOption::new_local(Some((route_node.item.view)()));

        box_ref
            .edit(move |mut this| {
                if let Some(widget) = widget.take() {
                    RawBox::set_child(&mut this, widget)
                }
            })
            .consume_with_log_err();
    } else {
        box_ref
            .edit(|mut this| {
                RawBox::remove_child(&mut this);
            })
            .consume_with_log_err();
    }
}

fn route_show_triad(
    index: usize,
    matches: &ArcMemo<MatchedRoutes>,
) -> (Memo<Option<RouteId>>, RouteParams, ChildRoute) {
    let root_route_id = get_route_id_memo(index, matches.clone());

    let root_params = get_root_params_memo(index, matches.clone());

    let child_route = get_child_route(index, matches);

    (root_route_id, root_params, child_route)
}

pub(crate) fn get_child_route(index: usize, matches: &ArcMemo<MatchedRoutes>) -> ChildRoute {
    let index = index + 1;
    ChildRoute {
        route_id: get_route_id_memo(index, matches.clone()),
        params: get_root_params_memo(index, matches.clone()),
        index,
    }
}

fn get_root_params_memo(index: usize, matches: ArcMemo<MatchedRoutes>) -> RouteParams {
    let matches = matches.clone();
    RouteParams(Memo::new(move |_| {
        matches
            .read()
            .matches
            .get(index)
            .map(|d| d.params.clone())
            .unwrap_or_default()
    }))
}

fn get_route_id_memo(index: usize, matches: ArcMemo<MatchedRoutes>) -> Memo<Option<RouteId>> {
    let matches = matches.clone();
    Memo::new(move |_| matches.read().matches.get(index).map(|d| d.route_id))
}

pub fn use_route_tree() -> Arc<RouteTree> {
    expect_context()
}

pub fn use_navigation_controller() -> NavigationController {
    expect_context()
}

pub fn use_matched_routes() -> ArcMemo<MatchedRoutes> {
    expect_context()
}

pub fn use_params() -> RouteParams {
    expect_context()
}
