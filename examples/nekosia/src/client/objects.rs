pub mod image;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum Rating {
    #[default]
    Safe,
    Suggestive,
}
