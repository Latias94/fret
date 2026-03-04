pub const SOURCE: &str = include_str!("icons.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::DropdownMenu::new_controllable(cx, None, false)
        .into_element_parts(
            cx,
            |cx| {
                shadcn::DropdownMenuTrigger::new(
                    shadcn::Button::new("Open")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-dropdown-menu-icons-trigger")
                        .into_element(cx),
                )
            },
            shadcn::DropdownMenuContent::new()
                .align(shadcn::DropdownMenuAlign::Start)
                .side_offset(Px(4.0)),
            |_cx| {
                [
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
                        .variant(shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive)
                        .into(),
                ]
            },
        )
        .test_id("ui-gallery-dropdown-menu-icons")
}
// endregion: example
