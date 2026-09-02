use std::sync::{Arc, LazyLock, Mutex};

pub use dioxus_devtools_types::DevserverMsg;
use or_poisoned::OrPoisoned;
use reactive_graph::{
    effect::{Effect, EffectFunction},
    graph::{AnySource, ToAnySource, untrack},
    owner::{LocalStorage, on_cleanup},
    signal::{ReadSignal, Trigger, signal},
    traits::{Notify, Track},
};
use rustc_hash::FxHashMap;
use subsecond::{HotFn, HotFnPtr};

pub fn connect_to_dx_cli<C>(callback: C) -> bool
where
    C: Fn(DevserverMsg) + Send + Sync + 'static,
{
    let Some(endpoint) = dioxus_cli_config::devserver_ws_endpoint() else {
        return false;
    };
    std::thread::spawn(move || {
        let uri = format!(
            "{endpoint}?aslr_reference={}&build_id={}&pid={}",
            subsecond::aslr_reference(),
            dioxus_cli_config::build_id(),
            std::process::id()
        );

        let (mut websocket, _req) = match tungstenite::connect(uri) {
            Ok((websocket, req)) => (websocket, req),
            Err(_) => return,
        };

        while let Ok(msg) = websocket.read() {
            if let tungstenite::Message::Text(text) = msg
                && let Ok(msg) = serde_json::from_str(&text)
            {
                callback(msg);
            }
        }
    });
    true
}

type CurrentHotPtr = Box<dyn Fn() -> Option<subsecond::HotFnPtr> + Send + Sync>;

type HotReloadSubscribers =
    LazyLock<Mutex<FxHashMap<AnySource, (Trigger, HotFnPtr, CurrentHotPtr)>>>;

static HOT_RELOAD_SUBSCRIBERS: HotReloadSubscribers = LazyLock::new(|| {
    subsecond::register_handler(Arc::new(|| {
        HOT_RELOAD_SUBSCRIBERS
            .lock()
            .or_poisoned()
            .retain(|_, (trigger, prev_ptr, hot_fn_ptr)| match hot_fn_ptr() {
                None => false,
                Some(curr_hot_ptr) => {
                    if curr_hot_ptr != *prev_ptr {
                        log::warn!(
                            "{prev_ptr:?} <> \
                            {curr_hot_ptr:?}",
                        );
                        *prev_ptr = curr_hot_ptr;

                        trigger.notify();
                    }
                    true
                }
            });
    }));
    Default::default()
});

pub fn shrink_fit_subscribers_map() {
    HOT_RELOAD_SUBSCRIBERS.lock().or_poisoned().shrink_to_fit();
}

pub fn hot_signal_value<F, V>(val_fn: F) -> ReadSignal<V>
where
    F: Fn() -> V + 'static,
    V: Send + Sync + 'static,
{
    let (hot_fn_ptr, fun) = {
        let fun = Arc::new(Mutex::new(subsecond::HotFn::current(val_fn)));
        (
            {
                let fun = Arc::downgrade(&fun);
                let wrapped = send_wrapper::SendWrapper::new(move || {
                    fun.upgrade().map(|n| n.lock().or_poisoned().ptr_address())
                });
                // it's not redundant, it's due to the SendWrapper deref
                #[allow(clippy::redundant_closure)]
                Box::new(move || wrapped())
            },
            move || fun.lock().or_poisoned().call(()),
        )
    };
    let mut fun = HotFn::current(fun);
    let (val, set_val) = signal(untrack(|| fun.call(())));
    let trigger = Trigger::default();
    let initial_ptr = hot_fn_ptr().unwrap();
    HOT_RELOAD_SUBSCRIBERS
        .lock()
        .or_poisoned()
        .insert(trigger.to_any_source(), (trigger, initial_ptr, hot_fn_ptr));
    Effect::new(move || {
        trigger.track();
        set_val(fun.call(()))
    });

    on_cleanup({
        let source = trigger.to_any_source();
        move || {
            HOT_RELOAD_SUBSCRIBERS.lock().or_poisoned().remove(&source);
        }
    });
    val
}

pub fn hot_local_effect<F, T, M>(mut fun: F) -> Effect<LocalStorage>
where
    F: EffectFunction<T, M> + 'static,
    T: 'static,
{
    let (hot_fn_ptr, fun) = {
        let fun = Arc::new(Mutex::new(subsecond::HotFn::current(
            move |last_val: Option<T>| fun.run(last_val),
        )));
        (
            {
                let fun = Arc::downgrade(&fun);
                let wrapped = send_wrapper::SendWrapper::new(move || {
                    fun.upgrade().map(|n| n.lock().or_poisoned().ptr_address())
                });
                // it's not redundant, it's due to the SendWrapper deref
                #[allow(clippy::redundant_closure)]
                Box::new(move || wrapped())
            },
            move |prev| fun.lock().or_poisoned().call((prev,)),
        )
    };
    let mut fun = HotFn::current(fun);
    let trigger = Trigger::default();
    let initial_ptr = hot_fn_ptr().unwrap();
    HOT_RELOAD_SUBSCRIBERS
        .lock()
        .or_poisoned()
        .insert(trigger.to_any_source(), (trigger, initial_ptr, hot_fn_ptr));
    let effect = Effect::new(move |old_value: Option<T>| {
        trigger.track();
        fun.call((old_value,))
    });

    on_cleanup({
        let source = trigger.to_any_source();
        move || {
            HOT_RELOAD_SUBSCRIBERS.lock().or_poisoned().remove(&source);
        }
    });
    effect
}
