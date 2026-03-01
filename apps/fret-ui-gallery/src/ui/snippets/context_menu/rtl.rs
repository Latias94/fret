// region: example
use fret_ui_kit::primitives::direction::{LayoutDirection, with_direction_provider};
use fret_ui_shadcn::{self as shadcn, prelude::*};

fn trigger_surface<H: UiHost>(cx: &mut ElementContext<'_, H>, label: &'static str) -> AnyElement {
    shadcn::Button::new(label)
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
        .into_element(cx)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        shadcn::ContextMenu::new_controllable(cx, None, false)
            .content_test_id("ui-gallery-context-menu-rtl-content")
            .into_element(
                cx,
                |cx| {
                    trigger_surface(cx, "Right click in RTL")
                        .test_id("ui-gallery-context-menu-rtl-trigger")
                },
                |_cx| {
                    vec![
                        shadcn::ContextMenuEntry::Item(shadcn::ContextMenuItem::new("Open")),
                        shadcn::ContextMenuEntry::Item(shadcn::ContextMenuItem::new("Settings")),
                        shadcn::ContextMenuEntry::Separator,
                        shadcn::ContextMenuEntry::Item(
                            shadcn::ContextMenuItem::new("Delete")
                                .variant(shadcn::context_menu::ContextMenuItemVariant::Destructive),
                        ),
                    ]
                },
            )
    })
    .test_id("ui-gallery-context-menu-rtl")
}
// endregion: example
