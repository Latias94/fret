pub const SOURCE: &str = include_str!("avatar.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::DropdownMenu::new_controllable(cx, None, false)
        .into_element_parts(
            cx,
            |cx| {
                let avatar =
                    shadcn::Avatar::new([shadcn::AvatarFallback::new("JD").into_element(cx)])
                        .refine_layout(LayoutRefinement::default().w_px(Px(36.0)).h_px(Px(36.0)))
                        .into_element(cx)
                        .test_id("ui-gallery-dropdown-menu-avatar-trigger");
                shadcn::DropdownMenuTrigger::new(avatar)
            },
            shadcn::DropdownMenuContent::new()
                .align(shadcn::DropdownMenuAlign::End)
                .side_offset(Px(4.0))
                .min_width(Px(220.0)),
            |_cx| {
                [
                    shadcn::DropdownMenuLabel::new("john@fret.dev").into(),
                    shadcn::DropdownMenuSeparator::new().into(),
                    shadcn::DropdownMenuItem::new("Profile").into(),
                    shadcn::DropdownMenuItem::new("Log out")
                        .variant(shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive)
                        .into(),
                ]
            },
        )
        .test_id("ui-gallery-dropdown-menu-avatar")
}
// endregion: example
