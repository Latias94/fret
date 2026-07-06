#[path = "imui_editor_collection_modularization_surface/asset_grid.rs"]
mod asset_grid;
#[path = "imui_editor_collection_modularization_surface/assets.rs"]
mod assets;
#[path = "imui_editor_collection_modularization_surface/authoring_parity.rs"]
mod authoring_parity;
#[path = "imui_editor_collection_modularization_surface/box_select.rs"]
mod box_select;
#[path = "imui_editor_collection_modularization_surface/browser_input_runtime.rs"]
mod browser_input_runtime;
#[path = "imui_editor_collection_modularization_surface/browser_scope.rs"]
mod browser_scope;
#[path = "imui_editor_collection_modularization_surface/child_models.rs"]
mod child_models;
#[path = "imui_editor_collection_modularization_surface/chrome.rs"]
mod chrome;
#[path = "imui_editor_collection_modularization_surface/collection_module.rs"]
mod collection_module;
#[path = "imui_editor_collection_modularization_surface/command_buttons.rs"]
mod command_buttons;
#[path = "imui_editor_collection_modularization_surface/context_menu.rs"]
mod context_menu;
#[path = "imui_editor_collection_modularization_surface/demo_module.rs"]
mod demo_module;
#[path = "imui_editor_collection_modularization_surface/derived_state.rs"]
mod derived_state;
#[path = "imui_editor_collection_modularization_surface/drag_drop.rs"]
mod drag_drop;
#[path = "imui_editor_collection_modularization_surface/editor_owners.rs"]
mod editor_owners;
#[path = "imui_editor_collection_modularization_surface/geometry.rs"]
mod geometry;
#[path = "imui_editor_collection_modularization_surface/import_target.rs"]
mod import_target;
#[path = "imui_editor_collection_modularization_surface/keyboard.rs"]
mod keyboard;
#[path = "imui_editor_collection_modularization_surface/lifecycle.rs"]
mod lifecycle;
#[path = "imui_editor_collection_modularization_surface/model_owner.rs"]
mod model_owner;
#[path = "imui_editor_collection_modularization_surface/models.rs"]
mod models;
#[path = "imui_editor_collection_modularization_surface/order_toggle.rs"]
mod order_toggle;
#[path = "imui_editor_collection_modularization_surface/rename.rs"]
mod rename;
#[path = "imui_editor_collection_modularization_surface/render_states.rs"]
mod render_states;
#[path = "imui_editor_collection_modularization_surface/runtime_state.rs"]
mod runtime_state;
#[path = "imui_editor_collection_modularization_surface/selection.rs"]
mod selection;
#[path = "imui_editor_collection_modularization_surface/selection_commands.rs"]
mod selection_commands;
#[path = "imui_editor_collection_modularization_surface/selection_context_menu.rs"]
mod selection_context_menu;
#[path = "imui_editor_collection_modularization_surface/selection_delete.rs"]
mod selection_delete;
#[path = "imui_editor_collection_modularization_surface/selection_duplicate.rs"]
mod selection_duplicate;
#[path = "imui_editor_collection_modularization_surface/selection_duplicate_naming.rs"]
mod selection_duplicate_naming;
#[path = "imui_editor_collection_modularization_surface/selection_duplicate_selection.rs"]
mod selection_duplicate_selection;
#[path = "imui_editor_collection_modularization_surface/selection_keyboard.rs"]
mod selection_keyboard;
#[path = "imui_editor_collection_modularization_surface/selection_select_all.rs"]
mod selection_select_all;
#[path = "imui_editor_collection_modularization_surface/status_readouts.rs"]
mod status_readouts;

