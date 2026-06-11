use std::sync::Arc;

use masonry::{core::ArcStr, widgets::BadgeCountOverflow};

/// a relatively simple
pub fn badge_count_overflow(count: u32, overflow: BadgeCountOverflow) -> ArcStr {
    match overflow {
        BadgeCountOverflow::Exact => Arc::from(count.to_string().into_boxed_str()),
        BadgeCountOverflow::Cap { max, show_plus } => {
            if count > max {
                if show_plus {
                    Arc::from(format!("{max}+").into_boxed_str())
                } else {
                    Arc::from(max.to_string().into_boxed_str())
                }
            } else {
                Arc::from(count.to_string().into_boxed_str())
            }
        }
    }
}
