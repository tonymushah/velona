use std::{
    num::NonZeroU64,
    sync::atomic::{AtomicU64, Ordering},
};

/// An event handler id.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct HandlerId(pub(crate) NonZeroU64);

impl HandlerId {
    /// Allocates a new, unique `HandlerId`.
    ///
    /// All handlers are assigned ids automatically; you should only create
    /// an explicit id if you need to know it ahead of time.
    ///
    /// You must ensure that a given `HandlerId` is only ever used for one
    /// handler at a time.
    pub fn next() -> Self {
        static HANDLER_ID_COUNTER: AtomicU64 = AtomicU64::new(1);
        let id = HANDLER_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self(id.try_into().unwrap())
    }
}
