use std::panic::Location;

use reactive_graph::{
    computed::ArcAsyncDerived,
    graph::{ReactiveNode, Source, ToAnySource},
    signal::{ArcWriteSignal, arc_signal},
    traits::{DefinedAt, IsDisposed, Read, ReadUntracked, Update},
};

#[derive(Debug, Clone)]
pub struct ArcResource<T> {
    #[cfg(debug_assertions)]
    defined_at: &'static Location<'static>,
    derived: ArcAsyncDerived<T>,
    refetch: ArcWriteSignal<u64>,
}

impl<T> ArcResource<T>
where
    T: 'static,
{
    pub fn refetch(&self) {
        self.refetch.update(|u| *u += 1);
    }
}

impl<T> ArcResource<T>
where
    T: Send + Sync + 'static,
{
    pub fn new_with_initial<Fut>(
        initial_value: Option<T>,
        fun: impl Fn() -> Fut + Send + Sync + 'static,
    ) -> Self
    where
        Fut: Future<Output = T> + Send + 'static,
    {
        let (refetch, set_refetch) = arc_signal(0u64);
        Self {
            #[cfg(debug_assertions)]
            defined_at: std::panic::Location::caller(),
            derived: ArcAsyncDerived::new_with_initial(initial_value, move || {
                let _ = refetch.read();
                fun()
            }),
            refetch: set_refetch,
        }
    }
    pub fn new<Fut>(fun: impl Fn() -> Fut + Send + Sync + 'static) -> Self
    where
        Fut: Future<Output = T> + Send + 'static,
    {
        Self::new_with_initial(None, fun)
    }
}

impl<T> ArcResource<T>
where
    T: 'static,
{
    pub fn new_unsync_with_initial<Fut>(
        initial_value: Option<T>,
        fun: impl Fn() -> Fut + 'static,
    ) -> Self
    where
        Fut: Future<Output = T> + 'static,
    {
        let (refetch, set_refetch) = arc_signal(0u64);
        Self {
            #[cfg(debug_assertions)]
            defined_at: std::panic::Location::caller(),
            derived: ArcAsyncDerived::new_unsync_with_initial(initial_value, move || {
                let _ = refetch.read();
                fun()
            }),
            refetch: set_refetch,
        }
    }
    pub fn new_unsync<Fut>(fun: impl Fn() -> Fut + 'static) -> Self
    where
        Fut: Future<Output = T> + 'static,
    {
        Self::new_unsync_with_initial(None, fun)
    }
}

impl<T> DefinedAt for ArcResource<T> {
    #[inline(always)]
    fn defined_at(&self) -> Option<&'static Location<'static>> {
        #[cfg(debug_assertions)]
        {
            Some(self.defined_at)
        }
        #[cfg(not(debug_assertions))]
        {
            None
        }
    }
}

impl<T> ReadUntracked for ArcResource<T>
where
    T: 'static,
{
    type Value = <ArcAsyncDerived<T> as ReadUntracked>::Value;

    fn try_read_untracked(&self) -> Option<Self::Value> {
        self.derived.try_read_untracked()
    }
}

impl<T> IsDisposed for ArcResource<T>
where
    T: 'static,
{
    fn is_disposed(&self) -> bool {
        self.derived.is_disposed()
    }
}

impl<T> ToAnySource for ArcResource<T>
where
    T: 'static,
{
    fn to_any_source(&self) -> reactive_graph::graph::AnySource {
        self.derived.to_any_source()
    }
}

impl<T> Source for ArcResource<T>
where
    T: 'static,
{
    fn add_subscriber(&self, subscriber: reactive_graph::graph::AnySubscriber) {
        self.derived.add_subscriber(subscriber);
    }

    fn remove_subscriber(&self, subscriber: &reactive_graph::graph::AnySubscriber) {
        self.derived.remove_subscriber(subscriber);
    }

    fn clear_subscribers(&self) {
        self.derived.clear_subscribers();
    }
}

impl<T> ReactiveNode for ArcResource<T>
where
    T: 'static,
{
    fn mark_dirty(&self) {
        self.derived.mark_dirty();
    }

    fn mark_check(&self) {
        self.derived.mark_check();
    }

    fn mark_subscribers_check(&self) {
        self.derived.mark_subscribers_check();
    }

    fn update_if_necessary(&self) -> bool {
        self.derived.update_if_necessary()
    }
}

impl<T> IntoFuture for ArcResource<T>
where
    T: Clone + 'static,
{
    type Output = T;
    type IntoFuture = <ArcAsyncDerived<T> as IntoFuture>::IntoFuture;
    fn into_future(self) -> Self::IntoFuture {
        self.derived.into_future()
    }
}
