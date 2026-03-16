use crate::context_menu::{ContextMenu, ContextMenuEntry, ContextMenuItem};
use fret_runtime::{CommandId, Model};
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::IntoUiElement;

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
/// This intentionally stays `-> AnyElement` because `ContextMenu::build(...)` is itself the final
/// wrapper landing seam: the helper evaluates the typed trigger and injects the fixed entry set in
/// one root-level overlay call.
///
/// The underlying commands are `edit.*` so runners can map them to native OS actions when
/// available (desktop now; mobile later).
///
/// Example (wrap a shadcn `Input`):
/// ```
/// # use fret_runtime::Model;
/// # use fret_ui::ElementContext;
/// # use fret_ui::UiHost;
/// # use fret_ui_shadcn::{facade as shadcn, prelude::*};
/// # use fret_ui_shadcn::text_edit_context_menu_controllable;
/// # fn demo<H: UiHost>(cx: &mut ElementContext<'_, H>, model: Model<String>) {
/// let trigger =
///     |cx: &mut ElementContext<'_, H>| shadcn::Input::new(model.clone()).into_element(cx);
/// let _menu = text_edit_context_menu_controllable(cx, None, false, trigger);
/// # }
/// ```
#[track_caller]
pub fn text_edit_context_menu<H: UiHost, TTrigger>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    trigger: impl FnOnce(&mut ElementContext<'_, H>) -> TTrigger,
) -> AnyElement
where
    TTrigger: IntoUiElement<H>,
{
    let trigger = trigger(cx);
    ContextMenu::from_open(open).build(cx, trigger, |_cx| text_edit_context_menu_entries())
}

/// Wraps a trigger element with a standard text-selection context menu.
///
/// Use this for non-editable selectable text surfaces.
/// This keeps the same deliberate raw landing seam as [`text_edit_context_menu`].
#[track_caller]
pub fn text_selection_context_menu<H: UiHost, TTrigger>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    trigger: impl FnOnce(&mut ElementContext<'_, H>) -> TTrigger,
) -> AnyElement
where
    TTrigger: IntoUiElement<H>,
{
    let trigger = trigger(cx);
    ContextMenu::from_open(open).build(cx, trigger, |_cx| text_selection_context_menu_entries())
}

/// Like [`text_edit_context_menu`], but supports controlled/uncontrolled open state.
/// This keeps the same deliberate raw landing seam as [`text_edit_context_menu`].
#[track_caller]
pub fn text_edit_context_menu_controllable<H: UiHost, TTrigger>(
    cx: &mut ElementContext<'_, H>,
    open: Option<Model<bool>>,
    default_open: bool,
    trigger: impl FnOnce(&mut ElementContext<'_, H>) -> TTrigger,
) -> AnyElement
where
    TTrigger: IntoUiElement<H>,
{
    let trigger = trigger(cx);
    ContextMenu::new_controllable(cx, open, default_open)
        .build(cx, trigger, |_cx| text_edit_context_menu_entries())
}

/// Like [`text_selection_context_menu`], but supports controlled/uncontrolled open state.
/// This keeps the same deliberate raw landing seam as [`text_edit_context_menu`].
#[track_caller]
pub fn text_selection_context_menu_controllable<H: UiHost, TTrigger>(
    cx: &mut ElementContext<'_, H>,
    open: Option<Model<bool>>,
    default_open: bool,
    trigger: impl FnOnce(&mut ElementContext<'_, H>) -> TTrigger,
) -> AnyElement
where
    TTrigger: IntoUiElement<H>,
{
    let trigger = trigger(cx);
    ContextMenu::new_controllable(cx, open, default_open)
        .build(cx, trigger, |_cx| text_selection_context_menu_entries())
}
