// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::DropdownMenu::new_controllable(cx, None, false)
        .into_element_parts(
            cx,
            |cx| {
                shadcn::DropdownMenuTrigger::new(
                    shadcn::Button::new("Parts")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-dropdown-menu-parts-trigger")
                        .into_element(cx),
                )
            },
            shadcn::DropdownMenuContent::new()
                .align(shadcn::DropdownMenuAlign::Start)
                .side_offset(Px(4.0)),
            |cx| {
                [
                    shadcn::DropdownMenuLabel::new("My Account").into(),
                    shadcn::DropdownMenuSeparator::new().into(),
                    shadcn::DropdownMenuItem::new("Profile")
                        .leading_icon(IconId::new_static("lucide.user"))
                        .trailing(shadcn::DropdownMenuShortcut::new("Cmd+P").into_element(cx))
                        .test_id("ui-gallery-dropdown-menu-parts-profile")
                        .into(),
                    shadcn::DropdownMenuSub::new(
                        shadcn::DropdownMenuSubTrigger::new("More tools").refine(|item| {
                            item.test_id("ui-gallery-dropdown-menu-parts-more")
                                .close_on_select(false)
                        }),
                        shadcn::DropdownMenuSubContent::new(vec![
                            shadcn::DropdownMenuItem::new("Rename")
                                .test_id("ui-gallery-dropdown-menu-parts-sub-rename")
                                .into(),
                            shadcn::DropdownMenuItem::new("Duplicate")
                                .test_id("ui-gallery-dropdown-menu-parts-sub-duplicate")
                                .into(),
                        ]),
                    )
                    .into(),
                ]
            },
        )
        .test_id("ui-gallery-dropdown-menu-parts")
}
// endregion: example
