use crate::context_menu::{ContextMenuEntry, ContextMenuItem};
use fret_runtime::CommandId;

/// Returns standard text-edit context menu entries (Copy/Cut/Paste/Select All).
///
/// This is intended to be used with `ContextMenu` triggers that wrap text inputs/areas.
///
/// Notes:
/// - Entries are wired to `edit.*` commands to stay consistent across desktop/web/mobile runners.
/// - Enable/disable behavior is driven by command gating + widget `command_availability`.
pub fn text_edit_context_menu_entries() -> Vec<ContextMenuEntry> {
    vec![
        ContextMenuEntry::Item(
            ContextMenuItem::new("Copy").on_select(CommandId::from("edit.copy")),
        ),
        ContextMenuEntry::Item(ContextMenuItem::new("Cut").on_select(CommandId::from("edit.cut"))),
        ContextMenuEntry::Item(
            ContextMenuItem::new("Paste").on_select(CommandId::from("edit.paste")),
        ),
        ContextMenuEntry::Separator,
        ContextMenuEntry::Item(
            ContextMenuItem::new("Select All").on_select(CommandId::from("edit.select_all")),
        ),
    ]
}
