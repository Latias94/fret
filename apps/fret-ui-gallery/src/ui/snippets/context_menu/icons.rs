pub const SOURCE: &str = include_str!("icons.rs");

// region: example
use fret_runtime::CommandId;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn trigger_surface<H: UiHost>(label: &'static str) -> impl IntoUiElement<H> + use<H> {
    shadcn::Button::new(label)
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::ContextMenu::new_controllable(cx, None, false)
        .content_test_id("ui-gallery-context-menu-icons-content")
        .into_element(
            cx,
            |cx| {
                trigger_surface("Right click for icons")
                    .into_element(cx)
                    .test_id("ui-gallery-context-menu-icons-trigger")
            },
            |_cx| {
                vec![
                    shadcn::ContextMenuEntry::Item(
                        shadcn::ContextMenuItem::new("Profile")
                            .action(CommandId::new("ui_gallery.context_menu.icons.profile"))
                            .leading_icon(IconId::new_static("lucide.user"))
                            .test_id("ui-gallery-context-menu-icons-profile"),
                    ),
                    shadcn::ContextMenuEntry::Item(
                        shadcn::ContextMenuItem::new("Settings")
                            .action(CommandId::new("ui_gallery.context_menu.icons.settings"))
                            .leading_icon(IconId::new_static("lucide.settings"))
                            .test_id("ui-gallery-context-menu-icons-settings"),
                    ),
                    shadcn::ContextMenuEntry::Separator,
                    shadcn::ContextMenuEntry::Item(
                        shadcn::ContextMenuItem::new("Download")
                            .action(CommandId::new("ui_gallery.context_menu.icons.download"))
                            .leading_icon(IconId::new_static("lucide.download"))
                            .test_id("ui-gallery-context-menu-icons-download"),
                    ),
                ]
            },
        )
        .test_id("ui-gallery-context-menu-icons")
}
// endregion: example
