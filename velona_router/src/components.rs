use masonry_raw_box::RawBox;
use velona_core::{
    AnyNewWidget, NewWidgetExt,
    masonry_core::core::Widget,
    reactive::{
        effect::Effect,
        owner::{expect_context, provide_context},
    },
};

use crate::runtime::{ChildRoute, get_child_route, update_raw_box, use_matched_routes};

pub fn outlet() -> AnyNewWidget {
    let raw_box = RawBox::empty().prepare();

    let raw_box_ref = raw_box.create_velona_ref();
    let child_route = expect_context::<ChildRoute>();

    let route_id = child_route.route_id;

    let matches = use_matched_routes();
    Effect::new(move |_| {
        provide_context(child_route.params);
        {
            let child_route = get_child_route(child_route.index, &matches);
            provide_context(child_route);
        }

        let raw_box_ref = raw_box_ref.clone();

        Effect::new(move || {
            update_raw_box(&raw_box_ref, &route_id);
        });
    });
    raw_box.erased()
}
