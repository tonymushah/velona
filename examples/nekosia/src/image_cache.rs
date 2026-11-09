use std::{num::NonZeroUsize, sync::Arc};

use image::ImageFormat;
use lru::LruCache;
use reqwest::header::CONTENT_TYPE;
use tokio::{
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, BufStream},
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
        owner::{expect_context, provide_context},
        traits::SignalOrFn,
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
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Debug)]
struct ImageCacheEntryValue {
    image: WeakBlob<u8>,
    // The image blob temp file
    file: BufStream<tokio::fs::File>,
    size: PhysicalSize<u32>,
}

#[derive(Debug)]
struct ImageCacheInner {
    lru: LruCache<ImageCacheEntryKey, ImageCacheEntryValue>,
}

impl ImageCacheInner {
    async fn get_image_data(
        &mut self,
        key: &ImageCacheEntryKey,
    ) -> Result<Option<ImageData>, ImageCacheError> {
        match self.lru.get_mut(key) {
            Some(entry) => {
                let blob: Blob<u8> = match entry.image.upgrade() {
                    Some(blob) => blob,
                    None => {
                        entry.file.rewind().await?;
                        let mut buf = Vec::<u8>::new();
                        entry.file.read_to_end(&mut buf).await?;
                        let blob = Blob::new(Arc::new(buf));
                        entry.image = blob.downgrade();
                        blob
                    }
                };
                log::trace!("Obtained image from key {:#?}", key);
                Ok(Some(ImageData {
                    data: blob,
                    format: velona::masonry::peniko::ImageFormat::Rgba8,
                    alpha_type: velona::masonry::peniko::ImageAlphaType::Alpha,
                    width: entry.size.width,
                    height: entry.size.height,
                }))
            }
            None => Ok(None),
        }
    }
    // Assuming that the `image` correspond at what said.
    async fn insert_rgba_image(
        &mut self,
        key: ImageCacheEntryKey,
        image: image::RgbaImage,
    ) -> Result<ImageData, ImageCacheError> {
        let mut file = tokio::io::BufStream::new(tokio::fs::File::from_std(
            tokio::task::spawn_blocking(tempfile::tempfile).await??,
        ));
        let size = PhysicalSize {
            width: image.width(),
            height: image.height(),
        };
        let buf = image.into_vec().into_boxed_slice();
        file.write_all(&buf).await?;
        file.flush().await?;
        let blob = Blob::new(Arc::new(buf));
        let weak_blob = blob.downgrade();

        log::trace!("Fetched image from key {:#?}", key);

        self.lru.put(
            key,
            ImageCacheEntryValue {
                image: weak_blob,
                file,
                size,
            },
        );

        Ok(ImageData {
            data: blob,
            format: velona::masonry::peniko::ImageFormat::Rgba8,
            alpha_type: velona::masonry::peniko::ImageAlphaType::Alpha,
            width: size.width,
            height: size.height,
        })
    }
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
    pub fn image_of_as_res(
        &self,
        entry: ImageCacheEntryKey,
    ) -> ArcResource<Result<ImageData, ImageCacheError>> {
        let sender = self.sender_task.clone();
        ArcResource::new(move || image_of_inner(sender.clone(), entry.clone()))
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

async fn fetch_rgba_image(
    key: &ImageCacheEntryKey,
    client: &reqwest::Client,
) -> Result<image::RgbaImage, ImageCacheError> {
    let response = client
        .get(key.url.clone())
        .send()
        .await?
        .error_for_status()?;

    let format: ImageFormat = if let Some(format) = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|d| image::ImageFormat::from_mime_type(d.to_str().ok()?))
    {
        format
    } else {
        image::ImageFormat::from_path(key.url.path())?
    };

    let body_bytes = response.bytes().await?;
    let mut dyn_img = image::load_from_memory_with_format(&body_bytes, format)?;
    if let Some(target_size) = &key.size {
        dyn_img = dyn_img.resize(
            target_size.width,
            target_size.height,
            image::imageops::FilterType::Lanczos3,
        );
    }
    Ok(dyn_img.into_rgba8())
}

async fn fetch_image(
    key: ImageCacheEntryKey,
    client: &reqwest::Client,
    images: &mut ImageCacheInner,
) -> Result<ImageData, ImageCacheError> {
    let img = fetch_rgba_image(&key, client).await?;
    images.insert_rgba_image(key, img).await
}

async fn task_run_fn(
    client: reqwest::Client,
    mut receiver: UnboundedReceiver<ImageActionVal>,
    mut images: ImageCacheInner,
) {
    while let Some(todo) = receiver.recv().await {
        if let Some(data) = images.get_image_data(&todo.key).await.transpose() {
            let _ = todo.sender.send(data);
        } else {
            let _ = todo
                .sender
                .send(fetch_image(todo.key, &client, &mut images).await);
        }
    }
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

pub fn provide_image_cache(cache: ImageCache) -> Arc<ImageCache> {
    let cache = Arc::new(cache);
    provide_context(cache.clone());
    cache
}

pub fn expect_image_cache() -> Arc<ImageCache> {
    expect_context()
}
