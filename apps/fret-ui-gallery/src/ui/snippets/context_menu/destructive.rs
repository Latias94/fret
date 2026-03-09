pub const SOURCE: &str = include_str!("destructive.rs");

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
        .content_test_id("ui-gallery-context-menu-destructive-content")
        .into_element(
            cx,
            |cx| {
                trigger_surface(cx, "Right click for destructive items")
                    .test_id("ui-gallery-context-menu-destructive-trigger")
            },
            |_cx| {
                vec![
                    shadcn::ContextMenuEntry::Item(
                        shadcn::ContextMenuItem::new("Rename")
                            .action(CommandId::new("ui_gallery.context_menu.destructive.rename")),
                    ),
                    shadcn::ContextMenuEntry::Separator,
                    shadcn::ContextMenuEntry::Item(
                        shadcn::ContextMenuItem::new("Delete project")
                            .action(CommandId::new(
                                "ui_gallery.context_menu.destructive.delete_project",
                            ))
                            .variant(shadcn::context_menu::ContextMenuItemVariant::Destructive)
                            .test_id("ui-gallery-context-menu-destructive-delete"),
                    ),
                ]
            },
        )
        .test_id("ui-gallery-context-menu-destructive")
}
// endregion: example
