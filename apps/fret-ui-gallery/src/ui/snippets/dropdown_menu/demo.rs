// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::DropdownMenu::new_controllable(cx, None, false).into_element_parts(
        cx,
        |cx| {
            shadcn::DropdownMenuTrigger::new(
                shadcn::Button::new("Open menu")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-dropdown-menu-demo-trigger")
                    .into_element(cx),
            )
        },
        shadcn::DropdownMenuContent::new()
            .align(shadcn::DropdownMenuAlign::Start)
            .side_offset(Px(4.0)),
        |cx| {
            [
                shadcn::DropdownMenuGroup::new([
                    shadcn::DropdownMenuLabel::new("My Account").into(),
                    shadcn::DropdownMenuSeparator::new().into(),
                    shadcn::DropdownMenuItem::new("Profile")
                        .leading_icon(IconId::new_static("lucide.user"))
                        .trailing(shadcn::DropdownMenuShortcut::new("Cmd+P").into_element(cx))
                        .test_id("ui-gallery-dropdown-menu-demo-profile")
                        .into(),
                    shadcn::DropdownMenuItem::new("Billing")
                        .leading_icon(IconId::new_static("lucide.credit-card"))
                        .trailing(shadcn::DropdownMenuShortcut::new("Cmd+B").into_element(cx))
                        .into(),
                    shadcn::DropdownMenuItem::new("Settings")
                        .leading_icon(IconId::new_static("lucide.settings"))
                        .trailing(shadcn::DropdownMenuShortcut::new("Cmd+S").into_element(cx))
                        .into(),
                    shadcn::DropdownMenuSeparator::new().into(),
                    shadcn::DropdownMenuItem::new("More")
                        .leading_icon(IconId::new_static("lucide.more-horizontal"))
                        .submenu([
                            shadcn::DropdownMenuItem::new("Invite users")
                                .leading_icon(IconId::new_static("lucide.user-plus"))
                                .into(),
                            shadcn::DropdownMenuItem::new("Preferences")
                                .leading_icon(IconId::new_static("lucide.sliders-horizontal"))
                                .into(),
                        ])
                        .into(),
                ])
                .into(),
            ]
        },
    )
    .test_id("ui-gallery-dropdown-menu-demo")
}
// endregion: example

