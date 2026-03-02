pub const SOURCE: &str = include_str!("shortcuts.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::DropdownMenu::new_controllable(cx, None, false)
        .into_element_parts(
            cx,
            |cx| {
                shadcn::DropdownMenuTrigger::new(
                    shadcn::Button::new("Shortcuts")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-dropdown-menu-shortcuts-trigger")
                        .into_element(cx),
                )
            },
            shadcn::DropdownMenuContent::new()
                .align(shadcn::DropdownMenuAlign::Start)
                .side_offset(Px(4.0)),
            |cx| {
                [
                    shadcn::DropdownMenuItem::new("Open file")
                        .trailing(shadcn::DropdownMenuShortcut::new("Cmd+O").into_element(cx))
                        .into(),
                    shadcn::DropdownMenuItem::new("Save file")
                        .trailing(shadcn::DropdownMenuShortcut::new("Cmd+S").into_element(cx))
                        .into(),
                    shadcn::DropdownMenuItem::new("Close tab")
                        .trailing(shadcn::DropdownMenuShortcut::new("Cmd+W").into_element(cx))
                        .into(),
                ]
            },
        )
        .test_id("ui-gallery-dropdown-menu-shortcuts")
}
// endregion: example
