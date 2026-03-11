pub const SOURCE: &str = include_str!("shortcuts.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::DropdownMenu::new_controllable(cx, None, false).build_parts(
        cx,
        shadcn::DropdownMenuTrigger::build(
            shadcn::Button::new("Open")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-dropdown-menu-shortcuts-trigger"),
        ),
        shadcn::DropdownMenuContent::new()
            .align(shadcn::DropdownMenuAlign::Start)
            .side_offset(Px(4.0)),
        |cx| {
            [
                shadcn::DropdownMenuGroup::new([
                    shadcn::DropdownMenuLabel::new("My Account").into(),
                    shadcn::DropdownMenuItem::new("Profile")
                        .trailing(shadcn::DropdownMenuShortcut::new("⇧⌘P").into_element(cx))
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
                shadcn::DropdownMenuItem::new("Log out")
                    .trailing(shadcn::DropdownMenuShortcut::new("⇧⌘Q").into_element(cx))
                    .into(),
            ]
        },
    )
}
// endregion: example
