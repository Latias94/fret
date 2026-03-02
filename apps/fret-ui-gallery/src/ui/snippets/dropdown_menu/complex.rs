pub const SOURCE: &str = include_str!("complex.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::DropdownMenu::new_controllable(cx, None, false)
        .arrow(true)
        .into_element_parts(
            cx,
            |cx| {
                shadcn::DropdownMenuTrigger::new(
                    shadcn::Button::new("Complex")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-dropdown-menu-complex-trigger")
                        .into_element(cx),
                )
            },
            shadcn::DropdownMenuContent::new()
                .align(shadcn::DropdownMenuAlign::Start)
                .side_offset(Px(4.0)),
            |_cx| {
                [
                    shadcn::DropdownMenuLabel::new("Actions").into(),
                    shadcn::DropdownMenuSeparator::new().into(),
                    shadcn::DropdownMenuItem::new("Open")
                        .leading_icon(IconId::new_static("lucide.folder-open"))
                        .into(),
                    shadcn::DropdownMenuItem::new("Share")
                        .leading_icon(IconId::new_static("lucide.share-2"))
                        .submenu([
                            shadcn::DropdownMenuItem::new("Invite").into(),
                            shadcn::DropdownMenuItem::new("Native share sheet").into(),
                        ])
                        .into(),
                    shadcn::DropdownMenuItem::new("Delete")
                        .variant(shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive)
                        .leading_icon(IconId::new_static("lucide.trash-2"))
                        .into(),
                ]
            },
        )
        .test_id("ui-gallery-dropdown-menu-complex")
}
// endregion: example
