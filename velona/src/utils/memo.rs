use reactive_graph::{
    callback::{Callable, UnsyncCallback},
    computed::Memo,
};

pub fn unsync_memo<F, T>(val: F) -> Memo<T>
where
    F: Fn() -> T + 'static,
    T: Send + Sync + PartialEq + 'static,
{
    let val_callback = UnsyncCallback::<(), T>::new(move |_| val());
    Memo::new(move |_| val_callback.run(()))
}
