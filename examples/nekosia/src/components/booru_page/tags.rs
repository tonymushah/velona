use std::sync::Arc;

use color::{AlphaColor, palette::css::TRANSPARENT};
use velona::{
    NewWidgetExt,
    masonry::{
        core::{ArcStr, CollectionWidget, Selector, StyleProperty, Widget},
        layout::AsUnit,
        properties::{BorderColor, BorderWidth, ContentColor, CornerRadius, Padding},
        widgets::{Button, Flex},
    },
    reactive::{
        computed::Memo,
        owner::{ArenaItem, expect_context, provide_context},
        signal::signal,
        traits::{Read, ReadUntracked, Set, Update},
    },
    scoped_styling::{ApplyScopedStyles, ScopedPropstack, scoped_propstack},
    widgets::{
        View, button::NewButtonPressEventsExt, label::IntoLabel, sized_box::IntoNewSizedBox,
    },
};

use crate::{
    booru::{BooruContextStoreFields, expect_booru_context},
    components::booru_page::ResourceTagsResult,
    svgs::{MINUS_ICON, PLUS_ICON, STAR_ICON, build_static_svg},
};

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
        ["switch", "base"] {
            BorderColor::new(AlphaColor::from_rgb8(0x06, 0x06, 0xa0));
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
        ["label", "base"] {
            ContentColor::new(AlphaColor::from_rgb8(0x06, 0x06, 0xa0));
        }
    });
    provide_context(TagsViewStyle(style));
    (move || {
        let tags_read = tags.read();
        let tags_result = tags_read.as_ref().and_then(|a| a.as_ref().ok())?;
        let mut flex = Flex::column();

        flex = tags_view_base(flex, "Tags", tags_result.tags.clone()).with_fixed_spacer(8.px());

        flex = tags_view_base(flex, "Characters", tags_result.characters.clone())
            .with_fixed_spacer(8.px());

        flex = tags_view_base(flex, "Anime", tags_result.anime.clone()).with_fixed_spacer(8.px());

        Some(flex.prepare())
    })
    .into_new_sized_box()
}

const TAGS_PER_PAGE: usize = 5;

fn tags_view_base(mut flex: Flex, label: &str, tags: Arc<[ArcStr]>) -> Flex {
    let pages = tags.len() / TAGS_PER_PAGE;

    let (current_page, set_page) = signal(0usize);

    flex = flex
        .with_fixed(tags_controls(label, pages, current_page, set_page))
        .with_fixed_spacer(2.px());

    let tags_flex = Flex::row()
        .main_axis_alignment(velona::masonry::properties::types::MainAxisAlignment::Center)
        .prepare()
        .use_widget_mut(
            {
                let tags = tags.clone();
                move || {
                    tags.iter()
                        .skip(TAGS_PER_PAGE * current_page())
                        .take(TAGS_PER_PAGE)
                        .map(|tag| tag_switch(tag.clone()).into_erased())
                        .collect::<Box<[_]>>()
                }
            },
            |mut this, tags| {
                Flex::clear(&mut this);
                for tag in tags {
                    Flex::add_fixed(&mut this, tag);
                    Flex::add_fixed_spacer(&mut this, 3.px());
                }
            },
        );

    flex = flex.with_fixed(tags_flex);
    flex
}

