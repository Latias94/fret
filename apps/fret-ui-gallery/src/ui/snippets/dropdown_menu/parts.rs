pub const SOURCE: &str = include_str!("parts.rs");

// region: example
use fret_core::Px;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    super::preview_frame_with(cx, |cx| {
        shadcn::DropdownMenu::new_controllable(cx, None, false).build_parts(
            cx,
            shadcn::DropdownMenuTrigger::build(
                shadcn::Button::new("Parts")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-dropdown-menu-parts-trigger"),
            ),
            shadcn::DropdownMenuContent::new()
                .align(shadcn::DropdownMenuAlign::Start)
                .side_offset(Px(4.0)),
            |_cx| {
                [
                    shadcn::DropdownMenuLabel::new("My Account").into(),
                    shadcn::DropdownMenuSeparator::new().into(),
                    shadcn::DropdownMenuItem::new("Profile")
                        .leading_icon(IconId::new_static("lucide.user"))
                        .shortcut("Cmd+P")
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
    })
    .into_element(cx)
}
// endregion: example
