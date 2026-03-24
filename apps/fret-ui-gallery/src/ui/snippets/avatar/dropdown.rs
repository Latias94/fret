pub const SOURCE: &str = include_str!("dropdown.rs");

// region: example
use super::demo_image;
use fret::{UiChild, UiCx};
use fret_core::{Corners, Px};
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn wrap_row<H: UiHost, F>(children: F) -> impl IntoUiElement<H> + use<H, F>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
{
    fret_ui_kit::ui::h_flex(children)
        .gap(Space::N4)
        .wrap()
        .w_full()
        .items_center()
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let avatar_image = demo_image(cx);
    let open = cx.local_model_keyed("open", || false);

    wrap_row(|cx| {
        let avatar_image_for_trigger = avatar_image;

        let entries = |_cx: &mut UiCx<'_>| {
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
                        .variant(shadcn::raw::dropdown_menu::DropdownMenuItemVariant::Destructive)
                        .test_id("ui-gallery-avatar-dropdown-item-logout"),
                ),
            ]
        };

        let image = shadcn::AvatarImage::maybe(avatar_image_for_trigger).into_element(cx);
        let fallback = shadcn::AvatarFallback::new("CN")
            .when_image_missing(avatar_image_for_trigger)
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

        vec![
            shadcn::DropdownMenu::from_open(open)
                .compose()
                // shadcn/Radix parity: the authored child button is the actual trigger surface.
                // The nested Avatar is presentational content inside that pressable child.
                .trigger(shadcn::DropdownMenuTrigger::new(trigger))
                .content(
                    shadcn::DropdownMenuContent::new()
                        .align(shadcn::DropdownMenuAlign::End)
                        .side_offset(Px(4.0))
                        .min_width(Px(128.0)),
                )
                .entries(entries(cx))
                .into_element(cx),
        ]
    })
    .into_element(cx)
    .test_id("ui-gallery-avatar-dropdown-row")
}
// endregion: example
