//! Utilities for working with asynchronous tasks.
// A blatant copy of [`leptos::task`] module

use any_spawner::Executor;
use std::future::Future;

/// Spawns a thread-safe [`Future`].
///
/// This will be run without current reactive owner since it is a call of [`Builder::spawn_fn`](crate::Builder::spawn_fn)
// It is technically possible but I don't want you guys reporting
// a bunch of [`SendWraper`] in the near feature
#[track_caller]
#[inline(always)]
pub fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
    reactive_graph::spawn(fut);
}

/// Spawns a [`Future`] that cannot be sent across threads.
#[track_caller]
#[inline(always)]
pub fn spawn_local(fut: impl Future<Output = ()> + 'static) {
    reactive_graph::spawn_local(fut)
}

/// Waits until the next "tick" of the current async executor.
pub async fn tick() {
    Executor::tick().await
}

pub use reactive_graph::{spawn_local_scoped, spawn_local_scoped_with_cancellation};
