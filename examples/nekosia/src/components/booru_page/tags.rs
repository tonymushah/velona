use std::sync::Arc;

use velona::{
    masonry::{
        core::{StyleProperty, Widget},
        layout::AsUnit,
        widgets::{Button, Flex},
    },
    reactive::{
        computed::Memo,
        traits::{Read, ReadUntracked, Update},
    },
    widgets::{
        View, button::NewButtonPressEventsExt, label::IntoLabel, sized_box::IntoNewSizedBox,
    },
};
use velona_reactive_resource::Resource;

use crate::{
    booru::{BooruContextStoreFields, expect_booru_context},
    components::booru_page::FetchError,
};

type ResourceTagsResult =
    Resource<Result<crate::client::endpoint::tags::GetTagsResponse, super::FetchError>>;

pub fn tags_error(tags: ResourceTagsResult) -> impl View {
    (move || {
        let tags_read = tags.read();
        let err = tags_read.as_ref()?.as_ref().err()?;
        Some(
            Flex::column()
                .cross_axis_alignment(
                    velona::masonry::properties::types::CrossAxisAlignment::Center,
                )
                .with_fixed(format!("{err}").into_label().prepare())
                .with_fixed(
                    "Details"
                        .into_label()
                        .with_style(StyleProperty::Underline(true))
                        .prepare(),
                )
                .with_fixed(match err {
                    FetchError::Client(crate::client::error::Error::NekosiaRequest(err)) => {
                        format!("{} (status: {})", err.message, err.status)
                            .into_label()
                            .prepare()
                    }
                    FetchError::Client(crate::client::error::Error::Reqwest(err)) => format!(
                        "Reqwest err: [url: {}] [type: {}]",
                        err.url().map(|e| e.as_str()).unwrap_or("(nothing)"),
                        if err.is_body() {
                            "body"
                        } else if err.is_builder() {
                            "builder"
                        } else if err.is_connect() {
                            "connect"
                        } else if err.is_decode() {
                            "decode"
                        } else if err.is_dns() {
                            "dns"
                        } else if err.is_redirect() {
                            "redirect"
                        } else if err.is_request() {
                            "request"
                        } else if err.is_status() {
                            "status"
                        } else if err.is_timeout() {
                            "timeout"
                        } else if err.is_upgrade() {
                            "upgrade"
                        } else {
                            "unknown"
                        }
                    )
                    .into_label()
                    .prepare(),
                    FetchError::Client(crate::client::error::Error::Url(url)) => {
                        format!("Url parsing error: {}", url).into_label().prepare()
                    }
                    FetchError::TokioJoin(join_error) => format!("Tokio Join error ({join_error})")
                        .into_label()
                        .prepare(),
                })
                .prepare(),
        )
    })
    .into_new_sized_box()
}

pub fn tags_view(tags: ResourceTagsResult) -> impl View {
    (move || {
        let tags_read = tags.read();
        let tags_result = tags_read.as_ref().and_then(|a| a.as_ref().ok())?;
        let mut flex = Flex::column();
        {
            flex = flex.with_fixed(
                "Tags"
                    .into_label()
                    .with_style(StyleProperty::Underline(true))
                    .prepare(),
            );
            let mut tags_flex = Flex::row()
                .main_axis_alignment(velona::masonry::properties::types::MainAxisAlignment::Center);
            for tag in &tags_result.tags {
                tags_flex = tags_flex.with_fixed(tag_switch(tag.as_str().into()).into_new_widget());
            }
            flex = flex.with_fixed(tags_flex.prepare());
        }
        {
            flex = flex.with_fixed(
                "Character"
                    .into_label()
                    .with_style(StyleProperty::Underline(true))
                    .prepare(),
            );
            let mut tags_flex = Flex::row()
                .main_axis_alignment(velona::masonry::properties::types::MainAxisAlignment::Center);
            for tag in &tags_result.characters {
                tags_flex = tags_flex.with_fixed(tag_switch(tag.as_str().into()).into_new_widget());
            }
            flex = flex.with_fixed(tags_flex.prepare());
        }
        {
            flex = flex.with_fixed(
                "Anime"
                    .into_label()
                    .with_style(StyleProperty::Underline(true))
                    .prepare(),
            );
            let mut tags_flex = Flex::row()
                .main_axis_alignment(velona::masonry::properties::types::MainAxisAlignment::Center);
            for tag in &tags_result.anime {
                tags_flex = tags_flex.with_fixed(tag_switch(tag.as_str().into()).into_new_widget());
            }
            flex = flex.with_fixed(tags_flex.prepare());
        }
        Some(flex.prepare())
    })
    .into_new_sized_box()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TagState {
    Included,
    Excluded,
}

fn tag_switch(tag: Arc<str>) -> impl View {
    let ctx = expect_booru_context();
    let tag_state = {
        let tag = tag.clone();
        Memo::new(move |_| {
            let selection = ctx.selection().read();
            if selection.additional_tags.contains(&*tag) {
                Some(TagState::Included)
            } else if selection.excluded_tags.contains(&*tag) {
                Some(TagState::Excluded)
            } else {
                None
            }
        })
    };
    let set_tag_state = {
        let tag = tag.clone();
        move |state: Option<TagState>| match state {
            Some(TagState::Excluded) => {
                ctx.selection().update(|selection| {
                    selection.additional_tags.take(&tag);
                    selection.excluded_tags.insert(tag.clone());
                });
            }
            Some(TagState::Included) => {
                ctx.selection().update(|selection| {
                    selection.excluded_tags.take(&tag);
                    selection.additional_tags.insert(tag.clone());
                });
            }
            None => {
                ctx.selection().update(|selection| {
                    selection.excluded_tags.take(&tag);
                    selection.additional_tags.take(&tag.clone());
                });
            }
        }
    };
    Button::new(
        Flex::row()
            .main_axis_alignment(velona::masonry::properties::types::MainAxisAlignment::Center)
            .cross_axis_alignment(velona::masonry::properties::types::CrossAxisAlignment::Center)
            // TODO fill this
            .with_fixed((move || None::<()>).into_new_sized_box())
            .with_fixed_spacer(4.px())
            .with_fixed(tag.into_label().prepare())
            .prepare(),
    )
    .prepare()
    .on_primary(move || match tag_state.read_untracked().as_ref() {
        Some(TagState::Excluded) => {
            set_tag_state(None);
        }
        Some(TagState::Included) => {
            set_tag_state(Some(TagState::Excluded));
        }
        None => set_tag_state(Some(TagState::Included)),
    })
}
