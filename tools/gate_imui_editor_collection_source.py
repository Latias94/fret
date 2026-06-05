from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from _gate_lib import WORKSPACE_ROOT, fail, ok


GATE_NAME = "imui editor collection source"


@dataclass(frozen=True)
class SourceCheck:
    label: str
    path: Path
    required: list[str]
    forbidden: list[str]
    extra_paths: tuple[Path, ...] = ()


def read_source(path: Path) -> str:
    try:
        return (WORKSPACE_ROOT / path).read_text(encoding="utf-8")
    except OSError as exc:
        fail(GATE_NAME, f"failed to read {path.as_posix()}: {exc}")


def check_source(check: SourceCheck, failures: list[str]) -> None:
    source = "\n".join(
        [read_source(check.path)]
        + [read_source(extra_path) for extra_path in check.extra_paths]
    )
    for marker in check.required:
        if marker not in source:
            failures.append(f"{check.label}: {check.path.as_posix()}: missing {marker}")
    for marker in check.forbidden:
        if marker in source:
            failures.append(f"{check.label}: {check.path.as_posix()}: forbidden {marker}")


def main() -> None:
    demo = Path("apps/fret-examples/src/imui_editor_proof_demo.rs")
    collection = Path("apps/fret-examples/src/imui_editor_proof_demo/collection.rs")
    collection_geometry = Path(
        "apps/fret-examples/src/imui_editor_proof_demo/collection/geometry.rs"
    )
    collection_models = Path(
        "apps/fret-examples/src/imui_editor_proof_demo/collection/models.rs"
    )
    collection_readouts = Path(
        "apps/fret-examples/src/imui_editor_proof_demo/collection/readouts.rs"
    )
    collection_rename = Path(
        "apps/fret-examples/src/imui_editor_proof_demo/collection/rename.rs"
    )
    collection_selection = Path(
        "apps/fret-examples/src/imui_editor_proof_demo/collection/selection.rs"
    )
    collection_children = (
        collection_geometry,
        collection_models,
        collection_readouts,
        collection_rename,
        collection_selection,
    )

    checks = [
        SourceCheck(
            "modularization demo routing",
            demo,
            required=[
                "mod collection;",
                "collection::render_collection_first_asset_browser_proof(ui);",
                "collection::authoring_parity_collection_assets()",
            ],
            forbidden=[
                "fn proof_collection_assets_in_visible_order(",
                "fn authoring_parity_collection_assets() -> Arc<[ProofCollectionAsset]> {",
                "struct ProofCollectionAsset {",
                "fn proof_collection_drag_rect_normalizes_drag_direction()",
            ],
        ),
        SourceCheck(
            "modularization collection owner",
            collection,
            required=[
                "pub(super) fn authoring_parity_collection_assets() -> Arc<[ProofCollectionAsset]> {",
                "pub(super) fn render_collection_first_asset_browser_proof(",
                "ui: &mut ImUi<'_, '_, KernelApp>",
                "mod models;",
                "mod rename;",
                "mod selection;",
                "#[cfg(test)]",
                "fn proof_collection_drag_rect_normalizes_drag_direction() {",
            ],
            forbidden=[],
            extra_paths=collection_children,
        ),
        SourceCheck(
            "collection models owner",
            collection_models,
            required=[
                "pub(super) fn authoring_parity_collection_selection_model<H: UiHost>(",
                "pub(super) fn authoring_parity_collection_assets_model<H: UiHost>(",
                "pub(super) fn authoring_parity_collection_reverse_order_model<H: UiHost>(",
                "pub(super) fn authoring_parity_collection_box_select_model<H: UiHost>(",
                "pub(super) fn authoring_parity_collection_keyboard_model<H: UiHost>(",
                "pub(super) fn authoring_parity_collection_zoom_model<H: UiHost>(",
                "pub(super) fn authoring_parity_collection_scroll_handle<H: UiHost>(",
                "pub(super) fn authoring_parity_collection_context_menu_anchor_model<H: UiHost>(",
                "pub(super) fn authoring_parity_collection_rename_session_model<H: UiHost>(",
                "pub(super) fn authoring_parity_collection_rename_draft_model<H: UiHost>(",
                "pub(super) fn authoring_parity_collection_rename_focus_pending_model<H: UiHost>(",
                "pub(super) fn authoring_parity_collection_active_focus_target_model<H: UiHost>(",
                "pub(super) fn authoring_parity_collection_rename_status_model<H: UiHost>(",
                "pub(super) fn authoring_parity_collection_command_status_model<H: UiHost>(",
                "pub(super) fn authoring_parity_collection_drop_status_model<H: UiHost>(",
                "imui_editor_proof_demo.model.authoring_parity.collection_selection",
                "imui_editor_proof_demo.state.authoring_parity.collection_scroll_handle",
            ],
            forbidden=[
                "render_collection_first_asset_browser_proof",
                "proof_collection_readout_text",
                "proof_collection_drag_rect",
            ],
        ),
        SourceCheck(
            "collection inline rename owner",
            collection_rename,
            required=[
                "pub(super) struct ProofCollectionRenameSession {",
                "pub(super) struct ProofCollectionRenameCommit {",
                "struct ProofCollectionInlineRenameFocusState {",
                "pub(super) fn proof_collection_rename_shortcut_matches(",
                "pub(super) fn proof_collection_begin_rename_session(",
                "pub(super) fn proof_collection_begin_inline_rename_in_app(",
                "pub(super) fn proof_collection_commit_rename(",
                "pub(super) fn proof_collection_inline_rename_focus_state<",
                "pub(super) fn proof_collection_sync_inline_rename_focus<",
                "pub(super) fn proof_collection_restore_focus_after_inline_rename(",
                "proof_collection_rename_ready_status(",
                "host.request_focus(input_id);",
                "fn proof_collection_begin_rename_session_prefers_active_visible_asset()",
                "fn proof_collection_commit_rename_updates_label_without_touching_order_or_ids()",
            ],
            forbidden=[
                "render_collection_first_asset_browser_proof",
                "TextField::new(",
                "TextFieldOptions {",
                "DragPreviewGhostOptions",
                "drag_preview_ghost",
                "kit::ButtonOptions",
                "kit::ChildRegionOptions",
                "kit::GridOptions",
                "kit::MenuItemOptions",
            ],
        ),
        SourceCheck(
            "collection selection owner",
            collection_selection,
            required=[
                "pub(super) struct ProofCollectionKeyboardState {",
                "pub(super) struct ProofCollectionDeleteResult {",
                "pub(super) struct ProofCollectionDuplicateResult {",
                "pub(super) fn proof_collection_assets_in_visible_order(",
                "pub(super) fn proof_collection_selected_assets",
                "pub(super) fn proof_collection_active_id(",
                "pub(super) fn proof_collection_select_all_shortcut_matches(",
                "pub(super) fn proof_collection_duplicate_shortcut_matches(",
                "pub(super) fn proof_collection_select_all_selection(",
                "pub(super) fn proof_collection_context_menu_selection(",
                "pub(super) fn proof_collection_keyboard_selection(",
                "pub(super) fn proof_collection_delete_key_matches(",
                "pub(super) fn proof_collection_delete_selection(",
                "pub(super) fn proof_collection_duplicate_selection(",
                "fn proof_collection_keyboard_arrow_replaces_selection_and_moves_active_tile()",
                "fn proof_collection_duplicate_selection_reselects_visible_copies_and_preserves_active_copy()",
                "fn proof_collection_delete_selection_removes_selected_assets_and_refocuses_next_visible_item()",
            ],
            forbidden=[
                "render_collection_first_asset_browser_proof",
                "TextField",
                "DragPreviewGhostOptions",
                "drag_preview_ghost",
                "kit::ButtonOptions",
                "kit::ChildRegionOptions",
            ],
        ),
        SourceCheck(
            "collection command package",
            collection,
            required=[
                "struct ProofCollectionDuplicateResult {",
                "fn proof_collection_command_package_line() -> String {",
                "fn proof_collection_command_status_line(status: &str) -> String {",
                "fn proof_collection_duplicate_shortcut_matches(",
                "fn proof_collection_duplicate_selection(",
                "fn proof_collection_duplicate_status(",
                "fn proof_collection_begin_inline_rename_in_app(",
                "fn authoring_parity_collection_command_status_model<H: UiHost>(",
                "imui_editor_proof_demo.model.authoring_parity.collection_command_status",
                "\"Duplicate, delete, rename, and select-all stay inside one app-owned collection command package; duplicate/delete/rename now route across keyboard, explicit buttons, and context menu without widening shared IMUI helpers.\"",
                "\"Duplicate selected assets\"",
                "\"Rename active asset\"",
                "\"imui-editor-proof.authoring.imui.collection.duplicate-selected\"",
                "\"imui-editor-proof.authoring.imui.collection.rename-active\"",
                "\"imui-editor-proof.authoring.imui.collection.context-menu.duplicate-selected\"",
                "\"Command status: {status}\"",
                "proof_collection_duplicate_shortcut_matches(",
                "KeyCode::KeyD",
                "shortcut: Some(Arc::from(\"Primary+D\"))",
            ],
            forbidden=[
                "fret_ui_kit::imui::collection_command_package",
                "pub fn collection_command_package",
                "pub fn collection_duplicate_selected",
                "struct ImUiCollectionCommandPackage",
            ],
            extra_paths=collection_children,
        ),
        SourceCheck(
            "collection context menu",
            collection,
            required=[
                "fn proof_collection_context_menu_line() -> String {",
                "fn proof_collection_context_menu_selection(",
                "fn authoring_parity_collection_context_menu_anchor_model<H: UiHost>(",
                "imui_editor_proof_demo.model.authoring_parity.collection_context_menu_anchor",
                "\"Right-click an asset or the collection background to open app-local collection actions.\"",
                "trigger.context_menu_requested()",
                "ui.open_popup_at(",
                "\"imui-editor-proof.authoring.imui.collection.context-menu\"",
                "\"imui-editor-proof.authoring.imui.collection.context-menu.delete-selected\"",
                "\"imui-editor-proof.authoring.imui.collection.context-menu.dismiss\"",
                "Dismiss quick actions",
            ],
            forbidden=[
                "fret_ui_kit::imui::collection_context_menu",
                "pub fn collection_context_menu",
                "struct ImUiCollectionContextMenu",
            ],
            extra_paths=collection_children,
        ),
        SourceCheck(
            "collection keyboard owner",
            collection,
            required=[
                "struct ProofCollectionKeyboardState {",
                "fn proof_collection_active_line(",
                "fn proof_collection_keyboard_selection(",
                "fn proof_collection_keyboard_next_index(",
                "fn proof_collection_keyboard_move_selection(",
                "imui_editor_proof_demo.model.authoring_parity.collection_keyboard",
                "cx.key_on_key_down_for(scope_id, Arc::new(move |host, acx, down| {",
                "host.request_focus(acx.target);",
                "state.active_id = next_selection.first_selected().cloned();",
                "state.active_id = None;",
                "\"Active tile: none. Click background to focus the collection scope, then use Arrow/Home/End to drive selection app-locally.\"",
                "\"Active tile: {}. Shift+Arrow/Home/End extends from the current anchor; Escape clears the selection without widening shared IMUI helper ownership.\"",
            ],
            forbidden=[
                "fret_ui_kit::imui::collection_keyboard_owner",
                "pub fn collection_keyboard_owner",
                "pub fn set_next_collection_shortcut",
                "SetNextItemShortcut",
                "SetItemKeyOwner",
                "state.active_id = next_selection.selected.first().cloned();",
            ],
            extra_paths=collection_children,
        ),
        SourceCheck(
            "collection select all",
            collection,
            required=[
                "fn proof_collection_select_all_line() -> String {",
                "fn proof_collection_select_all_shortcut_matches(",
                "fn proof_collection_select_all_selection(",
                "\"Primary+A selects all visible assets inside the focused collection scope.\"",
                "proof_collection_select_all_shortcut_matches(",
                "KeyCode::KeyA",
                "proof_collection_select_all_selection(",
                "proof_collection_readout_text(\n        ui,\n        proof_collection_select_all_line(),",
            ],
            forbidden=[
                "fret_ui_kit::imui::collection_select_all",
                "pub fn collection_select_all",
                "struct ImUiCollectionSelectAll",
            ],
            extra_paths=collection_children,
        ),
        SourceCheck(
            "collection inline rename",
            collection,
            required=[
                "fn proof_collection_rename_line() -> String {",
                "fn proof_collection_rename_shortcut_matches(",
                "fn proof_collection_begin_rename_session(",
                "fn proof_collection_begin_inline_rename_in_app(",
                "fn proof_collection_commit_rename(",
                "fn proof_collection_inline_rename_focus_state<",
                "fn proof_collection_sync_inline_rename_focus<",
                "fn proof_collection_restore_focus_after_inline_rename(",
                "\"F2, the explicit rename button, or the context menu starts an app-local inline rename editor for the current active asset.\"",
                "proof_collection_rename_shortcut_matches(",
                "KeyCode::F2",
                "\"imui-editor-proof.authoring.imui.collection.rename-active\"",
                "\"imui-editor-proof.authoring.imui.collection.context-menu.rename\"",
                "\"imui-editor-proof.authoring.imui.collection.asset.{}.rename.inline\"",
                "\"Rename active asset\"",
                "TextField::new(",
                "TextFieldOptions {",
                "EditorTextSelectionBehavior::SelectAllOnFocus",
                "TextFieldBlurBehavior::Cancel",
                "proof_collection_inline_rename_focus_state(",
                "proof_collection_sync_inline_rename_focus(",
                "proof_collection_readout_text(\n        ui,\n        proof_collection_rename_line(),",
            ],
            forbidden=[
                "ui.begin_popup_modal_with_options(",
                "PROOF_COLLECTION_RENAME_COMMIT_COMMAND",
                "PROOF_COLLECTION_RENAME_CANCEL_COMMAND",
                "\"imui-editor-proof.authoring.imui.collection.rename.input\"",
                "\"imui-editor-proof.authoring.imui.collection.rename.commit\"",
                "\"imui-editor-proof.authoring.imui.collection.rename.cancel\"",
                "fret_ui_kit::imui::collection_rename",
                "pub fn collection_rename",
                "struct ImUiCollectionRename",
            ],
            extra_paths=collection_children,
        ),
        SourceCheck(
            "collection delete action",
            collection,
            required=[
                "struct ProofCollectionDeleteResult {",
                "fn proof_collection_assets_line(",
                "fn proof_collection_delete_key_matches(",
                "fn proof_collection_delete_selection(",
                "fn authoring_parity_collection_assets_model<H: UiHost>(",
                "imui_editor_proof_demo.model.authoring_parity.collection_assets",
                "\"Delete selected assets\"",
                "\"imui-editor-proof.authoring.imui.collection.delete-selected\"",
                "\"Assets: {}. Press Delete/Backspace or use the explicit action button to remove the selected set app-locally.\"",
                "proof_collection_delete_key_matches(down.key)",
            ],
            forbidden=[
                "fret_ui_kit::imui::collection_delete_action",
                "pub fn collection_delete_action",
                "pub fn delete_selected_assets",
                "struct ImUiCollectionDeleteAction",
            ],
            extra_paths=collection_children,
        ),
        SourceCheck(
            "collection box select",
            collection,
            required=[
                "Background drag now draws a marquee and updates grid selection app-locally while shared helper widening stays deferred until another first-party proof surface exists.",
                "const PROOF_COLLECTION_BOX_SELECT_DRAG_THRESHOLD_PX: f32 = 6.0;",
                "struct ProofCollectionBoxSelectSession {",
                "struct ProofCollectionBoxSelectState {",
                "fn proof_collection_box_select_selection(",
                "fn proof_collection_box_select_active_rect(",
                "ImUiMultiSelectState::from_ordered_selection(",
                "imui_editor_proof_demo.model.authoring_parity.collection_box_select",
                "props.capture_phase_pointer_moves = true;",
                "cx.pointer_region_on_pointer_down(Arc::new(move |host, acx, down| {",
                "cx.pointer_region_on_pointer_move(Arc::new(move |host, acx, mv| {",
                "cx.pointer_region_on_pointer_up(Arc::new(move |host, acx, up| {",
                "host.capture_pointer();",
                "host.release_pointer_capture();",
                "\"imui-editor-proof.authoring.imui.collection.box-select.scope\"",
                "\"imui-editor-proof.authoring.imui.collection.box-select.marquee\"",
            ],
            forbidden=[
                "fret_ui_kit::imui::collection_box_select",
                "pub fn collection_box_select",
                "struct ImUiCollectionBoxSelect",
                "fn proof_collection_normalize_selection(",
            ],
            extra_paths=collection_children,
        ),
        SourceCheck(
            "collection zoom",
            collection,
            required=[
                "struct ProofCollectionLayoutMetrics {",
                "struct ProofCollectionZoomUpdate {",
                "fn proof_collection_layout_metrics(",
                "fn proof_collection_zoom_line(",
                "fn proof_collection_zoom_request(",
                "fn authoring_parity_collection_zoom_model<H: UiHost>(",
                "fn authoring_parity_collection_scroll_handle<H: UiHost>(",
                "imui_editor_proof_demo.model.authoring_parity.collection_zoom",
                "imui_editor_proof_demo.state.authoring_parity.collection_scroll_handle",
                "\"Primary+Wheel zoom stays app-owned: {} px target tiles across {} column(s), with hovered rows staying anchored inside the collection proof.\"",
                "collection_scroll_handle.viewport_size().width",
                "handle: Some(collection_scroll_handle.clone())",
                "proof_collection_zoom_request(",
                "collection_layout.columns",
                "collection_scroll_handle_for_wheel.set_offset(update.next_scroll_offset);",
            ],
            forbidden=[
                "fret_ui_kit::imui::collection_zoom",
                "pub fn collection_zoom",
                "pub fn collection_layout_metrics",
                "struct ImUiCollectionZoom",
            ],
            extra_paths=collection_children,
        ),
    ]

    failures: list[str] = []
    for check in checks:
        check_source(check, failures)

    if failures:
        fail(GATE_NAME, f"{len(failures)} source marker problem(s):\n  - " + "\n  - ".join(failures))

    ok(GATE_NAME)


if __name__ == "__main__":
    main()
