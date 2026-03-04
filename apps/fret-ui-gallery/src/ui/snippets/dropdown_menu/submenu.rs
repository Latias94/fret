pub const SOURCE: &str = include_str!("submenu.rs");

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
                        .test_id("ui-gallery-dropdown-menu-submenu-trigger")
                        .into_element(cx),
                )
            },
            shadcn::DropdownMenuContent::new()
                .align(shadcn::DropdownMenuAlign::Start)
                .side_offset(Px(4.0)),
            |cx| {
                [shadcn::DropdownMenuGroup::new([
                    shadcn::DropdownMenuItem::new("Team").into(),
                    shadcn::DropdownMenuSub::new(
                        shadcn::DropdownMenuSubTrigger::new("Invite users"),
                        shadcn::DropdownMenuSubContent::new([
                            shadcn::DropdownMenuItem::new("Email").into(),
                            shadcn::DropdownMenuItem::new("Message").into(),
                            shadcn::DropdownMenuSub::new(
                                shadcn::DropdownMenuSubTrigger::new("More options"),
                                shadcn::DropdownMenuSubContent::new([
                                    shadcn::DropdownMenuItem::new("Calendly").into(),
                                    shadcn::DropdownMenuItem::new("Slack").into(),
                                    shadcn::DropdownMenuSeparator::new().into(),
                                    shadcn::DropdownMenuItem::new("Webhook").into(),
                                ]),
                            )
                            .into(),
                            shadcn::DropdownMenuSeparator::new().into(),
                            shadcn::DropdownMenuItem::new("Advanced...").into(),
                        ]),
                    )
                    .into(),
                    shadcn::DropdownMenuItem::new("New Team")
                        .trailing(shadcn::DropdownMenuShortcut::new("⌘+T").into_element(cx))
                        .into(),
                ])
                .into()]
            },
        )
        .test_id("ui-gallery-dropdown-menu-submenu")
}
// endregion: example
