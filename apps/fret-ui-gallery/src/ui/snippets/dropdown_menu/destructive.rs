// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::DropdownMenu::new_controllable(cx, None, false).into_element_parts(
        cx,
        |cx| {
            shadcn::DropdownMenuTrigger::new(
                shadcn::Button::new("Destructive")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-dropdown-menu-destructive-trigger")
                    .into_element(cx),
            )
        },
        shadcn::DropdownMenuContent::new()
            .align(shadcn::DropdownMenuAlign::Start)
            .side_offset(Px(4.0)),
        |_cx| {
            [
                shadcn::DropdownMenuItem::new("Rename").into(),
                shadcn::DropdownMenuItem::new("Delete")
                    .variant(shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive)
                    .into(),
            ]
        },
    )
    .test_id("ui-gallery-dropdown-menu-destructive")
}
// endregion: example

