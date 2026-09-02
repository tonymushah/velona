use reactive_graph::{
    computed::Memo,
    effect::{Effect, EffectFunction},
    owner::LocalStorage,
};
#[cfg(feature = "subsecond")]
#[cfg_attr(docsrs, doc(feature = "subsecond"))]
pub use velona_subsecond::shrink_fit_subscribers_map;

use crate::utils::memo;

pub fn hot_local_effect<F, T, M>(fun: F) -> Effect<LocalStorage>
where
    F: EffectFunction<T, M> + 'static,
    T: 'static,
{
    #[cfg(feature = "subsecond")]
    {
        velona_subsecond::hot_local_effect(fun)
    }
    #[cfg(not(feature = "subsecond"))]
    {
        use reactive_graph::effect::Effect;

        Effect::new(fun)
    }
}

pub fn hot_value<F, V>(val_fn: F) -> impl Fn() -> V + 'static
where
    F: Fn() -> V + 'static,
    V: Send + Sync + Clone + 'static,
{
    #[cfg(feature = "subsecond")]
    {
        let read = velona_subsecond::hot_signal_value(val_fn);
        move || read()
    }
    #[cfg(not(feature = "subsecond"))]
    {
        move || val_fn()
    }
}

pub fn hot_value_with_memo_raw<F, V>(val_fn: F) -> Memo<V>
where
    F: Fn() -> V + 'static,
    V: Send + Sync + Clone + PartialEq + 'static,
{
    memo::unsync_memo(hot_value(val_fn))
}

pub fn hot_value_with_memo<F, V>(val_fn: F) -> impl Fn() -> V + 'static
where
    F: Fn() -> V + 'static,
    V: Send + Sync + Clone + PartialEq + 'static,
{
    let memo_val = hot_value_with_memo_raw(val_fn);
    move || {
        log::trace!("changes...");
        memo_val()
    }
}
