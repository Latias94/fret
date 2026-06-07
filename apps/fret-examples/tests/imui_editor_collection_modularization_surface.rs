#[test]
fn imui_editor_proof_demo_routes_collection_proof_through_demo_local_module() {
    let demo_source = include_str!("../src/imui_editor_proof_demo.rs");
    let authoring_parity_source = include_str!("../src/imui_editor_proof_demo/authoring_parity.rs");
    let authoring_parity_models_source =
        include_str!("../src/imui_editor_proof_demo/authoring_parity/models.rs");
    let collection_source = include_str!("../src/imui_editor_proof_demo/collection.rs");
    let asset_grid_source = include_str!("../src/imui_editor_proof_demo/collection/asset_grid.rs");
    let assets_source = include_str!("../src/imui_editor_proof_demo/collection/assets.rs");
    let browser_scope_source =
        include_str!("../src/imui_editor_proof_demo/collection/browser_scope.rs");
    let child_models_source =
        include_str!("../src/imui_editor_proof_demo/collection/child_models.rs");
    let chrome_source = include_str!("../src/imui_editor_proof_demo/collection/chrome.rs");
    let browser_input_runtime_source =
        include_str!("../src/imui_editor_proof_demo/collection/browser_scope/input_runtime.rs");
    let box_select_source = include_str!("../src/imui_editor_proof_demo/collection/box_select.rs");
    let command_buttons_source =
        include_str!("../src/imui_editor_proof_demo/collection/command_buttons.rs");
    let context_menu_source =
        include_str!("../src/imui_editor_proof_demo/collection/context_menu.rs");
    let derived_state_source =
        include_str!("../src/imui_editor_proof_demo/collection/derived_state.rs");
    let drag_drop_source = include_str!("../src/imui_editor_proof_demo/collection/drag_drop.rs");
    let geometry_source = include_str!("../src/imui_editor_proof_demo/collection/geometry.rs");
    let import_target_source =
        include_str!("../src/imui_editor_proof_demo/collection/import_target.rs");
    let keyboard_source = include_str!("../src/imui_editor_proof_demo/collection/keyboard.rs");
    let lifecycle_source = include_str!("../src/imui_editor_proof_demo/collection/lifecycle.rs");
    let models_source = include_str!("../src/imui_editor_proof_demo/collection/models.rs");
    let order_toggle_source =
        include_str!("../src/imui_editor_proof_demo/collection/order_toggle.rs");
    let rename_source = include_str!("../src/imui_editor_proof_demo/collection/rename.rs");
    let render_states_source =
        include_str!("../src/imui_editor_proof_demo/collection/render_states.rs");
    let runtime_state_source =
        include_str!("../src/imui_editor_proof_demo/collection/runtime_state.rs");
    let selection_source = include_str!("../src/imui_editor_proof_demo/collection/selection.rs");
    let selection_commands_source =
        include_str!("../src/imui_editor_proof_demo/collection/selection/commands.rs");
    let selection_delete_commands_source =
        include_str!("../src/imui_editor_proof_demo/collection/selection/commands/delete.rs");
    let selection_duplicate_commands_source =
        include_str!("../src/imui_editor_proof_demo/collection/selection/commands/duplicate.rs");
    let status_readouts_source =
        include_str!("../src/imui_editor_proof_demo/collection/status_readouts.rs");

    for needle in [
        "mod authoring_parity;",
        "mod collection;",
        "collection::render_collection_first_asset_browser_proof(ui);",
        "authoring_parity::drag_assets()",
    ] {
        assert!(
            demo_source.contains(needle),
            "imui_editor_proof_demo should keep the collection proof routed through demo-local owners; missing `{needle}`"
        );
    }

    for needle in [
        "fn proof_collection_assets_in_visible_order(",
        "fn authoring_parity_collection_assets() -> Arc<[ProofCollectionAsset]> {",
        "struct ProofCollectionAsset {",
        "fn proof_collection_drag_rect_normalizes_drag_direction()",
        "collection::authoring_parity_collection_assets()",
    ] {
        assert!(
            !demo_source.contains(needle),
            "imui_editor_proof_demo should not keep the collection implementation inline after modularization; unexpected `{needle}`"
        );
    }

    for needle in [
        "mod models;",
        "mod shared_state;",
        "pub(super) use models::{",
        "drag_assets",
        "outliner_items_model",
        "pub(super) use shared_state::render_shared_state;",
    ] {
        assert!(
            authoring_parity_source.contains(needle),
            "the demo-local authoring parity hub should re-export split owner surfaces; missing `{needle}`"
        );
    }

    for needle in [
        "pub(in super::super) fn drag_assets() -> Arc<[ProofDragAsset]> {",
        "super::super::collection::authoring_parity_collection_assets()",
        "pub(in super::super) fn outliner_items() -> Arc<[ProofOutlinerItem]> {",
        "pub(in super::super) fn outliner_items_model<H: UiHost>(",
    ] {
        assert!(
            authoring_parity_models_source.contains(needle),
            "the demo-local authoring parity model owner should own shared proof fixtures; missing `{needle}`"
        );
    }

    for needle in [
        "pub(super) fn render_collection_first_asset_browser_proof(",
        "ui: &mut ImUi<'_, '_, KernelApp>",
        "mod asset_grid;",
        "mod assets;",
        "mod browser_scope;",
        "mod box_select;",
        "mod child_models;",
        "mod chrome;",
        "mod command_buttons;",
        "mod context_menu;",
        "mod derived_state;",
        "mod drag_drop;",
        "mod geometry;",
        "mod import_target;",
        "mod keyboard;",
        "mod lifecycle;",
        "mod models;",
        "mod order_toggle;",
        "mod rename;",
        "mod render_states;",
        "mod runtime_state;",
        "mod selection;",
        "mod status_readouts;",
        "pub(super) use assets::{ProofCollectionAsset, authoring_parity_collection_assets};",
        "pub(super) use chrome::proof_collection_readout_text;",
        "use child_models::{ProofCollectionChildModels, proof_collection_child_models};",
        "use chrome::render_collection_header;",
        "use derived_state::proof_collection_derived_state;",
        "use import_target::render_collection_import_target;",
        "use lifecycle::clear_stale_collection_rename_session;",
        "use order_toggle::render_collection_order_toggle;",
        "use render_states::proof_collection_render_states;",
        "use runtime_state::proof_collection_runtime_state;",
        "render_collection_import_target(ui);",
        "render_collection_order_toggle(",
        "proof_collection_derived_state(",
        "proof_collection_runtime_state(",
        "proof_collection_child_models(&collection_runtime.models)",
        "proof_collection_render_states(",
        "clear_stale_collection_rename_session(",
        "use status_readouts::render_collection_status_readouts;",
        "render_collection_status_readouts(",
        "render_collection_header(ui);",
    ] {
        assert!(
            collection_source.contains(needle),
            "the demo-local collection module should keep the modularized implementation explicit; missing `{needle}`"
        );
    }

    for needle in [
        "pub(in super::super) fn proof_collection_readout_text(",
        "pub(super) fn render_collection_header(",
        "pub(super) fn proof_collection_section_label(",
        "Collection-first asset browser proof",
        "Stable keys keep browser selection pinned while visible order flips",
        "Background drag now draws a marquee and updates grid selection app-locally",
        "proof_compact_readout_element(cx, text, Arc::<str>::from(test_id))",
        "proof_section_chrome_label(cx, text, test_id)",
    ] {
        assert!(
            chrome_source.contains(needle),
            "the demo-local collection chrome owner should keep readout/title mounting explicit; missing `{needle}`"
        );
    }

    for needle in [
        "fn proof_collection_readout_text(",
        "fn render_collection_header(",
        "fn proof_collection_section_label(",
        "Collection-first asset browser proof",
        "Stable keys keep browser selection pinned while visible order flips",
        "Background drag now draws a marquee and updates grid selection app-locally",
        "proof_compact_readout_element(cx, text, Arc::<str>::from(test_id))",
        "proof_section_chrome_label(cx, text, test_id)",
    ] {
        assert!(
            !collection_source.contains(needle),
            "the collection root should route chrome/readout mounting through collection/chrome.rs; unexpected `{needle}`"
        );
    }

    for needle in [
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
    ] {
        assert!(
            derived_state_source.contains(needle),
            "the demo-local collection derived-state owner should keep visible asset/key/active/rename-ready projection explicit; missing `{needle}`"
        );
    }

    for needle in [
        "proof_collection_assets_in_visible_order(",
        "proof_collection_active_id(",
        "proof_collection_begin_rename_session(",
        "let collection_keys =",
        "let collection_active_id =",
        "let collection_rename_ready_session =",
    ] {
        assert!(
            !collection_source.contains(needle),
            "the collection root should route derived visible state through collection/derived_state.rs; unexpected `{needle}`"
        );
    }

    for needle in [
        "pub(super) struct ProofCollectionRuntimeState",
        "pub(super) struct ProofCollectionRuntimeModels",
        "pub(super) struct ProofCollectionRuntimeSnapshot",
        "pub(super) fn rename_session(&self) -> Option<&ProofCollectionRenameSession>",
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
    ] {
        assert!(
            runtime_state_source.contains(needle),
            "the demo-local collection runtime-state owner should keep model handles, selector snapshots, and layout projection explicit; missing `{needle}`"
        );
    }

    for needle in [
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
    ] {
        assert!(
            !collection_source.contains(needle),
            "the collection root should route runtime model/snapshot reads through collection/runtime_state.rs; unexpected `{needle}`"
        );
    }

    for needle in [
        "pub(super) struct ProofCollectionChildModels",
        "pub(super) fn proof_collection_child_models(",
        "models: &ProofCollectionRuntimeModels",
        "command_buttons: ProofCollectionCommandButtonModels {",
        "browser_scope: ProofCollectionBrowserScopeModels {",
        "context_menu: ProofCollectionContextMenuModels {",
        "assets: models.assets.clone()",
        "selection: models.selection.clone()",
        "keyboard: models.keyboard.clone()",
        "rename_session: models.rename_session.clone()",
        "scroll: models.scroll.clone()",
    ] {
        assert!(
            child_models_source.contains(needle),
            "the demo-local collection child-model owner should keep child model bundle projection explicit; missing `{needle}`"
        );
    }

    for needle in [
        "ProofCollectionCommandButtonModels {",
        "ProofCollectionBrowserScopeModels {",
        "ProofCollectionContextMenuModels {",
    ] {
        assert!(
            !collection_source.contains(needle),
            "the collection root should route child model bundle projection through collection/child_models.rs; unexpected `{needle}`"
        );
    }

    for needle in [
        "pub(super) fn clear_stale_collection_rename_session(",
        "models: &ProofCollectionRuntimeModels",
        "snapshot: &ProofCollectionRuntimeSnapshot",
        "assets: &[ProofCollectionAsset]",
        "snapshot.rename_session.as_ref()",
        "!assets.iter().any(|asset| asset.id == session.target_id)",
        ".update(&models.rename_session, |state| *state = None)",
        ".update(&models.rename_focus_pending, |state| *state = false)",
    ] {
        assert!(
            lifecycle_source.contains(needle),
            "the demo-local collection lifecycle owner should keep stale rename cleanup explicit; missing `{needle}`"
        );
    }

    for needle in [
        "snapshot.rename_session.as_ref()",
        "models.rename_session",
        "models.rename_focus_pending",
        ".update(&collection_runtime.models.rename_session",
        ".update(&collection_runtime.models.rename_focus_pending",
    ] {
        assert!(
            !collection_source.contains(needle),
            "the collection root should route stale rename cleanup through collection/lifecycle.rs; unexpected `{needle}`"
        );
    }

    for needle in [
        "pub(super) struct ProofCollectionRenderStates",
        "pub(super) fn proof_collection_render_states<'a>(",
        "runtime: &'a ProofCollectionRuntimeState",
        "state: &'a ProofCollectionDerivedState",
        "status_readouts: ProofCollectionStatusReadoutState {",
        "command_buttons: ProofCollectionCommandButtonState {",
        "browser_scope: ProofCollectionBrowserScopeState {",
        "rename_ready_session: state.rename_ready_session.as_ref()",
        "rename_session: runtime.snapshot.rename_session()",
        "rename_focus_pending: runtime.snapshot.rename_focus_pending",
    ] {
        assert!(
            render_states_source.contains(needle),
            "the demo-local collection render-state owner should keep child render-state projection explicit; missing `{needle}`"
        );
    }

    for needle in [
        "ProofCollectionStatusReadoutState {",
        "ProofCollectionCommandButtonState {",
        "ProofCollectionBrowserScopeState {",
        "collection_runtime.snapshot.rename_status.as_str()",
        "collection_runtime.snapshot.command_status.as_str()",
        "collection_runtime.snapshot.rename_session()",
        "collection_state.rename_ready_session.as_ref()",
    ] {
        assert!(
            !collection_source.contains(needle),
            "the collection root should route child render-state projection through collection/render_states.rs; unexpected `{needle}`"
        );
    }

    for needle in [
        "pub(super) fn render_collection_order_toggle(",
        "reverse_order_model: &Model<bool>",
        "if reverse_order {",
        "\"Show folder order\"",
        "\"Reverse visible order\"",
        "ui.button_with_options(",
        "kit::ButtonOptions {",
        "\"imui-editor-proof.authoring.imui.collection.order-toggle\"",
        "if !order_toggle.clicked()",
        ".update(reverse_order_model, |value| *value = !*value)",
        "!reverse_order",
    ] {
        assert!(
            order_toggle_source.contains(needle),
            "the demo-local collection order-toggle owner should keep reverse-order button logic explicit; missing `{needle}`"
        );
    }

    for needle in [
        "\"Show folder order\"",
        "\"Reverse visible order\"",
        "\"imui-editor-proof.authoring.imui.collection.order-toggle\"",
        "ui.button_with_options(",
        "kit::ButtonOptions {",
        ".update(&collection_reverse_order_model, |value| *value = !*value)",
    ] {
        assert!(
            !collection_source.contains(needle),
            "the collection root should route reverse-order button UI through collection/order_toggle.rs; unexpected `{needle}`"
        );
    }

    for needle in [
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
        "\"imui-editor-proof.authoring.imui.collection.assets-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.visible-order-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.selection-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.active-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.zoom-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.select-all-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.rename-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.context-menu-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.command-package-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.rename-status-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.command-status-readout\"",
    ] {
        assert!(
            status_readouts_source.contains(needle),
            "the demo-local collection status-readouts owner should keep readout mounting explicit; missing `{needle}`"
        );
    }

    for needle in [
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
        "\"imui-editor-proof.authoring.imui.collection.assets-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.visible-order-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.selection-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.active-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.zoom-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.select-all-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.rename-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.context-menu-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.command-package-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.rename-status-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.command-status-readout\"",
    ] {
        assert!(
            !collection_source.contains(needle),
            "the collection root should route status readouts through collection/status_readouts.rs; unexpected `{needle}`"
        );
    }

    for needle in [
        "pub(super) fn render_collection_import_target(",
        "authoring_parity_collection_drop_status_model(ui.cx_mut())",
        "ui.button_with_options(",
        "ui.drop_target::<ProofCollectionDragPayload>(import_trigger)",
        "proof_collection_drop_status(\"Delivered\", &payload)",
        "proof_collection_drop_status(\"Preview\", &payload)",
        "\"Compatible collection drag active\"",
        "\"imui-editor-proof.authoring.imui.collection.import-target\"",
        "\"imui-editor-proof.authoring.imui.collection.drop-status-readout\"",
    ] {
        assert!(
            import_target_source.contains(needle),
            "the demo-local collection import-target owner should keep import drop/status UI explicit; missing `{needle}`"
        );
    }

    for needle in [
        "ProofCollectionDragPayload",
        "proof_collection_drop_status(",
        "authoring_parity_collection_drop_status_model",
        "ui.drop_target::<",
        "\"imui-editor-proof.authoring.imui.collection.import-target\"",
        "\"imui-editor-proof.authoring.imui.collection.drop-status-readout\"",
    ] {
        assert!(
            !collection_source.contains(needle),
            "the collection root should route import target/drop-status UI through collection/import_target.rs; unexpected `{needle}`"
        );
    }

    for needle in [
        "pub(in super::super) struct ProofCollectionAsset {",
        "pub(in super::super) fn authoring_parity_collection_assets() -> Arc<[ProofCollectionAsset]> {",
        "ProofCollectionAsset {",
        "id: Arc::from(\"stone-albedo\")",
        "path: Arc::from(\"textures/stone/albedo.ktx2\")",
    ] {
        assert!(
            assets_source.contains(needle),
            "the demo-local collection assets owner should keep asset fixtures explicit; missing `{needle}`"
        );
    }

    for needle in [
        "pub(super) struct ProofCollectionBrowserScopeModels",
        "pub(super) struct ProofCollectionBrowserScopeState",
        "pub(super) fn render_collection_browser_scope(",
        "ui.child_region_with_options(",
        "proof_collection_browser_scope_pointer_props()",
        "install_collection_browser_scope_input_runtime(",
        "render_collection_asset_grid(",
        "imui-editor-proof.authoring.imui.collection.box-select.scope",
    ] {
        assert!(
            browser_scope_source.contains(needle),
            "the demo-local collection browser-scope owner should keep child-region pointer runtime explicit; missing `{needle}`"
        );
    }

    for needle in [
        "pub(super) struct ProofCollectionBrowserScopeInputModels",
        "pub(super) struct ProofCollectionBrowserScopeInputState",
        "pub(super) fn proof_collection_browser_scope_pointer_props()",
        "pub(super) fn install_collection_browser_scope_input_runtime(",
        "install_collection_keyboard_handler(",
        "cx.pointer_region_on_wheel(",
        "cx.pointer_region_on_pointer_down(",
        "cx.pointer_region_on_pointer_move(",
        "cx.pointer_region_on_pointer_up(",
        "cx.pointer_region_on_pointer_cancel(",
    ] {
        assert!(
            browser_input_runtime_source.contains(needle),
            "the demo-local collection browser input runtime owner should keep wheel/context/box-select handlers explicit; missing `{needle}`"
        );
    }

    for needle in [
        "pub(super) struct ProofCollectionAssetGridModels",
        "pub(super) struct ProofCollectionAssetGridState",
        "pub(super) fn render_collection_asset_grid(",
        "ui.grid_with_options(",
        "TextField::new(",
        "drag_preview_ghost_with_options(",
        "ProofCollectionRenderedItem {",
    ] {
        assert!(
            asset_grid_source.contains(needle),
            "the demo-local collection asset-grid owner should keep tile-grid interaction explicit; missing `{needle}`"
        );
    }

    for needle in [
        "pub(super) struct ProofCollectionBoxSelectSession",
        "pub(super) struct ProofCollectionBoxSelectState",
        "pub(super) struct ProofCollectionRenderedItem",
        "pub(super) fn proof_collection_box_select_selection(",
        "pub(super) fn proof_collection_box_select_active_rect(",
    ] {
        assert!(
            box_select_source.contains(needle),
            "the demo-local collection box-select owner should keep marquee selection state explicit; missing `{needle}`"
        );
    }

    for needle in [
        "pub(super) struct ProofCollectionCommandButtonModels",
        "pub(super) struct ProofCollectionCommandButtonState",
        "pub(super) fn render_collection_command_buttons(",
        "proof_collection_set_command_status(",
    ] {
        assert!(
            command_buttons_source.contains(needle),
            "the demo-local collection command-buttons owner should keep explicit command button routing separate; missing `{needle}`"
        );
    }

    for needle in [
        "pub(super) struct ProofCollectionContextMenuModels",
        "pub(super) fn render_collection_context_menu(",
        "PROOF_COLLECTION_CONTEXT_MENU_POPUP_ID",
        "ui.begin_popup_menu(",
        "kit::MenuItemOptions {",
    ] {
        assert!(
            context_menu_source.contains(needle),
            "the demo-local collection context-menu owner should keep popup workflow explicit; missing `{needle}`"
        );
    }

    for needle in [
        "pub(super) struct ProofCollectionDragPayload",
        "pub(super) fn proof_collection_drag_payload_for_asset(",
        "pub(super) fn proof_collection_drag_preview_title(",
        "pub(super) fn proof_collection_drag_preview_subtitle(",
        "pub(super) fn proof_collection_drop_status(",
    ] {
        assert!(
            drag_drop_source.contains(needle),
            "the demo-local collection drag/drop owner should keep payload and status projection explicit; missing `{needle}`"
        );
    }

    for needle in [
        "#[cfg(test)]",
        "fn proof_collection_drag_rect_normalizes_drag_direction() {",
    ] {
        assert!(
            geometry_source.contains(needle),
            "the demo-local collection geometry owner should keep the pure geometry test floor explicit; missing `{needle}`"
        );
    }

    for needle in [
        "pub(super) struct ProofCollectionKeyboardHandlerModels",
        "pub(super) fn install_collection_keyboard_handler(",
        "cx.key_on_key_down_for(",
        "proof_collection_keyboard_selection(",
    ] {
        assert!(
            keyboard_source.contains(needle),
            "the demo-local collection keyboard owner should keep scope keyboard dispatch explicit; missing `{needle}`"
        );
    }

    for needle in [
        "pub(super) fn authoring_parity_collection_selection_model<H: UiHost>(",
        "pub(super) fn authoring_parity_collection_assets_model<H: UiHost>(",
        "pub(super) fn authoring_parity_collection_scroll_handle<H: UiHost>(",
    ] {
        assert!(
            models_source.contains(needle),
            "the demo-local collection models owner should keep state slot registration explicit; missing `{needle}`"
        );
    }

    for needle in [
        "pub(super) struct ProofCollectionRenameSession",
        "pub(super) fn proof_collection_begin_rename_session(",
        "pub(super) fn proof_collection_commit_rename(",
        "pub(super) fn proof_collection_sync_inline_rename_focus<",
    ] {
        assert!(
            rename_source.contains(needle),
            "the demo-local collection rename owner should keep inline rename workflow state explicit; missing `{needle}`"
        );
    }

    for needle in [
        "mod commands;",
        "pub(super) use commands::{",
        "pub(super) struct ProofCollectionKeyboardState",
        "pub(super) fn proof_collection_assets_in_visible_order(",
        "pub(super) fn proof_collection_keyboard_selection(",
    ] {
        assert!(
            selection_source.contains(needle),
            "the demo-local collection selection owner should keep pure selection state and command delegation explicit; missing `{needle}`"
        );
    }

    for needle in [
        "mod delete;",
        "mod duplicate;",
        "pub(in super::super) use delete::{",
    ] {
        assert!(
            selection_commands_source.contains(needle),
            "the demo-local collection selection command hub should keep sub-owner re-exports explicit; missing `{needle}`"
        );
    }

    for needle in [
        "pub(in super::super::super) struct ProofCollectionDeleteResult",
        "pub(in super::super::super) fn proof_collection_delete_selection(",
        "pub(in super::super::super) fn proof_collection_delete_key_matches(",
        "proof_collection_delete_selection_removes_selected_assets_and_refocuses_next_visible_item",
    ] {
        assert!(
            selection_delete_commands_source.contains(needle),
            "the demo-local collection delete command owner should keep delete/refocus transitions explicit; missing `{needle}`"
        );
    }

    for needle in [
        "pub(in super::super::super) struct ProofCollectionDuplicateResult",
        "pub(in super::super::super) fn proof_collection_duplicate_selection(",
        "pub(in super::super::super) fn proof_collection_duplicate_shortcut_matches(",
        "fn proof_collection_unique_copy_text(",
        "proof_collection_duplicate_selection_reselects_visible_copies_and_preserves_active_copy",
    ] {
        assert!(
            selection_duplicate_commands_source.contains(needle),
            "the demo-local collection duplicate command owner should keep copy-suffix/reselect transitions explicit; missing `{needle}`"
        );
    }
}
