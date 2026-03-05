pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    cx.keyed("ui_gallery.dropdown_menu.demo", |cx| {
        shadcn::DropdownMenu::new_controllable(cx, None, false).into_element_parts(
            cx,
            |cx| {
                shadcn::DropdownMenuTrigger::new(
                    shadcn::Button::new("Open")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-dropdown-menu-demo-trigger")
                        .into_element(cx),
                )
            },
            shadcn::DropdownMenuContent::new()
                .align(shadcn::DropdownMenuAlign::Start)
                .side_offset(Px(4.0))
                // shadcn/ui docs: `DropdownMenuContent className="w-40"`.
                .min_width(Px(160.0)),
            |cx| {
                [
                    shadcn::DropdownMenuGroup::new([
                        shadcn::DropdownMenuLabel::new("My Account").into(),
                        shadcn::DropdownMenuItem::new("Profile")
                            .trailing(shadcn::DropdownMenuShortcut::new("⇧⌘P").into_element(cx))
                            .test_id("ui-gallery-dropdown-menu-demo-profile")
                            // ui-gallery diag scripts: assert this updates the global status bar.
                            .on_select("ui_gallery.menu.dropdown.apple")
                            .into(),
                        shadcn::DropdownMenuItem::new("Billing")
                            .trailing(shadcn::DropdownMenuShortcut::new("⌘B").into_element(cx))
                            .into(),
                        shadcn::DropdownMenuItem::new("Settings")
                            .trailing(shadcn::DropdownMenuShortcut::new("⌘S").into_element(cx))
                            .into(),
                    ])
                    .into(),
                    shadcn::DropdownMenuSeparator::new().into(),
                    shadcn::DropdownMenuGroup::new([
                        shadcn::DropdownMenuItem::new("Team").into(),
                        shadcn::DropdownMenuSub::new(
                            shadcn::DropdownMenuSubTrigger::new("Invite users"),
                            shadcn::DropdownMenuSubContent::new([
                                shadcn::DropdownMenuItem::new("Email").into(),
                                shadcn::DropdownMenuItem::new("Message").into(),
                                shadcn::DropdownMenuSeparator::new().into(),
                                shadcn::DropdownMenuItem::new("More...").into(),
                            ]),
                        )
                        .into(),
                        shadcn::DropdownMenuItem::new("New Team")
                            .trailing(shadcn::DropdownMenuShortcut::new("⌘+T").into_element(cx))
                            .into(),
                    ])
                    .into(),
                    shadcn::DropdownMenuSeparator::new().into(),
                    shadcn::DropdownMenuGroup::new([
                        shadcn::DropdownMenuItem::new("GitHub").into(),
                        shadcn::DropdownMenuItem::new("Support").into(),
                        shadcn::DropdownMenuItem::new("API").disabled(true).into(),
                    ])
                    .into(),
                    shadcn::DropdownMenuSeparator::new().into(),
                    shadcn::DropdownMenuGroup::new([shadcn::DropdownMenuItem::new("Log out")
                        .trailing(shadcn::DropdownMenuShortcut::new("⇧⌘Q").into_element(cx))
                        .into()])
                    .into(),
                ]
            },
        )
    })
}
// endregion: example