#[test]
fn imui_editor_proof_demo_routes_collection_proof_through_demo_local_module() {
    let demo_source = include_str!("../src/imui_editor_proof_demo.rs");
    let authoring_parity_source = include_str!("../src/imui_editor_proof_demo/authoring_parity.rs");
    let authoring_parity_models_source =
        include_str!("../src/imui_editor_proof_demo/authoring_parity/models.rs");
    let authoring_parity_surface_source =
        include_str!("../src/imui_editor_proof_demo/authoring_parity/surface.rs");
    let authoring_parity_common_source =
        include_str!("../src/imui_editor_proof_demo/authoring_parity/common.rs");
    let authoring_parity_declarative_source =
        include_str!("../src/imui_editor_proof_demo/authoring_parity/declarative.rs");
    let authoring_parity_imui_source =
        include_str!("../src/imui_editor_proof_demo/authoring_parity/imui.rs");
    let authoring_parity_shared_state_source =
        include_str!("../src/imui_editor_proof_demo/authoring_parity/shared_state.rs");
    let editor_state_source = include_str!("../src/imui_editor_proof_demo/editor_state.rs");
    let editor_advanced_router_source =
        include_str!("../src/imui_editor_proof_demo/editor_advanced.rs");
    let editor_advanced_surface_source =
        include_str!("../src/imui_editor_proof_demo/editor_advanced/surface.rs");
    let editor_gradient_source = include_str!("../src/imui_editor_proof_demo/editor_gradient.rs");
    let editor_inspector_source = include_str!("../src/imui_editor_proof_demo/editor_inspector.rs");
    let editor_model_owner_source =
        include_str!("../src/imui_editor_proof_demo/editor_model_owner.rs");
    let editor_material_router_source =
        include_str!("../src/imui_editor_proof_demo/editor_material.rs");
    let editor_material_surface_source =
        include_str!("../src/imui_editor_proof_demo/editor_material/surface.rs");
    let editor_object_router_source =
        include_str!("../src/imui_editor_proof_demo/editor_object.rs");
    let editor_object_surface_source =
        include_str!("../src/imui_editor_proof_demo/editor_object/surface.rs");
    let editor_text_assist_source =
        include_str!("../src/imui_editor_proof_demo/editor_text_assist.rs");
    let collection_source = include_str!("../src/imui_editor_proof_demo/collection.rs");
    let asset_grid_source = include_str!("../src/imui_editor_proof_demo/collection/asset_grid.rs");
    let asset_grid_tile_source =
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid/tile.rs");
    let asset_grid_actions_source =
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid/actions.rs");
    let asset_grid_chrome_source =
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid/chrome.rs");
    let asset_grid_inline_rename_source =
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid/inline_rename.rs");
    let asset_grid_inline_rename_actions_source = include_str!(
        "../src/imui_editor_proof_demo/collection/asset_grid/inline_rename/actions.rs"
    );
    let asset_grid_metadata_source =
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid/metadata.rs");
    let assets_source = include_str!("../src/imui_editor_proof_demo/collection/assets.rs");
    let browser_scope_source =
        include_str!("../src/imui_editor_proof_demo/collection/browser_scope.rs");
    let browser_scope_asset_grid_source =
        include_str!("../src/imui_editor_proof_demo/collection/browser_scope/asset_grid.rs");
    let browser_scope_chrome_source =
        include_str!("../src/imui_editor_proof_demo/collection/browser_scope/chrome.rs");
    let child_models_source =
        include_str!("../src/imui_editor_proof_demo/collection/child_models.rs");
    let chrome_source = include_str!("../src/imui_editor_proof_demo/collection/chrome.rs");
    let browser_input_runtime_source =
        include_str!("../src/imui_editor_proof_demo/collection/browser_scope/input_runtime.rs");
    let browser_input_box_select_runtime_source = include_str!(
        "../src/imui_editor_proof_demo/collection/browser_scope/input_runtime/box_select.rs"
    );
    let browser_input_box_select_session_source = include_str!(
        "../src/imui_editor_proof_demo/collection/browser_scope/input_runtime/box_select/session.rs"
    );
    let browser_input_box_select_session_tests_source = include_str!(
        "../src/imui_editor_proof_demo/collection/browser_scope/input_runtime/box_select/session/tests.rs"
    );
    let browser_input_box_select_session_tests_fixtures_source = include_str!(
        "../src/imui_editor_proof_demo/collection/browser_scope/input_runtime/box_select/session/tests/fixtures.rs"
    );
    let browser_input_context_menu_runtime_source = include_str!(
        "../src/imui_editor_proof_demo/collection/browser_scope/input_runtime/context_menu.rs"
    );
    let browser_input_context_menu_runtime_tests_source = include_str!(
        "../src/imui_editor_proof_demo/collection/browser_scope/input_runtime/context_menu/tests.rs"
    );
    let browser_input_context_menu_runtime_tests_fixtures_source = include_str!(
        "../src/imui_editor_proof_demo/collection/browser_scope/input_runtime/context_menu/tests/fixtures.rs"
    );
    let browser_input_zoom_runtime_source = include_str!(
        "../src/imui_editor_proof_demo/collection/browser_scope/input_runtime/zoom.rs"
    );
    let box_select_source = include_str!("../src/imui_editor_proof_demo/collection/box_select.rs");
    let box_select_tests_source =
        include_str!("../src/imui_editor_proof_demo/collection/box_select/tests.rs");
    let box_select_tests_fixtures_source =
        include_str!("../src/imui_editor_proof_demo/collection/box_select/tests/fixtures.rs");
    let command_buttons_source =
        include_str!("../src/imui_editor_proof_demo/collection/command_buttons.rs");
    let command_buttons_actions_source =
        include_str!("../src/imui_editor_proof_demo/collection/command_buttons/actions.rs");
    let command_buttons_chrome_source =
        include_str!("../src/imui_editor_proof_demo/collection/command_buttons/chrome.rs");
    let context_menu_source =
        include_str!("../src/imui_editor_proof_demo/collection/context_menu.rs");
    let context_menu_actions_source =
        include_str!("../src/imui_editor_proof_demo/collection/context_menu/actions.rs");
    let context_menu_chrome_source =
        include_str!("../src/imui_editor_proof_demo/collection/context_menu/chrome.rs");
    let derived_state_source =
        include_str!("../src/imui_editor_proof_demo/collection/derived_state.rs");
    let drag_drop_source = include_str!("../src/imui_editor_proof_demo/collection/drag_drop.rs");
    let drag_drop_tests_source =
        include_str!("../src/imui_editor_proof_demo/collection/drag_drop/tests.rs");
    let drag_drop_tests_fixtures_source =
        include_str!("../src/imui_editor_proof_demo/collection/drag_drop/tests/fixtures.rs");
    let geometry_source = include_str!("../src/imui_editor_proof_demo/collection/geometry.rs");
    let geometry_tests_source =
        include_str!("../src/imui_editor_proof_demo/collection/geometry/tests.rs");
    let geometry_zoom_source =
        include_str!("../src/imui_editor_proof_demo/collection/geometry/zoom.rs");
    let geometry_zoom_tests_source =
        include_str!("../src/imui_editor_proof_demo/collection/geometry/zoom/tests.rs");
    let geometry_zoom_tests_fixtures_source =
        include_str!("../src/imui_editor_proof_demo/collection/geometry/zoom/tests/fixtures.rs");
    let import_target_source =
        include_str!("../src/imui_editor_proof_demo/collection/import_target.rs");
    let keyboard_source = include_str!("../src/imui_editor_proof_demo/collection/keyboard.rs");
    let keyboard_actions_source =
        include_str!("../src/imui_editor_proof_demo/collection/keyboard/actions.rs");
    let lifecycle_source = include_str!("../src/imui_editor_proof_demo/collection/lifecycle.rs");
    let models_source = include_str!("../src/imui_editor_proof_demo/collection/models.rs");
    let model_owner_source =
        include_str!("../src/imui_editor_proof_demo/collection/model_owner.rs");
    let order_toggle_source =
        include_str!("../src/imui_editor_proof_demo/collection/order_toggle.rs");
    let readouts_source = include_str!("../src/imui_editor_proof_demo/collection/readouts.rs");
    let readout_status_source =
        include_str!("../src/imui_editor_proof_demo/collection/readouts/status.rs");
    let rename_source = include_str!("../src/imui_editor_proof_demo/collection/rename.rs");
    let rename_tests_source =
        include_str!("../src/imui_editor_proof_demo/collection/rename/tests.rs");
    let rename_tests_fixtures_source =
        include_str!("../src/imui_editor_proof_demo/collection/rename/tests/fixtures.rs");
    let rename_commit_source =
        include_str!("../src/imui_editor_proof_demo/collection/rename/commit.rs");
    let rename_commit_tests_source =
        include_str!("../src/imui_editor_proof_demo/collection/rename/commit/tests.rs");
    let rename_commit_tests_fixtures_source =
        include_str!("../src/imui_editor_proof_demo/collection/rename/commit/tests/fixtures.rs");
    let rename_focus_source =
        include_str!("../src/imui_editor_proof_demo/collection/rename/focus.rs");
    let render_states_source =
        include_str!("../src/imui_editor_proof_demo/collection/render_states.rs");
    let runtime_state_source =
        include_str!("../src/imui_editor_proof_demo/collection/runtime_state.rs");
    let selection_source = include_str!("../src/imui_editor_proof_demo/collection/selection.rs");
    let selection_context_menu_source =
        include_str!("../src/imui_editor_proof_demo/collection/selection/context_menu.rs");
    let selection_context_menu_tests_source =
        include_str!("../src/imui_editor_proof_demo/collection/selection/context_menu/tests.rs");
    let selection_context_menu_tests_fixtures_source = include_str!(
        "../src/imui_editor_proof_demo/collection/selection/context_menu/tests/fixtures.rs"
    );
    let selection_keyboard_source =
        include_str!("../src/imui_editor_proof_demo/collection/selection/keyboard.rs");
    let selection_keyboard_tests_source =
        include_str!("../src/imui_editor_proof_demo/collection/selection/keyboard/tests.rs");
    let selection_keyboard_tests_fixtures_source = include_str!(
        "../src/imui_editor_proof_demo/collection/selection/keyboard/tests/fixtures.rs"
    );
    let selection_keyboard_navigation_source =
        include_str!("../src/imui_editor_proof_demo/collection/selection/keyboard/navigation.rs");
    let selection_keyboard_navigation_tests_source = include_str!(
        "../src/imui_editor_proof_demo/collection/selection/keyboard/navigation/tests.rs"
    );
    let selection_keyboard_navigation_tests_fixtures_source = include_str!(
        "../src/imui_editor_proof_demo/collection/selection/keyboard/navigation/tests/fixtures.rs"
    );
    let selection_projection_source =
        include_str!("../src/imui_editor_proof_demo/collection/selection/projection.rs");
    let selection_select_all_source =
        include_str!("../src/imui_editor_proof_demo/collection/selection/select_all.rs");
    let selection_select_all_tests_source =
        include_str!("../src/imui_editor_proof_demo/collection/selection/select_all/tests.rs");
    let selection_select_all_tests_fixtures_source = include_str!(
        "../src/imui_editor_proof_demo/collection/selection/select_all/tests/fixtures.rs"
    );
    let selection_commands_source =
        include_str!("../src/imui_editor_proof_demo/collection/selection/commands.rs");
    let selection_delete_commands_source =
        include_str!("../src/imui_editor_proof_demo/collection/selection/commands/delete.rs");
    let selection_delete_commands_tests_source =
        include_str!("../src/imui_editor_proof_demo/collection/selection/commands/delete/tests.rs");
    let selection_delete_commands_tests_fixtures_source = include_str!(
        "../src/imui_editor_proof_demo/collection/selection/commands/delete/tests/fixtures.rs"
    );
    let selection_duplicate_commands_source =
        include_str!("../src/imui_editor_proof_demo/collection/selection/commands/duplicate.rs");
    let selection_duplicate_commands_tests_source = include_str!(
        "../src/imui_editor_proof_demo/collection/selection/commands/duplicate/tests.rs"
    );
    let selection_duplicate_naming_source = include_str!(
        "../src/imui_editor_proof_demo/collection/selection/commands/duplicate/naming.rs"
    );
    let selection_duplicate_naming_tests_source = include_str!(
        "../src/imui_editor_proof_demo/collection/selection/commands/duplicate/naming/tests.rs"
    );
    let selection_duplicate_naming_tests_fixtures_source = include_str!(
        "../src/imui_editor_proof_demo/collection/selection/commands/duplicate/naming/tests/fixtures.rs"
    );
    let selection_duplicate_selection_source = include_str!(
        "../src/imui_editor_proof_demo/collection/selection/commands/duplicate/selection.rs"
    );
    let selection_duplicate_selection_tests_source = include_str!(
        "../src/imui_editor_proof_demo/collection/selection/commands/duplicate/selection/tests.rs"
    );
    let selection_duplicate_selection_tests_fixtures_source = include_str!(
        "../src/imui_editor_proof_demo/collection/selection/commands/duplicate/selection/tests/fixtures.rs"
    );
    let status_readouts_source =
        include_str!("../src/imui_editor_proof_demo/collection/status_readouts.rs");

    demo_module::assert_demo_module_routing(demo_source);

    editor_owners::assert_editor_owner_split(
        editor_inspector_source,
        editor_object_router_source,
        editor_object_surface_source,
        editor_advanced_router_source,
        editor_advanced_surface_source,
        editor_gradient_source,
        editor_model_owner_source,
        editor_material_router_source,
        editor_material_surface_source,
        editor_state_source,
        editor_text_assist_source,
    );

    authoring_parity::assert_authoring_parity_owner_split(
        demo_source,
        authoring_parity_source,
        authoring_parity_models_source,
        authoring_parity_surface_source,
        authoring_parity_common_source,
        authoring_parity_declarative_source,
        authoring_parity_imui_source,
        authoring_parity_shared_state_source,
    );

    collection_module::assert_collection_module_routing(collection_source);

    chrome::assert_chrome_owner_split(collection_source, chrome_source);

    derived_state::assert_derived_state_owner_split(collection_source, derived_state_source);

    runtime_state::assert_runtime_state_owner_split(collection_source, runtime_state_source);

    child_models::assert_child_models_owner_split(collection_source, child_models_source);

    lifecycle::assert_lifecycle_owner_split(collection_source, lifecycle_source);

    render_states::assert_render_states_owner_split(collection_source, render_states_source);

    order_toggle::assert_order_toggle_owner_split(collection_source, order_toggle_source);

    status_readouts::assert_status_readouts_owner_split(
        collection_source,
        readouts_source,
        readout_status_source,
        status_readouts_source,
    );

    import_target::assert_import_target_owner_split(collection_source, import_target_source);

    assets::assert_assets_owner_split(assets_source);

    browser_scope::assert_browser_scope_owner_split(
        browser_scope_source,
        browser_scope_chrome_source,
        browser_scope_asset_grid_source,
    );

    browser_input_runtime::assert_browser_input_runtime_owner_split(
        browser_input_runtime_source,
        browser_input_box_select_runtime_source,
        browser_input_box_select_session_source,
        browser_input_box_select_session_tests_source,
        browser_input_box_select_session_tests_fixtures_source,
        browser_input_context_menu_runtime_source,
        browser_input_context_menu_runtime_tests_source,
        browser_input_context_menu_runtime_tests_fixtures_source,
        browser_input_zoom_runtime_source,
    );

    asset_grid::assert_asset_grid_owner_split(
        asset_grid_source,
        asset_grid_tile_source,
        asset_grid_actions_source,
        asset_grid_chrome_source,
        asset_grid_inline_rename_source,
        asset_grid_inline_rename_actions_source,
        asset_grid_metadata_source,
    );

    box_select::assert_box_select_owner_split(
        box_select_source,
        box_select_tests_source,
        box_select_tests_fixtures_source,
    );

    command_buttons::assert_command_buttons_owner_split(
        command_buttons_source,
        command_buttons_actions_source,
        command_buttons_chrome_source,
    );

    context_menu::assert_context_menu_owner_split(
        context_menu_source,
        context_menu_actions_source,
        context_menu_chrome_source,
    );

    drag_drop::assert_drag_drop_owner_split(
        drag_drop_source,
        drag_drop_tests_source,
        drag_drop_tests_fixtures_source,
    );

    geometry::assert_geometry_owner_split(
        geometry_source,
        geometry_tests_source,
        geometry_zoom_source,
        geometry_zoom_tests_source,
        geometry_zoom_tests_fixtures_source,
    );

    keyboard::assert_keyboard_owner_split(keyboard_source, keyboard_actions_source);

    models::assert_models_owner_split(models_source);

    model_owner::assert_model_owner_boundary(
        collection_source,
        model_owner_source,
        command_buttons_actions_source,
        context_menu_source,
        context_menu_actions_source,
        asset_grid_actions_source,
        rename_source,
    );

    rename::assert_rename_owner_split(
        rename_source,
        rename_tests_source,
        rename_tests_fixtures_source,
        rename_commit_source,
        rename_commit_tests_source,
        rename_commit_tests_fixtures_source,
        rename_focus_source,
    );

    selection::assert_selection_owner_split(selection_source, selection_projection_source);
    selection_commands::assert_selection_commands_owner_split(selection_commands_source);
    selection_keyboard::assert_selection_keyboard_owner_split(
        selection_keyboard_source,
        selection_keyboard_tests_source,
        selection_keyboard_tests_fixtures_source,
        selection_keyboard_navigation_source,
        selection_keyboard_navigation_tests_source,
        selection_keyboard_navigation_tests_fixtures_source,
    );
    selection_context_menu::assert_selection_context_menu_owner_split(
        selection_context_menu_source,
        selection_context_menu_tests_source,
        selection_context_menu_tests_fixtures_source,
    );
    selection_select_all::assert_selection_select_all_owner_split(
        selection_source,
        selection_select_all_source,
        selection_select_all_tests_source,
        selection_select_all_tests_fixtures_source,
    );

    selection_delete::assert_selection_delete_owner_split(
        selection_delete_commands_source,
        selection_delete_commands_tests_source,
        selection_delete_commands_tests_fixtures_source,
    );

    selection_duplicate::assert_selection_duplicate_owner_split(
        selection_duplicate_commands_source,
        selection_duplicate_commands_tests_source,
    );

    selection_duplicate_naming::assert_selection_duplicate_naming_owner_split(
        selection_duplicate_naming_source,
        selection_duplicate_naming_tests_source,
        selection_duplicate_naming_tests_fixtures_source,
    );

    selection_duplicate_selection::assert_selection_duplicate_selection_owner_split(
        selection_duplicate_selection_source,
        selection_duplicate_selection_tests_source,
        selection_duplicate_selection_tests_fixtures_source,
    );
}
