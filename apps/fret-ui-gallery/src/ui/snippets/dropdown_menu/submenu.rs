// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::DropdownMenu::new_controllable(cx, None, false).into_element_parts(
        cx,
        |cx| {
            shadcn::DropdownMenuTrigger::new(
                shadcn::Button::new("Submenu")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-dropdown-menu-submenu-trigger")
                    .into_element(cx),
            )
        },
        shadcn::DropdownMenuContent::new()
            .align(shadcn::DropdownMenuAlign::Start)
            .side_offset(Px(4.0)),
        |_cx| {
            [
                shadcn::DropdownMenuItem::new("Open").into(),
                shadcn::DropdownMenuItem::new("More tools")
                    .submenu([
                        shadcn::DropdownMenuItem::new("Rename").into(),
                        shadcn::DropdownMenuItem::new("Duplicate").into(),
                    ])
                    .into(),
            ]
        },
    )
    .test_id("ui-gallery-dropdown-menu-submenu")
}
// endregion: example

