pub const SOURCE: &str = include_str!("submenu.rs");

// region: example
use fret_core::Px;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    super::preview_frame_with(cx, |cx| {
        shadcn::DropdownMenu::uncontrolled(cx).build_parts(
            cx,
            shadcn::DropdownMenuTrigger::build(
                shadcn::Button::new("Open")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-dropdown-menu-submenu-trigger"),
            ),
            shadcn::DropdownMenuContent::new()
                .align(shadcn::DropdownMenuAlign::Start)
                .side_offset(Px(4.0)),
            |_cx| {
                [shadcn::DropdownMenuGroup::new([
                    shadcn::DropdownMenuItem::new("Team").into(),
                    shadcn::DropdownMenuSub::new(
                        shadcn::DropdownMenuSubTrigger::new("Invite users").refine(|item| {
                            item.test_id("ui-gallery-dropdown-menu-submenu-invite-users")
                        }),
                        shadcn::DropdownMenuSubContent::new([
                            shadcn::DropdownMenuItem::new("Email")
                                .test_id("ui-gallery-dropdown-menu-submenu-email")
                                .into(),
                            shadcn::DropdownMenuItem::new("Message").into(),
                            shadcn::DropdownMenuSub::new(
                                shadcn::DropdownMenuSubTrigger::new("More options").refine(
                                    |item| {
                                        item.test_id(
                                            "ui-gallery-dropdown-menu-submenu-more-options",
                                        )
                                    },
                                ),
                                shadcn::DropdownMenuSubContent::new([
                                    shadcn::DropdownMenuItem::new("Calendly")
                                        .test_id("ui-gallery-dropdown-menu-submenu-calendly")
                                        .into(),
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
                        .shortcut("⌘+T")
                        .into(),
                ])
                .into()]
            },
        )
    })
    .into_element(cx)
}
// endregion: example
