pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_runtime::CommandId;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::ContextMenu::new_controllable(cx, None, false).into_element_parts(
        cx,
        |cx| {
            shadcn::ContextMenuTrigger::new(
                shadcn::Button::new("Right click")
                    .variant(shadcn::ButtonVariant::Outline)
                    .into_element(cx),
            )
        },
        shadcn::ContextMenuContent::new(),
        |_cx| {
            vec![
                shadcn::ContextMenuEntry::Item(
                    shadcn::ContextMenuItem::new("Profile")
                        .action(CommandId::new("ui_gallery.context_menu.usage.profile")),
                ),
                shadcn::ContextMenuEntry::Item(
                    shadcn::ContextMenuItem::new("Billing")
                        .action(CommandId::new("ui_gallery.context_menu.usage.billing")),
                ),
                shadcn::ContextMenuEntry::Item(
                    shadcn::ContextMenuItem::new("Team")
                        .action(CommandId::new("ui_gallery.context_menu.usage.team")),
                ),
                shadcn::ContextMenuEntry::Item(
                    shadcn::ContextMenuItem::new("Subscription")
                        .action(CommandId::new("ui_gallery.context_menu.usage.subscription")),
                ),
            ]
        },
    )
}
// endregion: example
