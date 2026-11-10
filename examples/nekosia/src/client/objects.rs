pub mod image;
pub mod image_id;

use reactive_stores::Patch;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, Patch)]
#[serde(rename_all = "camelCase")]
pub enum Rating {
    #[default]
    Safe,
    Suggestive,
}
