use std::sync::Arc;

use masonry::{
    core::{ArcStr, NewWidget, Widget},
    parley::{FontWeight, StyleProperty},
    widgets::{Badge, BadgeCountOverflow, Label},
};
use reactive_graph::{computed::Memo, graph::untrack, traits::Get};

use crate::{utils::memo::unsync_memo, widgets::label::NewLabelExt};

/// Similar to [`Badge::with_text`] but with a reactive text
pub fn badge_with_text<Tf, T>(text: Tf) -> NewWidget<Badge>
where
    Tf: Fn() -> T + 'static,
    T: Into<ArcStr>,
{
    let label = Label::new(untrack(&text))
        .with_style(StyleProperty::FontSize(12.0))
        .with_style(StyleProperty::FontWeight(FontWeight::BOLD))
        .prepare()
        .text(text);
    Badge::new(label).prepare()
}

/// Similar to [`Badge::count_with_overflow`]
/// but with a reactive count and overflow
pub fn badge_count_with_overflow<C, O>(count: C, overflow: O) -> NewWidget<Badge>
where
    C: Fn() -> u32 + 'static,
    O: Fn() -> BadgeCountOverflow + 'static,
{
    let text: Memo<ArcStr> = unsync_memo(move || {
        let count = count();
        match overflow() {
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
    });
    badge_with_text(move || text.get())
}

/// Similar to [`Badge::count`]
/// but with a reactive count
pub fn badge_count<C>(count: C) -> NewWidget<Badge>
where
    C: Fn() -> u32 + 'static,
{
    badge_count_with_overflow(count, BadgeCountOverflow::default)
}
