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
    authoring_parity = Path(
        "apps/fret-examples/src/imui_editor_proof_demo/authoring_parity.rs"
    )
    authoring_parity_models = Path(
        "apps/fret-examples/src/imui_editor_proof_demo/authoring_parity/models.rs"
    )
    collection = Path("apps/fret-examples/src/imui_editor_proof_demo/collection.rs")
    collection_asset_grid = Path(
        "apps/fret-examples/src/imui_editor_proof_demo/collection/asset_grid.rs"
    )
    collection_assets = Path(
        "apps/fret-examples/src/imui_editor_proof_demo/collection/assets.rs"
    )
    collection_browser_scope = Path(
        "apps/fret-examples/src/imui_editor_proof_demo/collection/browser_scope.rs"
    )
    collection_browser_scope_input_runtime = Path(
        "apps/fret-examples/src/imui_editor_proof_demo/collection/browser_scope/input_runtime.rs"
    )
    collection_box_select = Path(
        "apps/fret-examples/src/imui_editor_proof_demo/collection/box_select.rs"
    )
    collection_chrome = Path(
        "apps/fret-examples/src/imui_editor_proof_demo/collection/chrome.rs"
    )
    collection_command_buttons = Path(
        "apps/fret-examples/src/imui_editor_proof_demo/collection/command_buttons.rs"
    )
    collection_context_menu = Path(
        "apps/fret-examples/src/imui_editor_proof_demo/collection/context_menu.rs"
    )
    collection_derived_state = Path(
        "apps/fret-examples/src/imui_editor_proof_demo/collection/derived_state.rs"
    )
    collection_runtime_state = Path(
        "apps/fret-examples/src/imui_editor_proof_demo/collection/runtime_state.rs"
    )
    collection_drag_drop = Path(
        "apps/fret-examples/src/imui_editor_proof_demo/collection/drag_drop.rs"
    )
    collection_geometry = Path(
        "apps/fret-examples/src/imui_editor_proof_demo/collection/geometry.rs"
    )
    collection_import_target = Path(
        "apps/fret-examples/src/imui_editor_proof_demo/collection/import_target.rs"
    )
    collection_keyboard = Path(
        "apps/fret-examples/src/imui_editor_proof_demo/collection/keyboard.rs"
    )
    collection_models = Path(
        "apps/fret-examples/src/imui_editor_proof_demo/collection/models.rs"
    )
    collection_order_toggle = Path(
        "apps/fret-examples/src/imui_editor_proof_demo/collection/order_toggle.rs"
    )
    collection_readouts = Path(
        "apps/fret-examples/src/imui_editor_proof_demo/collection/readouts.rs"
    )
    collection_status_readouts = Path(
        "apps/fret-examples/src/imui_editor_proof_demo/collection/status_readouts.rs"
    )
    collection_rename = Path(
        "apps/fret-examples/src/imui_editor_proof_demo/collection/rename.rs"
    )
    collection_selection = Path(
        "apps/fret-examples/src/imui_editor_proof_demo/collection/selection.rs"
    )
    collection_selection_commands = Path(
        "apps/fret-examples/src/imui_editor_proof_demo/collection/selection/commands.rs"
    )
    collection_selection_command_delete = Path(
        "apps/fret-examples/src/imui_editor_proof_demo/collection/selection/commands/delete.rs"
    )
    collection_selection_command_duplicate = Path(
        "apps/fret-examples/src/imui_editor_proof_demo/collection/selection/commands/duplicate.rs"
    )
    collection_children = (
        collection_asset_grid,
        collection_assets,
        collection_browser_scope,
        collection_browser_scope_input_runtime,
        collection_box_select,
        collection_chrome,
        collection_command_buttons,
        collection_context_menu,
        collection_derived_state,
        collection_runtime_state,
        collection_drag_drop,
        collection_geometry,
        collection_import_target,
        collection_keyboard,
        collection_models,
        collection_order_toggle,
        collection_readouts,
        collection_status_readouts,
        collection_rename,
        collection_selection,
        collection_selection_commands,
        collection_selection_command_delete,
        collection_selection_command_duplicate,
    )

    checks = [
        SourceCheck(
            "modularization demo routing",
            demo,
            required=[
                "mod authoring_parity;",
                "mod collection;",
                "collection::render_collection_first_asset_browser_proof(ui);",
                "authoring_parity::drag_assets()",
            ],
            forbidden=[
                "fn proof_collection_assets_in_visible_order(",
                "fn authoring_parity_collection_assets() -> Arc<[ProofCollectionAsset]> {",
                "struct ProofCollectionAsset {",
                "fn proof_collection_drag_rect_normalizes_drag_direction()",
                "collection::authoring_parity_collection_assets()",
            ],
        ),
        SourceCheck(
            "modularization authoring parity hub",
            authoring_parity,
            required=[
                "mod models;",
                "mod shared_state;",
                "pub(super) use models::{",
                "drag_assets",
                "outliner_items_model",
                "pub(super) use shared_state::render_shared_state;",
            ],
            forbidden=[
                "fn drag_assets(",
                "fn outliner_items_model(",
                "fn render_shared_state(",
            ],
        ),
        SourceCheck(
            "modularization authoring parity model owner",
            authoring_parity_models,
            required=[
                "pub(in super::super) fn drag_assets() -> Arc<[ProofDragAsset]> {",
                "super::super::collection::authoring_parity_collection_assets()",
                "pub(in super::super) fn outliner_items() -> Arc<[ProofOutlinerItem]> {",
                "pub(in super::super) fn outliner_items_model<H: UiHost>(",
            ],
            forbidden=[],
        ),
        SourceCheck(
            "modularization collection owner",
            collection,
            required=[
                "pub(super) fn render_collection_first_asset_browser_proof(",
                "ui: &mut ImUi<'_, '_, KernelApp>",
                "mod asset_grid;",
                "mod assets;",
                "mod browser_scope;",
                "mod box_select;",
                "mod chrome;",
                "mod command_buttons;",
                "mod context_menu;",
                "mod derived_state;",
                "mod drag_drop;",
                "mod keyboard;",
                "mod import_target;",
                "mod models;",
                "mod order_toggle;",
                "mod rename;",
                "mod runtime_state;",
                "mod selection;",
                "mod status_readouts;",
                "pub(super) use assets::{ProofCollectionAsset, authoring_parity_collection_assets};",
                "pub(super) use chrome::proof_collection_readout_text;",
                "use chrome::proof_collection_section_label;",
                "use derived_state::proof_collection_derived_state;",
                "use import_target::render_collection_import_target;",
                "use order_toggle::render_collection_order_toggle;",
                "use runtime_state::proof_collection_runtime_state;",
                "render_collection_import_target(ui);",
                "render_collection_order_toggle(",
                "proof_collection_derived_state(",
                "proof_collection_runtime_state(",
                "use status_readouts::{",
                "render_collection_status_readouts(",
                "#[cfg(test)]",
                "fn proof_collection_drag_rect_normalizes_drag_direction() {",
            ],
            forbidden=[],
            extra_paths=collection_children,
        ),
        SourceCheck(
            "collection derived state delegation",
            collection,
            required=[
                "mod derived_state;",
                "use derived_state::proof_collection_derived_state;",
                "let collection_state = proof_collection_derived_state(",
                "&collection_state.assets",
                "&collection_state.keys",
                "collection_state.active_id.as_ref()",
                "collection_state.rename_ready_session.as_ref()",
            ],
            forbidden=[
                "proof_collection_assets_in_visible_order(",
                "proof_collection_active_id(",
                "proof_collection_begin_rename_session(",
                "let collection_keys =",
                "let collection_active_id =",
                "let collection_rename_ready_session =",
            ],
        ),
        SourceCheck(
            "collection derived state owner",
            collection_derived_state,
            required=[
                "pub(super) struct ProofCollectionDerivedState",
                "pub(super) fn proof_collection_derived_state(",
                "stored_assets: &[ProofCollectionAsset]",
                "reverse_order: bool",
                "proof_collection_assets_in_visible_order(",
                "Arc::<[ProofCollectionAsset]>::from(stored_assets.to_vec())",
                "let keys = assets",
                ".map(|asset| asset.id.clone())",
                ".collect::<Vec<_>>();",
                "proof_collection_active_id(&keys, selection, keyboard)",
                "proof_collection_begin_rename_session(&assets, selection, keyboard)",
                "rename_ready_session",
            ],
            forbidden=[
                "render_collection_first_asset_browser_proof",
                "render_collection_browser_scope",
                "render_collection_status_readouts",
                "render_collection_command_buttons",
                "render_collection_import_target",
                "ui.",
                "kit::ButtonOptions",
                "TextField::new(",
            ],
        ),
        SourceCheck(
            "collection runtime state delegation",
            collection,
            required=[
                "mod runtime_state;",
                "use runtime_state::proof_collection_runtime_state;",
                "let collection_runtime = proof_collection_runtime_state(ui);",
                "collection_runtime.models.reverse_order",
                "collection_runtime.snapshot.stored_assets",
                "collection_runtime.snapshot.selection",
                "collection_runtime.snapshot.layout",
                "collection_runtime.snapshot.rename_session.as_ref()",
            ],
            forbidden=[
                "authoring_parity_collection_selection_model(ui.cx_mut())",
                "authoring_parity_collection_assets_model(ui.cx_mut())",
                "authoring_parity_collection_reverse_order_model(ui.cx_mut())",
                "authoring_parity_collection_box_select_model(ui.cx_mut())",
                "authoring_parity_collection_keyboard_model(ui.cx_mut())",
                "authoring_parity_collection_zoom_model(ui.cx_mut())",
                "authoring_parity_collection_context_menu_anchor_model(ui.cx_mut())",
                "authoring_parity_collection_rename_session_model(ui.cx_mut())",
                "authoring_parity_collection_rename_draft_model(ui.cx_mut())",
                "authoring_parity_collection_rename_focus_pending_model(ui.cx_mut())",
                "authoring_parity_collection_active_focus_target_model(ui.cx_mut())",
                "authoring_parity_collection_rename_status_model(ui.cx_mut())",
                "authoring_parity_collection_command_status_model(ui.cx_mut())",
                "authoring_parity_collection_scroll_handle(ui.cx_mut())",
                "selector_model_paint(",
                "proof_collection_layout_metrics(",
                "use fret::advanced::view::AppRenderDataExt as _;",
            ],
        ),
        SourceCheck(
            "collection runtime state owner",
            collection_runtime_state,
            required=[
                "pub(super) struct ProofCollectionRuntimeState",
                "pub(super) struct ProofCollectionRuntimeModels",
                "pub(super) struct ProofCollectionRuntimeSnapshot",
                "pub(super) fn proof_collection_runtime_state(",
                "selection: authoring_parity_collection_selection_model(ui.cx_mut())",
                "assets: authoring_parity_collection_assets_model(ui.cx_mut())",
                "reverse_order: authoring_parity_collection_reverse_order_model(ui.cx_mut())",
                "box_select: authoring_parity_collection_box_select_model(ui.cx_mut())",
                "keyboard: authoring_parity_collection_keyboard_model(ui.cx_mut())",
                "zoom: authoring_parity_collection_zoom_model(ui.cx_mut())",
                "context_menu_anchor: authoring_parity_collection_context_menu_anchor_model(ui.cx_mut())",
                "rename_session: authoring_parity_collection_rename_session_model(ui.cx_mut())",
                "rename_draft: authoring_parity_collection_rename_draft_model(ui.cx_mut())",
                "rename_focus_pending: authoring_parity_collection_rename_focus_pending_model(ui.cx_mut())",
                "active_focus_target: authoring_parity_collection_active_focus_target_model(ui.cx_mut())",
                "rename_status: authoring_parity_collection_rename_status_model(ui.cx_mut())",
                "command_status: authoring_parity_collection_command_status_model(ui.cx_mut())",
                "scroll: authoring_parity_collection_scroll_handle(ui.cx_mut())",
                "fn proof_collection_runtime_snapshot(",
                "selector_model_paint(&models.assets, |state| state.clone())",
                "selector_model_paint(&models.selection, |state| state)",
                "selector_model_paint(&models.rename_status, |state| state.clone())",
                "proof_collection_layout_metrics(models.scroll.viewport_size().width, tile_extent)",
            ],
            forbidden=[
                "render_collection_first_asset_browser_proof",
                "render_collection_browser_scope",
                "render_collection_status_readouts",
                "render_collection_command_buttons",
                "render_collection_import_target",
                "ui.button_with_options(",
                "TextField::new(",
            ],
        ),
        SourceCheck(
            "collection root chrome delegation",
            collection,
            required=[
                "mod chrome;",
                "pub(super) use chrome::proof_collection_readout_text;",
                "use chrome::proof_collection_section_label;",
            ],
            forbidden=[
                "fn proof_collection_readout_text(",
                "fn proof_collection_section_label(",
                "proof_compact_readout_element(cx, text, Arc::<str>::from(test_id))",
                "proof_section_chrome_label(cx, text, test_id)",
            ],
        ),
        SourceCheck(
            "collection chrome owner",
            collection_chrome,
            required=[
                "pub(in super::super) fn proof_collection_readout_text(",
                "pub(super) fn proof_collection_section_label(",
                "proof_compact_readout_element(cx, text, Arc::<str>::from(test_id))",
                "proof_section_chrome_label(cx, text, test_id)",
            ],
            forbidden=[
                "render_collection_first_asset_browser_proof",
                "selector_model_paint",
                "render_collection_asset_grid",
                "render_collection_context_menu",
                "ui.button_with_options(",
                "TextField::new(",
            ],
        ),
        SourceCheck(
            "collection status readouts delegation",
            collection,
            required=[
                "mod status_readouts;",
                "use status_readouts::{",
                "ProofCollectionStatusReadoutState {",
                "render_collection_status_readouts(",
            ],
            forbidden=[
                "proof_collection_assets_line(",
                "proof_collection_visible_order_line(",
                "proof_collection_selection_line(",
                "proof_collection_active_line(",
                "proof_collection_zoom_line(",
                "proof_collection_select_all_line(",
                "proof_collection_rename_line(",
                "proof_collection_context_menu_line(",
                "proof_collection_command_package_line(",
                "proof_collection_rename_status_line(",
                "proof_collection_command_status_line(",
                '"imui-editor-proof.authoring.imui.collection.assets-readout"',
                '"imui-editor-proof.authoring.imui.collection.visible-order-readout"',
                '"imui-editor-proof.authoring.imui.collection.selection-readout"',
                '"imui-editor-proof.authoring.imui.collection.active-readout"',
                '"imui-editor-proof.authoring.imui.collection.zoom-readout"',
                '"imui-editor-proof.authoring.imui.collection.select-all-readout"',
                '"imui-editor-proof.authoring.imui.collection.rename-readout"',
                '"imui-editor-proof.authoring.imui.collection.context-menu-readout"',
                '"imui-editor-proof.authoring.imui.collection.command-package-readout"',
                '"imui-editor-proof.authoring.imui.collection.rename-status-readout"',
                '"imui-editor-proof.authoring.imui.collection.command-status-readout"',
            ],
        ),
        SourceCheck(
            "collection status readouts owner",
            collection_status_readouts,
            required=[
                "pub(super) struct ProofCollectionStatusReadoutState",
                "pub(super) fn render_collection_status_readouts(",
                "proof_collection_assets_line(state.assets)",
                "proof_collection_visible_order_line(state.assets)",
                "proof_collection_selection_line(state.assets, state.selection)",
                "proof_collection_active_line(state.assets, state.selection, state.keyboard)",
                "proof_collection_zoom_line(state.layout)",
                "proof_collection_select_all_line()",
                "proof_collection_rename_line()",
                "proof_collection_context_menu_line()",
                "proof_collection_command_package_line()",
                "proof_collection_rename_status_line(state.rename_status)",
                "proof_collection_command_status_line(state.command_status)",
                '"imui-editor-proof.authoring.imui.collection.assets-readout"',
                '"imui-editor-proof.authoring.imui.collection.visible-order-readout"',
                '"imui-editor-proof.authoring.imui.collection.selection-readout"',
                '"imui-editor-proof.authoring.imui.collection.active-readout"',
                '"imui-editor-proof.authoring.imui.collection.zoom-readout"',
                '"imui-editor-proof.authoring.imui.collection.select-all-readout"',
                '"imui-editor-proof.authoring.imui.collection.rename-readout"',
                '"imui-editor-proof.authoring.imui.collection.context-menu-readout"',
                '"imui-editor-proof.authoring.imui.collection.command-package-readout"',
                '"imui-editor-proof.authoring.imui.collection.rename-status-readout"',
                '"imui-editor-proof.authoring.imui.collection.command-status-readout"',
            ],
            forbidden=[
                "render_collection_first_asset_browser_proof",
                "selector_model_paint",
                "ui.button_with_options(",
                "ui.drop_target::<",
                "render_collection_asset_grid",
                "render_collection_context_menu",
                "TextField::new(",
                "kit::ChildRegionOptions",
                "kit::GridOptions",
                "kit::MenuItemOptions",
            ],
        ),
        SourceCheck(
            "collection import target delegation",
            collection,
            required=[
                "mod import_target;",
                "use import_target::render_collection_import_target;",
                "render_collection_import_target(ui);",
            ],
            forbidden=[
                "ProofCollectionDragPayload",
                "proof_collection_drop_status(",
                "authoring_parity_collection_drop_status_model",
                "ui.drop_target::<",
                '"imui-editor-proof.authoring.imui.collection.import-target"',
                '"imui-editor-proof.authoring.imui.collection.drop-status-readout"',
            ],
        ),
        SourceCheck(
            "collection import target owner",
            collection_import_target,
            required=[
                "pub(super) fn render_collection_import_target(",
                "authoring_parity_collection_drop_status_model(ui.cx_mut())",
                "ui.button_with_options(",
                "ui.drop_target::<ProofCollectionDragPayload>(import_trigger)",
                'proof_collection_drop_status("Delivered", &payload)',
                'proof_collection_drop_status("Preview", &payload)',
                '"Compatible collection drag active"',
                '"imui-editor-proof.authoring.imui.collection.import-target"',
                '"imui-editor-proof.authoring.imui.collection.drop-status-readout"',
            ],
            forbidden=[
                "render_collection_first_asset_browser_proof",
                "render_collection_asset_grid",
                "render_collection_context_menu",
                "drag_source_with_options",
                "TextField::new(",
                "PointerRegionProps",
                "kit::ChildRegionOptions",
                "kit::GridOptions",
                "kit::MenuItemOptions",
            ],
        ),
        SourceCheck(
            "collection order toggle delegation",
            collection,
            required=[
                "mod order_toggle;",
                "use order_toggle::render_collection_order_toggle;",
                "render_collection_order_toggle(",
            ],
            forbidden=[
                '"Show folder order"',
                '"Reverse visible order"',
                '"imui-editor-proof.authoring.imui.collection.order-toggle"',
                "ui.button_with_options(",
                "kit::ButtonOptions {",
                ".update(&collection_reverse_order_model, |value| *value = !*value)",
            ],
        ),
        SourceCheck(
            "collection order toggle owner",
            collection_order_toggle,
            required=[
                "pub(super) fn render_collection_order_toggle(",
                "reverse_order_model: &Model<bool>",
                "if reverse_order {",
                '"Show folder order"',
                '"Reverse visible order"',
                "ui.button_with_options(",
                "kit::ButtonOptions {",
                '"imui-editor-proof.authoring.imui.collection.order-toggle"',
                "if !order_toggle.clicked()",
                ".update(reverse_order_model, |value| *value = !*value)",
                "!reverse_order",
            ],
            forbidden=[
                "render_collection_first_asset_browser_proof",
                "selector_model_paint",
                "render_collection_asset_grid",
                "render_collection_context_menu",
                "ui.drop_target::<",
                "TextField::new(",
                "kit::ChildRegionOptions",
                "kit::GridOptions",
                "kit::MenuItemOptions",
            ],
        ),
        SourceCheck(
            "collection assets owner",
            collection_assets,
            required=[
                "pub(in super::super) struct ProofCollectionAsset {",
                "pub(in super::super) fn authoring_parity_collection_assets() -> Arc<[ProofCollectionAsset]> {",
                'id: Arc::from("stone-albedo")',
                'path: Arc::from("textures/stone/albedo.ktx2")',
                'kind: Arc::from("Texture")',
            ],
            forbidden=[
                "render_collection_first_asset_browser_proof",
                "proof_collection_readout_text",
                "proof_collection_assets_in_visible_order",
            ],
        ),
        SourceCheck(
            "collection asset grid owner",
            collection_asset_grid,
            required=[
                "pub(super) struct ProofCollectionAssetGridModels {",
                "pub(super) struct ProofCollectionAssetGridState<'a> {",
                "pub(super) fn render_collection_asset_grid(",
                "fn render_collection_asset_tile(",
                "fn render_collection_inline_rename_field(",
                "ui.grid_with_options(",
                "ui.multi_selectable_with_options(",
                "proof_collection_context_menu_selection(",
                "TextField::new(",
                "EditorTextSelectionBehavior::SelectAllOnFocus",
                "TextFieldBlurBehavior::Cancel",
                "drag_preview_ghost_with_options(",
                "ProofCollectionRenderedItem {",
                "\"imui-editor-proof.authoring.imui.collection.grid\"",
                "\"imui-editor-proof.authoring.imui.collection.asset.{}.select\"",
                "\"imui-editor-proof.authoring.imui.collection.asset.{}.rename.inline\"",
                "\"imui-editor-proof.authoring.imui.collection.asset.{}.ghost\"",
            ],
            forbidden=[
                "render_collection_first_asset_browser_proof",
                "cx.pointer_region_on_wheel(",
                "cx.pointer_region_on_pointer_down(",
                "cx.pointer_region_on_pointer_move(",
                "cx.pointer_region_on_pointer_up(",
                "cx.pointer_region_on_pointer_cancel(",
                "kit::ChildRegionOptions",
                "ui.button_with_options(",
                "ui.begin_popup_menu(",
                "drop_target::<",
            ],
        ),
        SourceCheck(
            "collection browser scope owner",
            collection_browser_scope,
            required=[
                "mod input_runtime;",
                "use input_runtime::{",
                "pub(super) struct ProofCollectionBrowserScopeModels {",
                "pub(super) struct ProofCollectionBrowserScopeState<'a> {",
                "pub(super) fn render_collection_browser_scope(",
                "ui.child_region_with_options(",
                "kit::ChildRegionOptions {",
                "proof_collection_browser_scope_pointer_props()",
                "install_collection_browser_scope_input_runtime(",
                "proof_collection_box_select_active_rect(",
                "render_collection_asset_grid(",
                "\"imui-editor-proof.authoring.imui.collection.browser\"",
                "\"imui-editor-proof.authoring.imui.collection.browser.viewport\"",
                "\"imui-editor-proof.authoring.imui.collection.browser.content\"",
                "\"imui-editor-proof.authoring.imui.collection.box-select.scope\"",
                "\"imui-editor-proof.authoring.imui.collection.box-select.marquee\"",
            ],
            forbidden=[
                "render_collection_first_asset_browser_proof",
                "fret_ui::element::PointerRegionProps::default()",
                "props.capture_phase_pointer_moves = true;",
                "install_collection_keyboard_handler(",
                "cx.pointer_region_on_wheel(",
                "proof_collection_zoom_request(",
                "cx.pointer_region_on_pointer_down(",
                "cx.pointer_region_on_pointer_move(",
                "cx.pointer_region_on_pointer_up(",
                "cx.pointer_region_on_pointer_cancel(",
                "proof_collection_box_select_selection(",
                "state.clear();",
                "ui.button_with_options(",
                "ui.begin_popup_menu(",
                "drop_target::<",
                "TextField::new(",
                "drag_preview_ghost_with_options(",
            ],
        ),
        SourceCheck(
            "collection browser input runtime owner",
            collection_browser_scope_input_runtime,
            required=[
                "pub(super) struct ProofCollectionBrowserScopeInputModels {",
                "pub(super) struct ProofCollectionBrowserScopeInputState<'a> {",
                "pub(super) fn proof_collection_browser_scope_pointer_props() -> PointerRegionProps {",
                "props.capture_phase_pointer_moves = true;",
                "pub(super) fn install_collection_browser_scope_input_runtime(",
                "install_collection_keyboard_handler(",
                "cx.pointer_region_on_wheel(",
                "proof_collection_zoom_request(",
                "cx.pointer_region_on_pointer_down(",
                "host.request_focus(acx.target);",
                "ProofCollectionBoxSelectSession {",
                "host.capture_pointer();",
                "cx.pointer_region_on_pointer_move(",
                "proof_collection_box_select_selection(",
                "cx.pointer_region_on_pointer_up(",
                "context_menu_anchor_model_for_up",
                "state.clear();",
                "host.release_pointer_capture();",
                "cx.pointer_region_on_pointer_cancel(",
            ],
            forbidden=[
                "render_collection_first_asset_browser_proof",
                "ui.child_region_with_options(",
                "render_collection_asset_grid(",
                "proof_collection_box_select_active_rect(",
                "TextField::new(",
                "drag_preview_ghost_with_options(",
                "ui.begin_popup_menu(",
                "ui.button_with_options(",
            ],
        ),
        SourceCheck(
            "collection box select owner",
            collection_box_select,
            required=[
                "pub(super) struct ProofCollectionRenderedItem {",
                "pub(super) struct ProofCollectionBoxSelectSession {",
                "pub(super) struct ProofCollectionBoxSelectState {",
                "fn proof_collection_box_select_hits(",
                "fn proof_collection_box_select_state_for_hits(",
                "pub(super) fn proof_collection_box_select_selection(",
                "pub(super) fn proof_collection_box_select_active_rect(",
                "ImUiMultiSelectState::from_ordered_selection(",
                "fn proof_collection_box_select_replace_uses_visible_collection_order()",
                "fn proof_collection_box_select_append_preserves_baseline_and_adds_hits()",
            ],
            forbidden=[
                "render_collection_first_asset_browser_proof",
                "PointerRegionProps",
                "pointer_region_on_pointer_down",
                "pointer_region_on_pointer_move",
                "pointer_region_on_pointer_up",
                "TextField::new(",
                "DragPreviewGhostOptions",
                "drag_preview_ghost",
                "kit::ButtonOptions",
                "kit::ChildRegionOptions",
                "kit::GridOptions",
                "kit::MenuItemOptions",
            ],
        ),
        SourceCheck(
            "collection command buttons owner",
            collection_command_buttons,
            required=[
                "pub(super) struct ProofCollectionCommandButtonModels {",
                "pub(super) struct ProofCollectionCommandButtonState<'a> {",
                "pub(super) fn render_collection_command_buttons(",
                "let duplicate_selected = ui.button_with_options(",
                "proof_collection_duplicate_selection(",
                "proof_collection_begin_inline_rename_in_app(",
                "let delete_selected = ui.button_with_options(",
                "proof_collection_delete_selection(",
                "proof_collection_set_command_status(",
                "\"imui-editor-proof.authoring.imui.collection.duplicate-selected\"",
                "\"imui-editor-proof.authoring.imui.collection.rename-active\"",
                "\"imui-editor-proof.authoring.imui.collection.delete-selected\"",
            ],
            forbidden=[
                "render_collection_first_asset_browser_proof",
                "cx.key_on_key_down_for(",
                "ui.begin_popup_menu(",
                "drag_source_with_options",
                "drop_target::<",
                "drag_preview_ghost",
                "PointerRegionProps",
                "TextField::new(",
                "kit::ChildRegionOptions",
                "kit::GridOptions",
                "kit::MenuItemOptions",
            ],
        ),
        SourceCheck(
            "collection context menu owner",
            collection_context_menu,
            required=[
                "pub(super) struct ProofCollectionContextMenuModels {",
                "pub(super) fn render_collection_context_menu(",
                "PROOF_COLLECTION_CONTEXT_MENU_POPUP_ID",
                "ui.open_popup_at(",
                "ui.begin_popup_menu(",
                "proof_collection_begin_rename_session(",
                "proof_collection_begin_inline_rename_in_app(",
                "proof_collection_duplicate_selection(",
                "proof_collection_delete_selection(",
                "kit::MenuItemOptions {",
                "\"imui-editor-proof.authoring.imui.collection.context-menu.selection-readout\"",
                "\"imui-editor-proof.authoring.imui.collection.context-menu.duplicate-selected\"",
                "\"imui-editor-proof.authoring.imui.collection.context-menu.rename\"",
                "\"imui-editor-proof.authoring.imui.collection.context-menu.delete-selected\"",
                "\"imui-editor-proof.authoring.imui.collection.context-menu.dismiss\"",
            ],
            forbidden=[
                "drag_source_with_options",
                "drop_target::<",
                "drag_preview_ghost",
                "TextField::new(",
                "PointerRegionProps",
                "proof_collection_box_select_selection(",
                "proof_collection_drag_payload_for_asset(",
            ],
        ),
        SourceCheck(
            "collection drag drop owner",
            collection_drag_drop,
            required=[
                "pub(super) struct ProofCollectionDragPayload {",
                "pub(super) fn proof_collection_drag_payload_for_asset(",
                "pub(super) fn proof_collection_drag_preview_title(",
                "pub(super) fn proof_collection_drag_preview_subtitle(",
                "pub(super) fn proof_collection_drop_status(",
                "proof_collection_selected_assets(",
                "fn proof_collection_drag_payload_for_selected_asset_carries_selected_set()",
                "fn proof_collection_drag_payload_for_unselected_asset_carries_dragged_asset_only()",
            ],
            forbidden=[
                "render_collection_first_asset_browser_proof",
                "drag_source_with_options",
                "drop_target::<",
                "drag_preview_ghost",
                "proof_drag_preview_card",
                "TextField::new(",
                "PointerRegionProps",
                "kit::ButtonOptions",
                "kit::ChildRegionOptions",
                "kit::GridOptions",
                "kit::MenuItemOptions",
            ],
        ),
        SourceCheck(
            "collection keyboard handler owner",
            collection_keyboard,
            required=[
                "pub(super) struct ProofCollectionKeyboardHandlerModels {",
                "pub(super) fn install_collection_keyboard_handler(",
                "cx.key_on_key_down_for(",
                "down.ime_composing",
                "proof_collection_delete_key_matches(down.key)",
                "proof_collection_rename_shortcut_matches(down.key, down.modifiers)",
                "proof_collection_select_all_shortcut_matches(down.key, down.modifiers)",
                "proof_collection_duplicate_shortcut_matches(down.key, down.modifiers)",
                "proof_collection_keyboard_selection(",
                "host.notify(acx);",
            ],
            forbidden=[
                "render_collection_first_asset_browser_proof",
                "TextField::new(",
                "drag_source_with_options",
                "drop_target::<",
                "drag_preview_ghost",
                "PointerRegionProps",
                "kit::ButtonOptions",
                "kit::ChildRegionOptions",
                "kit::GridOptions",
                "kit::MenuItemOptions",
            ],
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
                "mod commands;",
                "pub(super) use commands::{",
                "pub(super) struct ProofCollectionKeyboardState {",
                "pub(super) fn proof_collection_assets_in_visible_order(",
                "pub(super) fn proof_collection_selected_assets",
                "pub(super) fn proof_collection_active_id(",
                "pub(super) fn proof_collection_select_all_shortcut_matches(",
                "pub(super) fn proof_collection_select_all_selection(",
                "pub(super) fn proof_collection_context_menu_selection(",
                "pub(super) fn proof_collection_keyboard_selection(",
                "fn proof_collection_keyboard_arrow_replaces_selection_and_moves_active_tile()",
            ],
            forbidden=[
                "render_collection_first_asset_browser_proof",
                "HashSet",
                "fn proof_collection_unique_copy_text(",
                "fn proof_collection_duplicate_label_candidate(",
                "fn proof_collection_duplicate_id_candidate(",
                "fn proof_collection_duplicate_path_candidate(",
                "pub(super) fn proof_collection_duplicate_selection(",
                "pub(super) fn proof_collection_delete_selection(",
                "pub(super) fn proof_collection_delete_key_matches(",
                "pub(super) fn proof_collection_duplicate_shortcut_matches(",
                "TextField",
                "DragPreviewGhostOptions",
                "drag_preview_ghost",
                "kit::ButtonOptions",
                "kit::ChildRegionOptions",
            ],
        ),
        SourceCheck(
            "collection selection command hub",
            collection_selection_commands,
            required=[
                "mod delete;",
                "mod duplicate;",
                "pub(in super::super) use delete::{",
                "pub(in super::super) use duplicate::{",
            ],
            forbidden=[
                "render_collection_first_asset_browser_proof",
                "struct ProofCollectionDeleteResult",
                "struct ProofCollectionDuplicateResult",
                "fn proof_collection_delete_selection(",
                "fn proof_collection_duplicate_selection(",
                "fn proof_collection_unique_copy_text(",
                "fn proof_collection_duplicate_label_candidate(",
                "TextField",
                "DragPreviewGhostOptions",
                "drag_preview_ghost",
                "kit::ButtonOptions",
                "kit::ChildRegionOptions",
                "kit::GridOptions",
                "kit::MenuItemOptions",
            ],
        ),
        SourceCheck(
            "collection delete command owner",
            collection_selection_command_delete,
            required=[
                "pub(in super::super::super) struct ProofCollectionDeleteResult {",
                "pub(in super::super::super) fn proof_collection_delete_key_matches(",
                "pub(in super::super::super) fn proof_collection_delete_selection(",
                "fn proof_collection_delete_selection_removes_selected_assets_and_refocuses_next_visible_item()",
                "fn proof_collection_delete_selection_picks_previous_visible_item_at_end()",
            ],
            forbidden=[
                "ProofCollectionDuplicateResult",
                "proof_collection_duplicate_selection(",
                "proof_collection_unique_copy_text(",
                "render_collection_first_asset_browser_proof",
                "TextField",
                "DragPreviewGhostOptions",
                "drag_preview_ghost",
                "kit::ButtonOptions",
                "kit::ChildRegionOptions",
                "kit::GridOptions",
                "kit::MenuItemOptions",
            ],
        ),
        SourceCheck(
            "collection duplicate command owner",
            collection_selection_command_duplicate,
            required=[
                "pub(in super::super::super) struct ProofCollectionDuplicateResult {",
                "pub(in super::super::super) fn proof_collection_duplicate_shortcut_matches(",
                "pub(in super::super::super) fn proof_collection_duplicate_selection(",
                "fn proof_collection_unique_copy_text(",
                "fn proof_collection_duplicate_label_candidate(",
                "fn proof_collection_duplicate_id_candidate(",
                "fn proof_collection_duplicate_path_candidate(",
                "fn proof_collection_duplicate_shortcut_matches_primary_d_only()",
                "fn proof_collection_duplicate_selection_reselects_visible_copies_and_preserves_active_copy()",
            ],
            forbidden=[
                "ProofCollectionDeleteResult",
                "proof_collection_delete_selection(",
                "proof_collection_delete_key_matches(",
                "render_collection_first_asset_browser_proof",
                "TextField",
                "DragPreviewGhostOptions",
                "drag_preview_ghost",
                "kit::ButtonOptions",
                "kit::ChildRegionOptions",
                "kit::GridOptions",
                "kit::MenuItemOptions",
            ],
        ),
        SourceCheck(
            "collection command package",
            collection,
            required=[
                "struct ProofCollectionDuplicateResult {",
                "render_collection_command_buttons(",
                "pub(super) fn render_collection_command_buttons(",
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
            "collection command button delegation",
            collection,
            required=[
                "mod command_buttons;",
                "use command_buttons::{",
                "render_collection_command_buttons(",
                "ProofCollectionCommandButtonModels {",
                "ProofCollectionCommandButtonState {",
            ],
            forbidden=[
                "let duplicate_selected = ui.button_with_options(",
                "proof_collection_set_command_status(",
            ],
        ),
        SourceCheck(
            "collection browser scope asset grid mount",
            collection_browser_scope,
            required=[
                "render_collection_asset_grid(",
                "ProofCollectionAssetGridModels {",
                "ProofCollectionAssetGridState {",
            ],
            forbidden=[
                "ui.grid_with_options(",
                "TextField::new(",
                "drag_preview_ghost_with_options(",
                "render_collection_inline_rename_field(",
            ],
        ),
        SourceCheck(
            "collection browser scope delegation",
            collection,
            required=[
                "mod browser_scope;",
                "use browser_scope::{",
                "render_collection_browser_scope(",
                "ProofCollectionBrowserScopeModels {",
                "ProofCollectionBrowserScopeState {",
            ],
            forbidden=[
                "ui.child_region_with_options(",
                "fret_ui::element::PointerRegionProps::default()",
                "install_collection_keyboard_handler(",
                "cx.pointer_region_on_wheel(",
                "cx.pointer_region_on_pointer_down(",
                "cx.pointer_region_on_pointer_move(",
                "cx.pointer_region_on_pointer_up(",
                "cx.pointer_region_on_pointer_cancel(",
                "proof_collection_box_select_active_rect(",
            ],
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
                "install_collection_keyboard_handler(",
                "pub(super) fn install_collection_keyboard_handler(",
                "cx.key_on_key_down_for(",
                "proof_collection_delete_key_matches(down.key)",
                "proof_collection_rename_shortcut_matches(down.key, down.modifiers)",
                "proof_collection_select_all_shortcut_matches(down.key, down.modifiers)",
                "proof_collection_duplicate_shortcut_matches(down.key, down.modifiers)",
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
                "models.scroll.viewport_size().width",
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
