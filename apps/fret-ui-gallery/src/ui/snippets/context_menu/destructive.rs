pub const SOURCE: &str = include_str!("destructive.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_runtime::CommandId;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn trigger_surface<H: UiHost>(
    label: &'static str,
    test_id: &'static str,
) -> impl IntoUiElement<H> + use<H> {
    shadcn::Button::new(label)
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
        .test_id(test_id)
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::ContextMenu::uncontrolled(cx)
        .content_test_id("ui-gallery-context-menu-destructive-content")
        .build(
            cx,
            trigger_surface(
                "Right click for destructive items",
                "ui-gallery-context-menu-destructive-trigger",
            ),
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
                            .variant(
                                fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                            )
                            .test_id("ui-gallery-context-menu-destructive-delete"),
                    ),
                ]
            },
        )
        .test_id("ui-gallery-context-menu-destructive")
}
// endregion: example
