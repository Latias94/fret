pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_core::Px;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    cx.keyed("ui_gallery.dropdown_menu.demo", |cx| {
        super::preview_frame_with(cx, |cx| {
            shadcn::DropdownMenu::new_controllable(cx, None, false).build_parts(
                cx,
                shadcn::DropdownMenuTrigger::build(
                    shadcn::Button::new("Open")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-dropdown-menu-demo-trigger"),
                ),
                shadcn::DropdownMenuContent::new()
                    .align(shadcn::DropdownMenuAlign::Start)
                    .side_offset(Px(4.0))
                    // new-york-v4 dropdown-menu-demo: `DropdownMenuContent className="w-56"`.
                    .min_width(Px(224.0)),
                |_cx| {
                    [
                        shadcn::DropdownMenuGroup::new([
                            shadcn::DropdownMenuLabel::new("My Account").into(),
                            shadcn::DropdownMenuItem::new("Profile")
                                .shortcut("⇧⌘P")
                                .test_id("ui-gallery-dropdown-menu-demo-profile")
                                // ui-gallery diag scripts: assert this updates the global status bar.
                                .action("ui_gallery.menu.dropdown.apple")
                                .into(),
                            shadcn::DropdownMenuItem::new("Billing")
                                .shortcut("⌘B")
                                .into(),
                            shadcn::DropdownMenuItem::new("Settings")
                                .shortcut("⌘S")
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
                                .shortcut("⌘+T")
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
                            .shortcut("⇧⌘Q")
                            .into()])
                        .into(),
                    ]
                },
            )
        })
        .into_element(cx)
    })
}
// endregion: example
