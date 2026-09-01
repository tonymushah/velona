use reactive_graph::{
    effect::{Effect, EffectFunction},
    owner::LocalStorage,
};

pub fn local_effect<F, T, M>(fun: F) -> Effect<LocalStorage>
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
