pub const SOURCE: &str = include_str!("icons.rs");

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
                shadcn::Button::new("Open")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-dropdown-menu-icons-trigger"),
            )
            .content(
                shadcn::DropdownMenuContent::new()
                    .align(shadcn::DropdownMenuAlign::Start)
                    .side_offset(Px(4.0)),
            )
            .entries([
                shadcn::DropdownMenuItem::new("Profile")
                    .leading_icon(IconId::new_static("lucide.user"))
                    .test_id("ui-gallery-dropdown-menu-icons-profile")
                    .into(),
                shadcn::DropdownMenuItem::new("Billing")
                    .leading_icon(IconId::new_static("lucide.credit-card"))
                    .into(),
                shadcn::DropdownMenuItem::new("Settings")
                    .leading_icon(IconId::new_static("lucide.settings"))
                    .into(),
                shadcn::DropdownMenuSeparator::new().into(),
                shadcn::DropdownMenuItem::new("Log out")
                    .leading_icon(IconId::new_static("lucide.log-out"))
                    .variant(fret_ui_shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive)
                    .into(),
            ])
    })
    .into_element(cx)
}
// endregion: example
