use super::super::super::ChildRegionChrome;
use crate::ui::ScrollAreaBoxBuild;
use crate::ui_builder::UiBuilder;

pub(in crate::imui::child_region) fn apply_child_region_scroll_chrome<H, B>(
    builder: UiBuilder<ScrollAreaBoxBuild<H, B>>,
    chrome: ChildRegionChrome,
) -> UiBuilder<ScrollAreaBoxBuild<H, B>> {
    if chrome != ChildRegionChrome::Framed {
        return builder;
    }

    builder
        .p_2()
        .rounded_md()
        .border_1()
        .bg(crate::ColorRef::Token {
            key: "card",
            fallback: crate::ColorFallback::ThemePanelBackground,
        })
        .border_color(crate::ColorRef::Token {
            key: "border",
            fallback: crate::ColorFallback::ThemePanelBorder,
        })
}
