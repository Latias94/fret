pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_core::Px;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    super::preview_frame_with(cx, |cx| {
        with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
            shadcn::DropdownMenu::new_controllable(cx, None, false).build_parts(
                cx,
                shadcn::DropdownMenuTrigger::build(
                    shadcn::Button::new("RTL")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-dropdown-menu-rtl-trigger"),
                ),
                shadcn::DropdownMenuContent::new()
                    .align(shadcn::DropdownMenuAlign::Start)
                    .side_offset(Px(4.0)),
                |_cx| {
                    [
                        shadcn::DropdownMenuItem::new("Dashboard")
                            .test_id("ui-gallery-dropdown-menu-rtl-item-dashboard")
                            .into(),
                        shadcn::DropdownMenuItem::new("Settings")
                            .test_id("ui-gallery-dropdown-menu-rtl-item-settings")
                            .into(),
                    ]
                },
            )
        })
    })
    .into_element(cx)
}
// endregion: example
