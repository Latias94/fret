// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

fn trigger_surface<H: UiHost>(cx: &mut ElementContext<'_, H>, label: &'static str) -> AnyElement {
    shadcn::Button::new(label)
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
        .into_element(cx)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::ContextMenu::new_controllable(cx, None, false)
        .content_test_id("ui-gallery-context-menu-submenu-content")
        .into_element(
            cx,
            |cx| {
                trigger_surface(cx, "Right click for submenu")
                    .test_id("ui-gallery-context-menu-submenu-trigger")
            },
            |_cx| {
                vec![
                    shadcn::ContextMenuEntry::Item(
                        shadcn::ContextMenuItem::new("Open")
                            .test_id("ui-gallery-context-menu-submenu-open"),
                    ),
                    shadcn::ContextMenuEntry::Item(
                        shadcn::ContextMenuItem::new("More tools")
                            .test_id("ui-gallery-context-menu-submenu-more-tools")
                            .submenu(vec![
                                shadcn::ContextMenuEntry::Item(
                                    shadcn::ContextMenuItem::new("Rename")
                                        .test_id("ui-gallery-context-menu-submenu-rename"),
                                ),
                                shadcn::ContextMenuEntry::Item(
                                    shadcn::ContextMenuItem::new("Duplicate")
                                        .test_id("ui-gallery-context-menu-submenu-duplicate"),
                                ),
                            ]),
                    ),
                ]
            },
        )
        .test_id("ui-gallery-context-menu-submenu")
}
// endregion: example
