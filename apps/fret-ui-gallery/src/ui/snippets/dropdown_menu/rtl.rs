pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    super::preview_frame_with(cx, |cx| {
        with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
            shadcn::DropdownMenu::uncontrolled(cx)
                .compose()
                .trigger(
                    shadcn::Button::new("RTL")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-dropdown-menu-rtl-trigger"),
                )
                .content(
                    shadcn::DropdownMenuContent::new()
                        .align(shadcn::DropdownMenuAlign::Start)
                        .side_offset(Px(4.0)),
                )
                .entries([
                    shadcn::DropdownMenuItem::new("Dashboard")
                        .test_id("ui-gallery-dropdown-menu-rtl-item-dashboard")
                        .into(),
                    shadcn::DropdownMenuItem::new("Settings")
                        .test_id("ui-gallery-dropdown-menu-rtl-item-settings")
                        .into(),
                ])
        })
    })
    .into_element(cx)
}
// endregion: example
