use masonry_core::core::Widget;
use masonry_raw_box::RawBox;

use crate::{AnyNewWidget, NewWidgetExt, subsecond::hot_local_effect, utils::ConsumeResult};

/// A hot view is a view that reload whenever its function signature changes.
///
/// Beware that if this hot view has a parent, the parent will also reload.
pub fn hot_view<V>(view: V) -> AnyNewWidget
where
    V: Fn() -> AnyNewWidget + 'static,
{
    let _box = RawBox::empty().prepare();
    {
        let box_ref = _box.create_velona_ref();
        hot_local_effect(move || {
            log::warn!("Behold! A hot view is coming...");
            log::trace!("It just came out of the Subsecond Blast Compiler.");
            let new_view = view();
            box_ref
                .edit_local_now(|mut this| {
                    RawBox::set_child(&mut this, new_view);
                })
                .consume_with_log_err();
            log::info!("Cooled (I mean reloaded)")
        });
    }
    _box.erased()
}