fn tags_controls(
    label: &str,
    pages: usize,
    current_page: velona::reactive::signal::ReadSignal<usize>,
    set_page: velona::reactive::signal::WriteSignal<usize>,
) -> velona::masonry::core::NewWidget<Flex> {
    let is_on_first = Memo::new(move |_| current_page() == 0);
    let is_on_last = Memo::new(move |_| current_page() >= pages);
    Flex::row()
        .cross_axis_alignment(velona::masonry::properties::types::CrossAxisAlignment::Center)
        .main_axis_alignment(velona::masonry::properties::types::MainAxisAlignment::SpaceBetween)
        .with_fixed(
            Button::new(
                "Last"
                    .into_label()
                    .prepare()
                    .with_props_opt_reactive(move || {
                        if is_on_first() {
                            Some(ContentColor::new(TRANSPARENT))
                        } else {
                            None
                        }
                    }),
            )
            .prepare()
            .disabled_reactive(is_on_first)
            .on_primary(move || {
                set_page.maybe_update(|data| {
                    let Some(new) = data.checked_sub(1) else {
                        return false;
                    };
                    *data = new;
                    true
                });
            }),
        )
        .with_fixed(
            label
                .into_label()
                .with_style(StyleProperty::Underline(true))
                .prepare(),
        )
        .with_fixed(
            Button::new(
                "Next"
                    .into_label()
                    .prepare()
                    .with_props_opt_reactive(move || {
                        if is_on_last() {
                            Some(ContentColor::new(TRANSPARENT))
                        } else {
                            None
                        }
                    }),
            )
            .prepare()
            .disabled_reactive(is_on_last)
            .on_primary(move || {
                set_page.maybe_update(|data| {
                    let Some(new) = data.checked_add(1) else {
                        return false;
                    };
                    if new <= pages {
                        *data = new;
                        true
                    } else {
                        false
                    }
                });
            }),
        )
        .prepare()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TagState {
    Included,
    Excluded,
    Base,
}

fn tag_switch(tag: Arc<str>) -> impl View {
    let switch_style: TagsViewStyle = expect_context();
    let ctx = expect_booru_context();
    let tag_state = {
        let tag = tag.clone();
        Memo::new(move |_| {
            let selection = ctx.selection().read();
            let selected_tag = ctx.selected_category().read();
            if selected_tag.as_ref().is_some_and(|val| &*tag == val) {
                Some(TagState::Base)
            } else if selection.additional_tags.contains(&tag) {
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
            TagState::Base => Some("base".into()),
        }
    };
    let set_tag_state = {
        let tag = tag.clone();
        move |state: Option<TagState>| match state {
            Some(TagState::Excluded) => {
                if ctx.selected_category().read_untracked().is_none() {
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
            Some(TagState::Base) => {
                ctx.selected_category().set(Some(tag.to_string()));
            }
            None => {
                ctx.selected_category().maybe_update(|aa| {
                    if aa.is_none() {
                        false
                    } else {
                        aa.take();
                        true
                    }
                });
                ctx.selection().maybe_update(|selection| {
                    if selection.additional_tags.contains(&tag)
                        || selection.excluded_tags.contains(&tag)
                    {
                        selection
                            .excluded_tags
                            .retain(|inner_tag| inner_tag != &tag);
                        selection
                            .additional_tags
                            .retain(|inner_tag| inner_tag != &tag);
                        true
                    } else {
                        false
                    }
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
                    TagState::Included => Some(
                        build_static_svg(PLUS_ICON)
                            .prepare()
                            .with_props(Padding::right(4.px())),
                    ),
                    TagState::Excluded => Some(
                        build_static_svg(MINUS_ICON)
                            .prepare()
                            .with_props(Padding::right(4.px())),
                    ),
                    TagState::Base => Some(
                        build_static_svg(STAR_ICON)
                            .prepare()
                            .with_props(Padding::right(4.px())),
                    ),
                })
                .into_new_sized_box(),
            )
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
    .on_primary({
        let set_tag_state = set_tag_state.clone();
        move || match tag_state.read_untracked().as_ref() {
            Some(TagState::Excluded) => {
                set_tag_state(None);
            }
            Some(TagState::Included) => {
                set_tag_state(Some(TagState::Excluded));
            }
            None => set_tag_state(Some(TagState::Included)),
            _ => {}
        }
    })
    .on_secondary(move || match tag_state.read_untracked().as_ref() {
        None => set_tag_state(Some(TagState::Base)),
        Some(TagState::Base) => set_tag_state(None),
        _ => {}
    })
    .with_class("switch")
    .class_opt(class_state)
    .apply(&switch_style.0)
}
