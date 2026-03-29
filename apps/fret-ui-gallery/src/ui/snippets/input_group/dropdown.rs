pub const SOURCE: &str = include_str!("dropdown.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::{Corners, Px};
use fret_icons::IconId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let file_name = cx.local_model_keyed("file_name", String::new);
    let query = cx.local_model_keyed("query", String::new);

    let max_w = LayoutRefinement::default().w_full().max_w(Px(320.0));

    let more_trigger = shadcn::InputGroupButton::new("")
        .a11y_label("More")
        .test_id("ui-gallery-input-group-dropdown-leading-button")
        .variant(shadcn::ButtonVariant::Ghost)
        .size(shadcn::InputGroupButtonSize::IconXs)
        .icon(IconId::new_static("lucide.more-horizontal"))
        .into_element(cx);

    let more_dropdown = shadcn::DropdownMenu::uncontrolled(cx)
        .compose()
        .trigger(more_trigger)
        .content(
            shadcn::DropdownMenuContent::new()
                .align(shadcn::DropdownMenuAlign::End)
                .side_offset(Px(8.0))
                .align_offset(Px(-4.0)),
        )
        .entries([shadcn::DropdownMenuGroup::new([
            shadcn::DropdownMenuEntry::Item(shadcn::DropdownMenuItem::new("Settings")),
            shadcn::DropdownMenuEntry::Item(shadcn::DropdownMenuItem::new("Copy path")),
            shadcn::DropdownMenuEntry::Item(shadcn::DropdownMenuItem::new("Open location")),
        ])
        .into()])
        .into_element(cx);

    let search_trigger = shadcn::InputGroupButton::new("Search In...")
        .variant(shadcn::ButtonVariant::Ghost)
        .trailing_icon(IconId::new_static("lucide.chevron-down"))
        .into_element(cx);

    let search_dropdown = shadcn::DropdownMenu::uncontrolled(cx)
        .compose()
        .trigger(search_trigger)
        .content(
            shadcn::DropdownMenuContent::new()
                .align(shadcn::DropdownMenuAlign::End)
                .side_offset(Px(8.0)),
        )
        .entries([shadcn::DropdownMenuGroup::new([
            shadcn::DropdownMenuEntry::Item(shadcn::DropdownMenuItem::new("Documentation")),
            shadcn::DropdownMenuEntry::Item(shadcn::DropdownMenuItem::new("Blog Posts")),
            shadcn::DropdownMenuEntry::Item(shadcn::DropdownMenuItem::new("Changelog")),
        ])
        .into()])
        .into_element(cx);

    let file_name_group = shadcn::InputGroup::new(file_name)
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .placeholder("Enter file name")
        .a11y_label("File name")
        .control_test_id("ui-gallery-input-group-dropdown-control")
        .trailing([more_dropdown])
        .trailing_has_button(true)
        .into_element(cx);
    let query_group = shadcn::InputGroup::new(query)
        .corner_radii_override(Corners::all(Px(16.0)))
        .placeholder("Enter search query")
        .a11y_label("Search query")
        .trailing([search_dropdown])
        .trailing_has_button(true)
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx);

    ui::v_stack(move |_cx| vec![file_name_group, query_group])
        .gap(Space::N4)
        .items_start()
        .layout(max_w)
        .into_element(cx)
        .test_id("ui-gallery-input-group-dropdown")
}
// endregion: example
