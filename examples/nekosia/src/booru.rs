use std::sync::Arc;

use arrayvec::ArrayVec;
use rand::RngExt;
use reactive_stores::Store;
use velona::reactive::owner::{expect_context, provide_context};

use crate::client::objects::Rating;

#[derive(Debug, Store, Default)]
pub struct BooruTagsSelection {
    pub excluded_tags: ArrayVec<Arc<str>, 20>,
    pub additional_tags: ArrayVec<Arc<str>, 20>,
}

#[derive(Debug, Store, Default)]
pub struct BooruContext {
    session: String,
    selected_category: Option<String>,
    selection: BooruTagsSelection,
    rating: Rating,
    // available: BooruTagsAvailable,
}

// #[derive(Debug, Store, Patch, Default)]
// pub struct BooruTagsAvailable {
//     pub tags: Vec<String>,
//     pub anime: Vec<String>,
//     pub characters: Vec<String>,
// }

impl BooruContext {
    pub fn with_session() -> Self {
        BooruContext {
            session: format!("{:x}", rand::rng().random_range::<u32, _>(1000..100000)),
            ..Default::default()
        }
    }
}

pub fn provide_booru_context() -> Store<BooruContext> {
    let store = Store::new(BooruContext::with_session());
    provide_context(store);
    store
}

pub fn expect_booru_context() -> Store<BooruContext> {
    expect_context()
}
