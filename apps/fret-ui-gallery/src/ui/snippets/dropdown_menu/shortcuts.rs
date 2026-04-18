pub const SOURCE: &str = include_str!("shortcuts.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    super::preview_frame_with(cx, |cx| {
        shadcn::DropdownMenu::uncontrolled(cx)
            .compose()
            .trigger(
                shadcn::Button::new("Open")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-dropdown-menu-shortcuts-trigger"),
            )
            .content(
                shadcn::DropdownMenuContent::new()
                    .align(shadcn::DropdownMenuAlign::Start)
                    .side_offset(Px(4.0)),
            )
            .entries([
                shadcn::DropdownMenuGroup::new([
                    shadcn::DropdownMenuLabel::new("My Account").into(),
                    shadcn::DropdownMenuItem::new("Profile")
                        .shortcut("⇧⌘P")
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
                shadcn::DropdownMenuItem::new("Log out")
                    .shortcut("⇧⌘Q")
                    .into(),
            ])
    })
    .into_element(cx)
}
// endregion: example
