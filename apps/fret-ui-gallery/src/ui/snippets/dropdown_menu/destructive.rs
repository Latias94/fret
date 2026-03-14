pub const SOURCE: &str = include_str!("destructive.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    super::preview_frame_with(cx, |cx| {
        shadcn::DropdownMenu::uncontrolled(cx)
            .compose()
            .trigger(
                shadcn::Button::new("Actions")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-dropdown-menu-destructive-trigger"),
            )
            .content(
                shadcn::DropdownMenuContent::new()
                    .align(shadcn::DropdownMenuAlign::Start)
                    .side_offset(Px(4.0)),
            )
            .entries([
                shadcn::DropdownMenuGroup::new([
                    shadcn::DropdownMenuItem::new("Edit")
                        .leading_icon(IconId::new_static("lucide.pencil"))
                        .into(),
                    shadcn::DropdownMenuItem::new("Share")
                        .leading_icon(IconId::new_static("lucide.share"))
                        .into(),
                ])
                .into(),
                shadcn::DropdownMenuSeparator::new().into(),
                shadcn::DropdownMenuGroup::new([shadcn::DropdownMenuItem::new("Delete")
                    .leading_icon(IconId::new_static("lucide.trash"))
                    .variant(fret_ui_shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive)
                    .into()])
                .into(),
            ])
    })
    .into_element(cx)
}
// endregion: example
