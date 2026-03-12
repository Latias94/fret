pub const SOURCE: &str = include_str!("basic.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    cx.keyed("ui_gallery.dropdown_menu.basic", |cx| {
        super::preview_frame_with(cx, |cx| {
            shadcn::DropdownMenu::new_controllable(cx, None, false).build_parts(
                cx,
                shadcn::DropdownMenuTrigger::build(
                    shadcn::Button::new("Open")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-dropdown-menu-basic-trigger"),
                ),
                shadcn::DropdownMenuContent::new()
                    .align(shadcn::DropdownMenuAlign::Start)
                    .side_offset(Px(4.0)),
                |_cx| {
                    [
                        shadcn::DropdownMenuGroup::new([
                            shadcn::DropdownMenuLabel::new("My Account").into(),
                            shadcn::DropdownMenuItem::new("Profile")
                                .test_id("ui-gallery-dropdown-menu-basic-profile")
                                .action("ui_gallery.menu.dropdown.apple")
                                .into(),
                            shadcn::DropdownMenuItem::new("Billing")
                                // ui-gallery diag scripts: typeahead targets this item via `b` + Enter.
                                .test_id("ui-gallery-dropdown-menu-basic-billing")
                                .action("ui_gallery.menu.dropdown.orange")
                                .into(),
                            shadcn::DropdownMenuItem::new("Settings").into(),
                        ])
                        .into(),
                        shadcn::DropdownMenuSeparator::new().into(),
                        shadcn::DropdownMenuItem::new("GitHub").into(),
                        shadcn::DropdownMenuItem::new("Support").into(),
                        shadcn::DropdownMenuItem::new("API").disabled(true).into(),
                    ]
                },
            )
        })
    })
}
// endregion: example
