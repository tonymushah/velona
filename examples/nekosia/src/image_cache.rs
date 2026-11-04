use std::{
    fs::File,
    num::NonZeroUsize,
    ops::{Deref, DerefMut},
    sync::{Arc, Weak},
};

use clonelicious::clone;
use lru::LruCache;
use reactive_stores::{ArcStore, KeyedAccess, Store, StoreField};
use tokio::{
    sync::{
        mpsc::{UnboundedReceiver, UnboundedSender},
        oneshot,
    },
    task::JoinHandle,
};
use url::Url;
use velona::{
    masonry::{
        dpi::PhysicalSize,
        peniko::{Blob, ImageData, WeakBlob},
    },
    reactive::{
        computed::ArcMemo,
        traits::{Read, ReadUntracked, SignalOrFn, UpdateUntracked, WithUntracked, Write},
    },
};
use velona_reactive_resource::ArcResource;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImageCacheEntryKey {
    pub url: Url,
    pub size: Option<PhysicalSize<u32>>,
}

#[derive(Debug, thiserror::Error)]
pub enum ImageCacheError {
    #[error("Error on fetching the image: {}", .0)]
    Reqwest(#[from] reqwest::Error),
    #[error("Image transformation error: {}", .0)]
    Image(#[from] image::error::ImageError),
    #[error("Error on joining on a JoinHandle: {}", .0)]
    TokioJoin(#[from] tokio::task::JoinError),
    #[error("the task tread already ended")]
    TaskThreadEnded,
}

#[derive(Debug, Store)]
struct ImageCacheEntryValue {
    image: WeakBlob<u8>,
    // The image blob temp file
    file: File,
    size: PhysicalSize<u32>,
}

#[derive(Debug)]
struct ImageCacheInner {
    lru: LruCache<ImageCacheEntryKey, Result<ImageCacheEntryValue, Arc<ImageCacheError>>>,
}

struct ImageActionVal {
    key: ImageCacheEntryKey,
    sender: oneshot::Sender<Result<ImageData, ImageCacheError>>,
}

#[derive(Debug)]
pub struct ImageCache {
    task_thread: JoinHandle<()>,
    sender_task: UnboundedSender<ImageActionVal>,
}

async fn image_of_inner(
    sender: UnboundedSender<ImageActionVal>,
    entry: ImageCacheEntryKey,
) -> Result<ImageData, ImageCacheError> {
    let (val_send, val_recv) = oneshot::channel();
    if sender
        .send(ImageActionVal {
            key: entry,
            sender: val_send,
        })
        .is_err()
    {
        return Err(ImageCacheError::TaskThreadEnded);
    }
    val_recv
        .await
        .map_err(|_| ImageCacheError::TaskThreadEnded)
        .flatten()
}

impl ImageCache {
    pub async fn image_of(&self, entry: ImageCacheEntryKey) -> Result<ImageData, ImageCacheError> {
        image_of_inner(self.sender_task.clone(), entry).await
    }
    pub fn image_of_signal<E>(&self, entry: E) -> ArcResource<Result<ImageData, ImageCacheError>>
    where
        E: SignalOrFn<Output = ImageCacheEntryKey> + Send + Sync + 'static,
    {
        let sender = self.sender_task.clone();
        ArcResource::new(move || image_of_inner(sender.clone(), entry.run()))
    }
    pub fn image_of_unsync_signal<E>(
        &self,
        entry: E,
    ) -> ArcResource<Result<ImageData, ImageCacheError>>
    where
        E: SignalOrFn<Output = ImageCacheEntryKey> + 'static,
    {
        let sender = self.sender_task.clone();
        ArcResource::new_unsync(move || image_of_inner(sender.clone(), entry.run()))
    }
}

async fn task_run_fn(
    client: reqwest::Client,
    mut receiver: UnboundedReceiver<ImageActionVal>,
    mut images: ImageCacheInner,
) {
    while let Some(todo) = receiver.recv().await {}
}

impl ImageCache {
    pub fn new(client: reqwest::Client, tokio_handle: tokio::runtime::Handle) -> Self {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel::<ImageActionVal>();
        let images = ImageCacheInner {
            lru: LruCache::sparse(NonZeroUsize::new(100).unwrap()),
        };
        let task = tokio_handle.spawn(async move {
            task_run_fn(client, receiver, images).await;
        });
        Self {
            task_thread: task,
            sender_task: sender,
        }
    }
}

impl Drop for ImageCache {
    fn drop(&mut self) {
        self.task_thread.abort();
    }
}
