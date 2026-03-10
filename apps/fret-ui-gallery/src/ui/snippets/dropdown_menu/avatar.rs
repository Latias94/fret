pub const SOURCE: &str = include_str!("avatar.rs");

// region: example
use fret_core::{Corners, Px};
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let avatar =
        shadcn::Avatar::new([shadcn::AvatarFallback::new("LR").into_element(cx)]).into_element(cx);
    let trigger = shadcn::DropdownMenuTrigger::build(
        shadcn::Button::new("")
            .variant(shadcn::ButtonVariant::Ghost)
            .size(shadcn::ButtonSize::Icon)
            .a11y_label("Account")
            .corner_radii_override(Corners::all(Px(999.0)))
            .children([avatar])
            .test_id("ui-gallery-dropdown-menu-avatar-trigger"),
    );

    shadcn::DropdownMenu::new_controllable(cx, None, false).build_parts(
        cx,
        trigger,
        shadcn::DropdownMenuContent::new()
            .align(shadcn::DropdownMenuAlign::End)
            .side_offset(Px(4.0))
            .min_width(Px(224.0)),
        |_cx| {
            [
                shadcn::DropdownMenuGroup::new([
                    shadcn::DropdownMenuItem::new("Account")
                        .leading_icon(IconId::new_static("lucide.badge-check"))
                        .into(),
                    shadcn::DropdownMenuItem::new("Billing")
                        .leading_icon(IconId::new_static("lucide.credit-card"))
                        .into(),
                    shadcn::DropdownMenuItem::new("Notifications")
                        .leading_icon(IconId::new_static("lucide.bell"))
                        .into(),
                ])
                .into(),
                shadcn::DropdownMenuSeparator::new().into(),
                shadcn::DropdownMenuItem::new("Sign Out")
                    .leading_icon(IconId::new_static("lucide.log-out"))
                    .into(),
            ]
        },
    )
}
// endregion: example
