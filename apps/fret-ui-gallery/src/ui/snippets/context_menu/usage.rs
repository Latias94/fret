pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_runtime::CommandId;
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::ContextMenu::uncontrolled(cx)
        .compose()
        .trigger(shadcn::ContextMenuTrigger::build(
            shadcn::Button::new("Right click here").variant(shadcn::ButtonVariant::Outline),
        ))
        .content(shadcn::ContextMenuContent::new())
        .entries([
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
        ])
}
// endregion: example
