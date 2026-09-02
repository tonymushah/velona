use reactive_graph::{
    effect::{Effect, EffectFunction},
    owner::LocalStorage,
};

pub fn local_effect<F, T, M>(fun: F) -> Effect<LocalStorage>
where
    F: EffectFunction<T, M> + 'static,
    T: 'static,
{
    use reactive_graph::effect::Effect;

    Effect::new(fun)
}
