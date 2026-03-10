use crate::context_menu::{ContextMenu, ContextMenuEntry, ContextMenuItem};
use fret_runtime::{CommandId, Model};
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

/// Returns standard text-edit context menu entries (Copy/Cut/Paste/Select All).
///
/// This is intended to be used with `ContextMenu` triggers that wrap text inputs/areas.
///
/// Notes:
/// - Entries are wired to `edit.*` commands to stay consistent across desktop/web/mobile runners.
/// - Enable/disable behavior is driven by command gating + widget `command_availability`.
pub fn text_edit_context_menu_entries() -> Vec<ContextMenuEntry> {
    vec![
        ContextMenuEntry::Item(ContextMenuItem::new("Copy").action(CommandId::from("edit.copy"))),
        ContextMenuEntry::Item(ContextMenuItem::new("Cut").action(CommandId::from("edit.cut"))),
        ContextMenuEntry::Item(ContextMenuItem::new("Paste").action(CommandId::from("edit.paste"))),
        ContextMenuEntry::Separator,
        ContextMenuEntry::Item(
            ContextMenuItem::new("Select All").action(CommandId::from("edit.select_all")),
        ),
    ]
}

/// Returns standard non-editable text-selection context menu entries (Copy/Select All).
///
/// This is intended to be used with `ContextMenu` triggers that wrap non-editable text surfaces
/// (e.g. `SelectableText` / `SelectableLabel`).
pub fn text_selection_context_menu_entries() -> Vec<ContextMenuEntry> {
    vec![
        ContextMenuEntry::Item(ContextMenuItem::new("Copy").action(CommandId::from("edit.copy"))),
        ContextMenuEntry::Separator,
        ContextMenuEntry::Item(
            ContextMenuItem::new("Select All").action(CommandId::from("edit.select_all")),
        ),
    ]
}

/// Wraps a trigger element with a standard text-edit context menu.
///
/// This is an opt-in helper: it does not change `Input` / `Textarea` default behavior.
///
/// The underlying commands are `edit.*` so runners can map them to native OS actions when
/// available (desktop now; mobile later).
///
/// Example (wrap a shadcn `Input`):
/// ```
/// # use fret_runtime::Model;
/// # use fret_ui::ElementContext;
/// # use fret_ui::UiHost;
/// # use fret_ui_shadcn::{Input, text_edit_context_menu_controllable};
/// # fn demo<H: UiHost>(cx: &mut ElementContext<'_, H>, model: Model<String>) {
/// let trigger = |cx: &mut ElementContext<'_, H>| Input::new(model.clone()).into_element(cx);
/// let _menu = text_edit_context_menu_controllable(cx, None, false, trigger);
/// # }
/// ```
#[track_caller]
pub fn text_edit_context_menu<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
) -> AnyElement {
    ContextMenu::new(open).into_element(cx, trigger, |_cx| text_edit_context_menu_entries())
}

/// Wraps a trigger element with a standard text-selection context menu.
///
/// Use this for non-editable selectable text surfaces.
#[track_caller]
pub fn text_selection_context_menu<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
) -> AnyElement {
    ContextMenu::new(open).into_element(cx, trigger, |_cx| text_selection_context_menu_entries())
}

/// Like [`text_edit_context_menu`], but supports controlled/uncontrolled open state.
#[track_caller]
pub fn text_edit_context_menu_controllable<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: Option<Model<bool>>,
    default_open: bool,
    trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
) -> AnyElement {
    ContextMenu::new_controllable(cx, open, default_open)
        .into_element(cx, trigger, |_cx| text_edit_context_menu_entries())
}

/// Like [`text_selection_context_menu`], but supports controlled/uncontrolled open state.
#[track_caller]
pub fn text_selection_context_menu_controllable<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: Option<Model<bool>>,
    default_open: bool,
    trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
) -> AnyElement {
    ContextMenu::new_controllable(cx, open, default_open)
        .into_element(cx, trigger, |_cx| text_selection_context_menu_entries())
}
