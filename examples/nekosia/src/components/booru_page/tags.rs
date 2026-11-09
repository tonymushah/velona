use std::sync::Arc;

use color::AlphaColor;
use velona::{
    NewWidgetExt,
    masonry::{
        core::{Selector, StyleProperty, Widget},
        layout::AsUnit,
        properties::{BorderColor, BorderWidth, ContentColor, CornerRadius, Padding},
        widgets::{Button, Flex},
    },
    reactive::{
        computed::Memo,
        owner::{ArenaItem, expect_context, provide_context},
        traits::{Read, ReadUntracked, Update},
    },
    scoped_styling::{ApplyScopedStyles, ScopedPropstack, scoped_propstack},
    widgets::{
        View, button::NewButtonPressEventsExt, label::IntoLabel, sized_box::IntoNewSizedBox,
    },
};
use velona_reactive_resource::Resource;

use crate::{
    booru::{BooruContextStoreFields, expect_booru_context},
    svgs::{MINUS_ICON, PLUS_ICON, build_static_svg},
};

type ResourceTagsResult =
    Resource<Result<crate::client::endpoint::tags::GetTagsResponse, super::FetchError>>;

#[derive(Debug, Clone, Copy)]
struct TagsViewStyle(ArenaItem<ScopedPropstack>);

pub fn tags_view(tags: ResourceTagsResult) -> impl View {
    let style = ArenaItem::new(scoped_propstack! {
        ["switch"] {
            BorderColor::new(AlphaColor::from_rgb8(0x06, 0x06, 0x06));
            BorderWidth::all(1.px());
            Padding::from_vh(2.px(), 4.px());
            CornerRadius::all(3.px())
        }
        ["switch", "included"] {
            BorderColor::new(AlphaColor::from_rgb8(0x06, 0xa0, 0x06))
        }
        ["switch", "excluded"] {
            BorderColor::new(AlphaColor::from_rgb8(0xa0, 0x06, 0x06));
        }
        ["label"] {
            ContentColor::new(AlphaColor::from_rgb8(0x06, 0x06, 0x06));
        }
        ["label", "included"] {
            ContentColor::new(AlphaColor::from_rgb8(0x06, 0xa0, 0x06))
        }
        ["label", "excluded"] {
            ContentColor::new(AlphaColor::from_rgb8(0xa0, 0x06, 0x06));
        }
    });
    provide_context(TagsViewStyle(style));
    (move || {
        let tags_read = tags.read();
        let tags_result = tags_read.as_ref().and_then(|a| a.as_ref().ok())?;
        let mut flex = Flex::column();

        flex = tags_view_base(flex, "Tags", &tags_result.tags).with_fixed_spacer(8.px());

        flex =
            tags_view_base(flex, "Characters", &tags_result.characters).with_fixed_spacer(8.px());

        flex = tags_view_base(flex, "Anime", &tags_result.anime).with_fixed_spacer(8.px());

        Some(flex.prepare())
    })
    .into_new_sized_box()
}

fn tags_view_base(mut flex: Flex, label: &str, tags: &[String]) -> Flex {
    flex = flex
        .with_fixed(
            label
                .into_label()
                .with_style(StyleProperty::Underline(true))
                .prepare(),
        )
        .with_fixed_spacer(2.px());
    let mut tags_flex = Flex::row()
        .main_axis_alignment(velona::masonry::properties::types::MainAxisAlignment::Center);
    for tag in tags.iter().take(20) {
        tags_flex = tags_flex
            .with_fixed(tag_switch(tag.as_str().into()).into_new_widget())
            .with_fixed_spacer(4.px());
    }
    flex = flex.with_fixed(tags_flex.prepare());
    flex
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TagState {
    Included,
    Excluded,
}

fn tag_switch(tag: Arc<str>) -> impl View {
    let switch_style: TagsViewStyle = expect_context();
    let ctx = expect_booru_context();
    let tag_state = {
        let tag = tag.clone();
        Memo::new(move |_| {
            let selection = ctx.selection().read();
            if selection.additional_tags.contains(&tag) {
                Some(TagState::Included)
            } else if selection.excluded_tags.contains(&tag) {
                Some(TagState::Excluded)
            } else {
                None
            }
        })
    };
    let class_state = move || -> Option<String> {
        let tag_state_read = tag_state.read();
        match tag_state_read.as_ref()? {
            TagState::Included => Some("included".into()),
            TagState::Excluded => Some("excluded".into()),
        }
    };
    let set_tag_state = {
        let tag = tag.clone();
        move |state: Option<TagState>| match state {
            Some(TagState::Excluded) => {
                ctx.selection().maybe_update(|selection| {
                    if selection.excluded_tags.is_full() {
                        false
                    } else {
                        selection
                            .additional_tags
                            .retain(|inner_tag| inner_tag != &tag);
                        selection.excluded_tags.push(tag.clone());
                        true
                    }
                });
            }
            Some(TagState::Included) => {
                ctx.selection().maybe_update(|selection| {
                    if selection.additional_tags.is_full() {
                        false
                    } else {
                        selection
                            .excluded_tags
                            .retain(|inner_tag| inner_tag != &tag);
                        selection.additional_tags.push(tag.clone());
                        true
                    }
                });
            }
            None => {
                ctx.selection().update(|selection| {
                    selection
                        .excluded_tags
                        .retain(|inner_tag| inner_tag != &tag);
                    selection
                        .additional_tags
                        .retain(|inner_tag| inner_tag != &tag);
                });
            }
        }
    };
    Button::new(
        Flex::row()
            .main_axis_alignment(velona::masonry::properties::types::MainAxisAlignment::Center)
            .cross_axis_alignment(velona::masonry::properties::types::CrossAxisAlignment::Center)
            .with_fixed(
                (move || match tag_state()? {
                    TagState::Included => Some(build_static_svg(PLUS_ICON).prepare()),
                    TagState::Excluded => Some(build_static_svg(MINUS_ICON).prepare()),
                })
                .into_new_sized_box(),
            )
            .with_fixed_spacer(4.px())
            .with_fixed(
                tag.into_label()
                    .prepare()
                    .with_class("label")
                    .class_opt(class_state)
                    .apply(&switch_style.0),
            )
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
    .with_class("switch")
    .class_opt(class_state)
    .apply(&switch_style.0)
}
