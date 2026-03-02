pub const SOURCE: &str = include_str!("dropdown.rs");

// region: example
use fret_core::ImageId;
use fret_ui_shadcn::{self as shadcn, prelude::*};

fn wrap_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    fret_ui_kit::ui::h_flex(cx, children)
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
        let trigger = move |cx: &mut ElementContext<'_, H>| {
            let image =
                shadcn::AvatarImage::model(avatar_image_for_trigger.clone()).into_element(cx);
            let fallback = shadcn::AvatarFallback::new("CN")
                .when_image_missing_model(avatar_image_for_trigger.clone())
                .delay_ms(120)
                .into_element(cx);

            let avatar = shadcn::Avatar::new([image, fallback])
                .size(shadcn::AvatarSize::Default)
                .into_element(cx);

            // Match shadcn docs: Avatar is composed inside a ghost icon button used as the
            // dropdown trigger (`asChild`-style).
            shadcn::Button::new("")
                .variant(shadcn::ButtonVariant::Ghost)
                .size(shadcn::ButtonSize::Icon)
                .a11y_label("Open user menu")
                .refine_style(ChromeRefinement::default().rounded(Radius::Full))
                .children([avatar])
                .test_id("ui-gallery-avatar-dropdown-trigger")
                .into_element(cx)
        };

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
                        .variant(shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive)
                        .test_id("ui-gallery-avatar-dropdown-item-logout"),
                ),
            ]
        };

        vec![shadcn::DropdownMenu::new(open).into_element(cx, trigger, entries)]
    })
    .test_id("ui-gallery-avatar-dropdown-row")
}
// endregion: example
