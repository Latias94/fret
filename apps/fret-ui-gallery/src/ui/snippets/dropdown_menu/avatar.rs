pub const SOURCE: &str = include_str!("avatar.rs");

// region: example
use super::super::avatar::demo_image;
use fret::{UiChild, UiCx};
use fret_core::{Corners, Px};
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let avatar_image = demo_image(cx);
    let avatar = shadcn::Avatar::new([
        shadcn::AvatarImage::maybe(avatar_image).into_element(cx),
        shadcn::AvatarFallback::new("LR")
            .when_image_missing(avatar_image)
            .delay_ms(120)
            .into_element(cx),
    ])
    .into_element(cx);
    let trigger = shadcn::DropdownMenuTrigger::build(
        shadcn::Button::new("")
            .variant(shadcn::ButtonVariant::Ghost)
            .size(shadcn::ButtonSize::Icon)
            .a11y_label("Account")
            .corner_radii_override(Corners::all(Px(999.0)))
            .children([avatar])
            .test_id("ui-gallery-dropdown-menu-avatar-trigger"),
    );

    super::preview_frame_with(cx, move |cx| {
        shadcn::DropdownMenu::uncontrolled(cx)
            .compose()
            .trigger(trigger)
            .content(
                shadcn::DropdownMenuContent::new()
                    .align(shadcn::DropdownMenuAlign::End)
                    .side_offset(Px(4.0)),
            )
            .entries([
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
            ])
    })
    .into_element(cx)
}
// endregion: example
