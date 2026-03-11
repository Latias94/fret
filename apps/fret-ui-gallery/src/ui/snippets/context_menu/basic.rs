pub const SOURCE: &str = include_str!("basic.rs");

// region: example
use fret_runtime::CommandId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn trigger_surface<H: UiHost>(cx: &mut ElementContext<'_, H>, label: &'static str) -> AnyElement {
    shadcn::Button::new(label)
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
        .into_element(cx)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::ContextMenu::new_controllable(cx, None, false)
        .content_test_id("ui-gallery-context-menu-basic-content")
        .into_element(
            cx,
            |cx| {
                trigger_surface(cx, "Right click for actions")
                    .test_id("ui-gallery-context-menu-basic-trigger")
            },
            |_cx| {
                vec![
                    shadcn::ContextMenuEntry::Item(
                        shadcn::ContextMenuItem::new("Profile")
                            .action(CommandId::new("ui_gallery.context_menu.basic.profile"))
                            .test_id("ui-gallery-context-menu-basic-profile"),
                    ),
                    shadcn::ContextMenuEntry::Item(
                        shadcn::ContextMenuItem::new("Billing")
                            .action(CommandId::new("ui_gallery.context_menu.basic.billing")),
                    ),
                    shadcn::ContextMenuEntry::Item(
                        shadcn::ContextMenuItem::new("Team")
                            .action(CommandId::new("ui_gallery.context_menu.basic.team")),
                    ),
                    shadcn::ContextMenuEntry::Separator,
                    shadcn::ContextMenuEntry::Item(
                        shadcn::ContextMenuItem::new("Subscription")
                            .action(CommandId::new("ui_gallery.context_menu.basic.subscription")),
                    ),
                ]
            },
        )
        .test_id("ui-gallery-context-menu-basic")
}
// endregion: example
