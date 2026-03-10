pub const SOURCE: &str = include_str!("shortcuts.rs");

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
    shadcn::ContextMenu::new_controllable(cx, None, false)
        .content_test_id("ui-gallery-context-menu-shortcuts-content")
        .into_element(
            cx,
            |cx| {
                trigger_surface(cx, "Right click for shortcuts")
                    .test_id("ui-gallery-context-menu-shortcuts-trigger")
            },
            |cx| {
                vec![
                    shadcn::ContextMenuEntry::Item(
                        shadcn::ContextMenuItem::new("Open file")
                            .action(CommandId::new("ui_gallery.context_menu.shortcuts.open_file"))
                            .trailing(shadcn::ContextMenuShortcut::new("Cmd+O").into_element(cx))
                            .test_id("ui-gallery-context-menu-shortcuts-open-file"),
                    ),
                    shadcn::ContextMenuEntry::Item(
                        shadcn::ContextMenuItem::new("Save file")
                            .action(CommandId::new("ui_gallery.context_menu.shortcuts.save_file"))
                            .trailing(shadcn::ContextMenuShortcut::new("Cmd+S").into_element(cx)),
                    ),
                    shadcn::ContextMenuEntry::Item(
                        shadcn::ContextMenuItem::new("Close tab")
                            .action(CommandId::new("ui_gallery.context_menu.shortcuts.close_tab"))
                            .trailing(shadcn::ContextMenuShortcut::new("Cmd+W").into_element(cx)),
                    ),
                ]
            },
        )
        .test_id("ui-gallery-context-menu-shortcuts")
}
// endregion: example
