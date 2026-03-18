pub const SOURCE: &str = include_str!("dropdown.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_icons::IconId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model_keyed("value", String::new);

    let trigger = shadcn::InputGroupButton::new("")
        .a11y_label("More")
        .test_id("ui-gallery-input-group-dropdown-leading-button")
        .variant(shadcn::ButtonVariant::Ghost)
        .size(shadcn::InputGroupButtonSize::IconXs)
        .children([icon::icon(cx, IconId::new_static("lucide.more-horizontal"))])
        .into_element(cx);

    let dropdown = shadcn::DropdownMenu::uncontrolled(cx)
        .compose()
        .trigger(trigger)
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

    shadcn::InputGroup::new(value)
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .placeholder("Enter file name")
        .a11y_label("File name")
        .control_test_id("ui-gallery-input-group-dropdown-control")
        .trailing([dropdown])
        .trailing_has_button(true)
        .test_id("ui-gallery-input-group-dropdown")
        .into_element(cx)
}
// endregion: example
