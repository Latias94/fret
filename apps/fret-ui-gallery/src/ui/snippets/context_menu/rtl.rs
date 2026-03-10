pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_runtime::CommandId;
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
                |cx| {
                    vec![
                        shadcn::ContextMenuEntry::Item(
                            shadcn::ContextMenuItem::new("Open")
                                .action(CommandId::new("ui_gallery.context_menu.rtl.open"))
                                .test_id("ui-gallery-context-menu-rtl-item-open")
                                .trailing(shadcn::ContextMenuShortcut::new("⌘O").into_element(cx)),
                        ),
                        shadcn::ContextMenuEntry::Item(
                            shadcn::ContextMenuItem::new("Settings")
                                .action(CommandId::new("ui_gallery.context_menu.rtl.settings"))
                                .test_id("ui-gallery-context-menu-rtl-item-settings")
                                .trailing(shadcn::ContextMenuShortcut::new("⌘,").into_element(cx)),
                        ),
                        shadcn::ContextMenuEntry::Separator,
                        shadcn::ContextMenuEntry::Item(
                            shadcn::ContextMenuItem::new("More")
                                .test_id("ui-gallery-context-menu-rtl-item-more")
                                .submenu([
                                    shadcn::ContextMenuEntry::Item(
                                        shadcn::ContextMenuItem::new("Sub Alpha")
                                            .action(CommandId::new(
                                                "ui_gallery.context_menu.rtl.sub_alpha",
                                            ))
                                            .test_id("ui-gallery-context-menu-rtl-item-sub-alpha"),
                                    ),
                                    shadcn::ContextMenuEntry::Item(
                                        shadcn::ContextMenuItem::new("Sub Beta")
                                            .action(CommandId::new(
                                                "ui_gallery.context_menu.rtl.sub_beta",
                                            ))
                                            .test_id("ui-gallery-context-menu-rtl-item-sub-beta"),
                                    ),
                                ]),
                        ),
                        shadcn::ContextMenuEntry::Separator,
                        shadcn::ContextMenuEntry::Item(
                            shadcn::ContextMenuItem::new("Delete")
                                .action(CommandId::new("ui_gallery.context_menu.rtl.delete"))
                                .test_id("ui-gallery-context-menu-rtl-item-delete")
                                .variant(shadcn::context_menu::ContextMenuItemVariant::Destructive),
                        ),
                    ]
                },
            )
    })
    .test_id("ui-gallery-context-menu-rtl")
}
// endregion: example
