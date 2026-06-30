use masonry::{
    core::{FromDynWidget, NewWidget, Widget, WidgetMut},
    widgets::ResizeObserver,
};

#[cfg(doc)]
use masonry::core::MutateCtx;

use crate::{NewWidgetExt, utils::ConsumeResult};

/// A trait that allows you to [listen](Self::on_resize) any [`NewWidget`] sizes changes
/// _by wrapping it inside a [`ResizeObserver`]_.
pub trait BindResizeObserver {
    /// Listen to the widget sizes changes.
    ///
    /// Use the [`WidgetMut<ResizeObserver>::ctx`] to get the [`MutateCtx`].
    ///
    /// You can use the [`MutateCtx`] to access the size of this widget through methods like
    /// [`border_box`](MutateCtx::border_box) and [`content_box`](MutateCtx::content_box).
    ///
    /// ## Caveats
    ///
    /// To avoid infinite loops, it is recommended to not use the reported size
    /// in a way which will edit the child widget’s size.
    /// For example, using this to write the width of a label in that label would be unlikely to reach a steady-state.
    /// Currently Masonry will not detect these loops automatically,
    /// so using this incorrectly might cause your application to stop responding.
    ///
    /// You might also get several of the resulting actions in a sequence.
    fn on_resize<E>(self, handler: E) -> NewWidget<ResizeObserver>
    where
        E: Fn(WidgetMut<ResizeObserver>) + 'static;
}

impl<W> BindResizeObserver for NewWidget<W>
where
    W: Widget + FromDynWidget + ?Sized,
{
    fn on_resize<E>(self, handler: E) -> NewWidget<ResizeObserver>
    where
        E: Fn(WidgetMut<ResizeObserver>) + 'static,
    {
        let obs = ResizeObserver::new(self).prepare();
        let obs_ref = obs.create_velona_ref();
        obs.on_action(move |_| {
            obs_ref.edit_local_now(&handler).consume_with_log_err();
        })
    }
}
