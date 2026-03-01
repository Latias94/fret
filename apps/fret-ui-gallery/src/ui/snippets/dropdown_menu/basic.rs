// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::DropdownMenu::new_controllable(cx, None, false).into_element_parts(
        cx,
        |cx| {
            shadcn::DropdownMenuTrigger::new(
                shadcn::Button::new("Basic")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-dropdown-menu-basic-trigger")
                    .into_element(cx),
            )
        },
        shadcn::DropdownMenuContent::new()
            .align(shadcn::DropdownMenuAlign::Start)
            .side_offset(Px(4.0)),
        |_cx| {
            [
                shadcn::DropdownMenuGroup::new([
                    shadcn::DropdownMenuLabel::new("My Account").into(),
                    shadcn::DropdownMenuItem::new("Profile")
                        .test_id("ui-gallery-dropdown-menu-basic-profile")
                        .into(),
                    shadcn::DropdownMenuItem::new("Billing").into(),
                    shadcn::DropdownMenuSeparator::new().into(),
                ])
                .into(),
                shadcn::DropdownMenuGroup::new([
                    shadcn::DropdownMenuItem::new("Team").into(),
                    shadcn::DropdownMenuItem::new("Subscription").into(),
                ])
                .into(),
            ]
        },
    )
    .test_id("ui-gallery-dropdown-menu-basic")
}
// endregion: example

