pub const SOURCE: &str = include_str!("button.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_icons::IconId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let profile = cx.local_model_keyed("profile", String::new);
    let secure_site = cx.local_model_keyed("secure_site", String::new);
    let query = cx.local_model_keyed("query", String::new);

    let max_w = LayoutRefinement::default().w_full().max_w(Px(320.0));

    let copy_group = shadcn::InputGroup::new(profile)
        .a11y_label("Profile URL")
        .placeholder("https://x.com/shadcn")
        .control_test_id("ui-gallery-input-group-button-control")
        .trailing([shadcn::InputGroupButton::new("")
            .a11y_label("Copy")
            .test_id("ui-gallery-input-group-button-trailing-button")
            .size(shadcn::InputGroupButtonSize::IconXs)
            .icon(IconId::new_static("lucide.copy"))
            .into_element(cx)])
        .trailing_has_button(true)
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx);

    let connection_info = {
        let trigger = shadcn::InputGroupButton::new("")
            .a11y_label("Connection info")
            .variant(shadcn::ButtonVariant::Secondary)
            .size(shadcn::InputGroupButtonSize::IconXs)
            .icon(IconId::new_static("lucide.info"))
            .into_element(cx);
        let content = shadcn::PopoverContent::build(cx, |cx| {
            [shadcn::PopoverHeader::new([
                shadcn::PopoverTitle::new("Your connection is not secure.").into_element(cx),
                shadcn::PopoverDescription::new(
                    "You should not enter any sensitive information on this site.",
                )
                .into_element(cx),
            ])]
        });

        shadcn::Popover::new(cx, shadcn::PopoverTrigger::build(trigger), content)
            .align(shadcn::PopoverAlign::Start)
            .side_offset(Px(8.0))
            .into_element(cx)
    };

    let secure_group = shadcn::InputGroup::new(secure_site)
        .a11y_label("Secure site")
        .placeholder("example.com")
        .leading([
            connection_info,
            shadcn::InputGroupText::new("https://").into_element(cx),
        ])
        .leading_has_button(true)
        .trailing([shadcn::InputGroupButton::new("")
            .a11y_label("Favorite")
            .size(shadcn::InputGroupButtonSize::IconXs)
            .icon(IconId::new_static("lucide.star"))
            .into_element(cx)])
        .trailing_has_button(true)
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx);

    let search_group = shadcn::InputGroup::new(query)
        .a11y_label("Search query")
        .placeholder("Type to search...")
        .trailing([shadcn::InputGroupButton::new("Search")
            .variant(shadcn::ButtonVariant::Secondary)
            .into_element(cx)])
        .trailing_has_button(true)
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx);

    ui::v_stack(move |_cx| vec![copy_group, secure_group, search_group])
        .gap(Space::N4)
        .items_start()
        .layout(max_w)
        .into_element(cx)
        .test_id("ui-gallery-input-group-button")
}
// endregion: example
