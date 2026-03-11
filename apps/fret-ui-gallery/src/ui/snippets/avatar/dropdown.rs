pub const SOURCE: &str = include_str!("dropdown.rs");

// region: example
use fret_core::{Corners, ImageId, Px};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn wrap_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    fret_ui_kit::ui::h_flex(children)
        .gap(Space::N4)
        .wrap()
        .w_full()
        .items_center()
        .into_element(cx)
}

pub fn render<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    avatar_image: Model<Option<ImageId>>,
    open: Model<bool>,
) -> AnyElement {
    wrap_row(cx, |cx| {
        let avatar_image_for_trigger = avatar_image.clone();

        let entries = |_cx: &mut ElementContext<'_, H>| {
            vec![
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Profile")
                        .test_id("ui-gallery-avatar-dropdown-item-profile"),
                ),
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Billing")
                        .test_id("ui-gallery-avatar-dropdown-item-billing"),
                ),
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Settings")
                        .test_id("ui-gallery-avatar-dropdown-item-settings"),
                ),
                shadcn::DropdownMenuEntry::Separator,
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Log out")
                        .variant(
                            fret_ui_shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive,
                        )
                        .test_id("ui-gallery-avatar-dropdown-item-logout"),
                ),
            ]
        };

        vec![
            shadcn::DropdownMenu::new(open).into_element_parts(
                cx,
                move |cx| {
                    let image = shadcn::AvatarImage::model(avatar_image_for_trigger.clone())
                        .into_element(cx);
                    let fallback = shadcn::AvatarFallback::new("CN")
                        .when_image_missing_model(avatar_image_for_trigger.clone())
                        .delay_ms(120)
                        .into_element(cx);

                    let avatar = shadcn::Avatar::new([image, fallback])
                        .size(shadcn::AvatarSize::Default)
                        .into_element(cx);

                    let trigger = shadcn::Button::new("")
                        .variant(shadcn::ButtonVariant::Ghost)
                        .size(shadcn::ButtonSize::Icon)
                        .a11y_label("Open user menu")
                        .corner_radii_override(Corners::all(Px(999.0)))
                        .children([avatar])
                        .test_id("ui-gallery-avatar-dropdown-trigger-avatar")
                        .into_element(cx);

                    // shadcn/Radix parity: the authored child button is the actual trigger surface.
                    // The nested Avatar is presentational content inside that pressable child.
                    shadcn::DropdownMenuTrigger::new(trigger)
                },
                shadcn::DropdownMenuContent::new()
                    .align(shadcn::DropdownMenuAlign::End)
                    .side_offset(Px(4.0))
                    .min_width(Px(224.0)),
                entries,
            ),
        ]
    })
    .test_id("ui-gallery-avatar-dropdown-row")
}
// endregion: example
