use fret_diag_protocol::{DiagTransportMessageV1, UiActionScriptV1, UiActionScriptV2};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn repo_root_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .to_path_buf()
}

fn resolve_script_redirects_inline_or_repo(json: &str) -> String {
    const MAX_REDIRECT_DEPTH: usize = 8;

    let mut current = json.to_string();
    let mut visited: BTreeSet<String> = BTreeSet::new();

    for _ in 0..=MAX_REDIRECT_DEPTH {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&current) else {
            return current;
        };

        let is_redirect = value
            .get("kind")
            .and_then(|v| v.as_str())
            .is_some_and(|s| s == "script_redirect");
        if !is_redirect {
            return current;
        }

        let schema_version = value
            .get("schema_version")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        if schema_version != 1 {
            return current;
        }

        let Some(to) = value.get("to").and_then(|v| v.as_str()) else {
            return current;
        };

        if !visited.insert(to.to_string()) {
            return current;
        }

        let path = repo_root_dir().join(PathBuf::from(to));
        current = std::fs::read_to_string(&path).unwrap_or(current);
    }

    current
}

fn assert_script_v1_roundtrip(json: &str) {
    let json = resolve_script_redirects_inline_or_repo(json);
    let script_1: UiActionScriptV1 = serde_json::from_str(&json).expect("script v1 must parse");
    assert_eq!(script_1.schema_version, 1);

    let value_1 = serde_json::to_value(&script_1).expect("script v1 must serialize");
    let script_2: UiActionScriptV1 =
        serde_json::from_value(value_1.clone()).expect("script v1 must parse after serialize");
    let value_2 = serde_json::to_value(&script_2).expect("script v1 must serialize again");

    assert_eq!(value_1, value_2);
}

fn assert_script_v2_roundtrip(json: &str) {
    let json = resolve_script_redirects_inline_or_repo(json);
    let script_1: UiActionScriptV2 = serde_json::from_str(&json).expect("script v2 must parse");
    assert_eq!(script_1.schema_version, 2);

    let value_1 = serde_json::to_value(&script_1).expect("script v2 must serialize");
    let script_2: UiActionScriptV2 =
        serde_json::from_value(value_1.clone()).expect("script v2 must parse after serialize");
    let value_2 = serde_json::to_value(&script_2).expect("script v2 must serialize again");

    assert_eq!(value_1, value_2);
}

#[test]
fn script_v2_roundtrip_todo_baseline() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/todo-baseline.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_popover_click_through_outside_press_focus_underlay() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/overlay/ui-gallery-popover-click-through-outside-press-focus-underlay.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_dropdown_nonmodal_outside_press_focus_underlay() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/overlay/ui-gallery-dropdown-nonmodal-outside-press-focus-underlay.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_motion_preset_runtime_token_mutation() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/motion-presets/ui-gallery-motion-preset-runtime-token-mutation.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_platform_preferences_runtime_environment_mutation() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/motion-presets/ui-gallery-platform-preferences-runtime-environment-mutation.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_item_vs_field_doc_intro_client721_startup_non_overlap() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/item/ui-gallery-item-vs-field-doc-intro-client721-startup-non-overlap.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_workspace_tabstrip_overflow_select_command() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/workspace-tabstrip/ui-gallery-workspace-tabstrip-overflow-select-command.json"
    ));
}

#[test]
fn script_v2_roundtrip_workspace_shell_demo_tab_cross_pane_move_to_end() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/workspace-shell-demo-tab-cross-pane-move-to-end.json"
    ));
}

#[test]
fn script_v2_roundtrip_workspace_shell_demo_tab_overflow_activate_hidden_smoke() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/workspace-shell-demo-tab-overflow-activate-hidden-smoke.json"
    ));
}

#[test]
fn script_v2_roundtrip_workspace_shell_demo_tab_close_button_dirty_shows_prompt_smoke() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/workspace-shell-demo-tab-close-button-dirty-shows-prompt-smoke.json"
    ));
}

#[test]
fn script_v2_roundtrip_workspace_shell_demo_tab_close_others_dirty_aggregation_smoke() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/workspace-shell-demo-tab-close-others-dirty-aggregation-smoke.json"
    ));
}

#[test]
fn script_v2_roundtrip_workspace_shell_demo_tab_close_cross_pane_button_ownership_smoke() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/workspace-shell-demo-tab-close-cross-pane-button-ownership-smoke.json"
    ));
}

#[test]
fn script_v2_roundtrip_workspace_shell_demo_tab_close_others_cross_pane_context_menu_ownership_smoke()
 {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/workspace-shell-demo-tab-close-others-cross-pane-context-menu-ownership-smoke.json"
    ));
}

#[test]
fn script_v2_roundtrip_workspace_shell_demo_window_close_dirty_aggregation_smoke() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-window-close-dirty-aggregation-smoke.json"
    ));
}

#[test]
fn script_v2_roundtrip_set_window_preferences_defaults() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    {
      "type": "set_window_preferences",
      "window": { "kind": "first_seen" },
      "color_scheme": { "kind": "clear" },
      "prefers_reduced_motion": { "kind": "set", "value": true },
      "text_scale_factor": { "kind": "set", "value": 1.25 }
    }
  ]
}"#,
    );
}

#[test]
fn script_v1_roundtrip_active_item_is_predicate() {
    assert_script_v1_roundtrip(
        r#"{
  "schema_version": 1,
  "steps": [
    {
      "type": "wait_until",
      "predicate": {
        "kind": "active_item_is",
        "container": { "kind": "test_id", "id": "listbox" },
        "item": { "kind": "test_id", "id": "item-a" }
      },
      "timeout_frames": 1
    }
  ]
}"#,
    );
}

#[test]
fn script_v1_roundtrip_active_item_is_none_predicate() {
    assert_script_v1_roundtrip(
        r#"{
  "schema_version": 1,
  "steps": [
    {
      "type": "wait_until",
      "predicate": {
        "kind": "active_item_is_none",
        "container": { "kind": "test_id", "id": "listbox" }
      },
      "timeout_frames": 1
    }
  ]
}"#,
    );
}

#[test]
fn script_v1_roundtrip_semantics_relation_predicates() {
    assert_script_v1_roundtrip(
        r#"{
  "schema_version": 1,
  "steps": [
    {
      "type": "wait_until",
      "predicate": {
        "kind": "semantics_relation_includes",
        "source": { "kind": "test_id", "id": "relation-source" },
        "relation": "labelled_by",
        "target": { "kind": "test_id", "id": "relation-label" }
      },
      "timeout_frames": 1
    },
    {
      "type": "assert",
      "predicate": {
        "kind": "semantics_relation_is_empty",
        "source": { "kind": "test_id", "id": "relation-source" },
        "relation": "controls"
      }
    }
  ]
}"#,
    );
}

#[test]
fn script_v1_roundtrip_window_style_effective_hit_test() {
    assert_script_v1_roundtrip(
        r#"{
  "schema_version": 1,
  "steps": [
    {
      "type": "wait_until",
      "predicate": {
        "kind": "window_style_effective_is",
        "window": { "kind": "current" },
        "style": { "hit_test": "passthrough_all", "opacity_alpha_u8": 128 }
      },
      "timeout_frames": 1
    }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_command_palette_shortcut_primary() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-command-palette-shortcut-primary.json"
    ));
}

#[test]
fn script_v2_roundtrip_press_keys() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    {
      "type": "press_keys",
      "keys": ["9", "0", "0", "0"]
    }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_ui_gallery_menubar_active_mnemonic() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/menubar/ui-gallery-menubar-active-mnemonic.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_menubar_escape_exits_active() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/menubar/ui-gallery-menubar-escape-exits-active.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_canvas_cull_torture_pan_zoom() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-canvas-cull-torture-pan-zoom.json"
    ));
}

#[test]
fn script_v2_roundtrip_chart_torture_pan_zoom() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-chart-torture-pan-zoom.json"
    ));
}

#[test]
fn script_v2_roundtrip_chart_torture_explicit_y_link_map() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-chart-torture-explicit-y-link-map.json"
    ));
}

#[test]
fn script_v2_roundtrip_chart_multi_axis_linked_domain_window_app_snapshot() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/charts/chart-multi-axis-linked-domain-window-app-snapshot.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_carousel_state_gates() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-events-select-gate.json"
    ));
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-events-reinit-gate.json"
    ));
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-plugin-autoplay-stop-on-last-snap-gate.json"
    ));
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-plugin-autoplay-stop-on-interaction-focus-gate.json"
    ));
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-rtl-controls-gate.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_node_graph_cull_torture_pan_zoom() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-node-graph-cull-torture-pan-zoom.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_node_graph_cull_window_shifts() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-node-graph-cull-window-shifts.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_node_graph_cull_window_no_shifts_small_pan() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-node-graph-cull-window-no-shifts-small-pan.json"
    ));
}

#[test]
fn script_v2_roundtrip_imui_editor_proof_advanced_axis_composites() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/imui-editor-proof-advanced-axis-composites.json"
    ));
}

#[test]
fn script_v2_roundtrip_imui_editor_proof_gradient_stop_lifecycle() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/imui-editor-proof-gradient-stop-lifecycle.json"
    ));
}

#[test]
fn script_v2_roundtrip_imui_editor_proof_numeric_input_validation() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/imui-editor-proof-numeric-input-validation.json"
    ));
}

#[test]
fn script_v2_roundtrip_imui_editor_proof_editor_components_screenshots_default() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/imui-editor-proof-editor-components-screenshots-default.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_button_with_icon_non_overlap() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-button-with-icon-non-overlap.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_shadcn_parity_seed_layout() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/shadcn-parity/ui-gallery-shadcn-parity-seed-layout.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_separator_decorative_hidden_semantics() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/separator/ui-gallery-separator-decorative-hidden-semantics.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_accordion_usage_toggle() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/accordion/ui-gallery-accordion-usage-toggle.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_accordion_focusable_disabled_keyboard_suppression() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/accordion/ui-gallery-accordion-focusable-disabled-keyboard-suppression.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_incoming_open_inject_smoke() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-incoming-open-inject-smoke.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_input_group_text_non_overlap() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-input-group-text-non-overlap.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_input_basic_and_file_long_text() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-input-basic-and-file-long-text.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_input_disabled_action_state() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/input/ui-gallery-input-disabled-action-state.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_input_required_invalid_semantics() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/input/ui-gallery-input-required-invalid-semantics.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_input_otp_invalid_required_semantics() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/input/ui-gallery-input-otp-invalid-required-semantics.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_date_picker_required_invalid_semantics() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/date-picker/ui-gallery-date-picker-required-invalid-semantics.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_textarea_required_invalid_semantics() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/textarea/ui-gallery-textarea-required-invalid-semantics.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_checkbox_disabled_action_state() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/checkbox/ui-gallery-checkbox-disabled-action-state.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_button_group_input_group_long_text() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-button-group-input-group-long-text.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_combobox_long_text_geometry() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-combobox-long-text-geometry.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_combobox_rtl_long_text_geometry() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-combobox-rtl-long-text-geometry.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_combobox_popup_trigger() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-combobox-popup-trigger.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_combobox_popup_trigger_bottom_room() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-combobox-popup-trigger-bottom-room.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_combobox_popup_doc_intro_non_overlap() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-combobox-popup-doc-intro-non-overlap.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_combobox_popup_doc_intro_short_startup_non_overlap() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-combobox-popup-doc-intro-short-startup-non-overlap.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_combobox_popup_doc_intro_logical994_startup_non_overlap() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-combobox-popup-doc-intro-logical994-startup-non-overlap.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_combobox_rtl_long_text_doc_intro_logical1083_startup_non_overlap()
{
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-combobox-rtl-long-text-doc-intro-logical1083-startup-non-overlap.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_combobox_rtl_long_text_doc_intro_client721_startup_non_overlap() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-combobox-rtl-long-text-doc-intro-client721-startup-non-overlap.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_combobox_full_page_startup_intro_non_overlap() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-combobox-full-page-startup-intro-non-overlap.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_combobox_auto_highlight_disabled_none_on_open() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-combobox-auto-highlight-disabled-none-on-open.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_combobox_auto_highlight_first_match() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-combobox-auto-highlight-first-match.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_command_palette_controlled_selection_arrowdown() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-command-palette-controlled-selection-arrowdown.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_command_palette_controlled_selection_value() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-command-palette-controlled-selection-value.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_command_docs_demo_long_query_text() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-command-docs-demo-long-query-text.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_command_basic_dialog_overlay_focus() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-command-basic-dialog-overlay-focus.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_command_scrollable_collection_metadata_mutation() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-command-scrollable-collection-metadata-mutation.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_command_retained_active_descendant_action_state() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/command/ui-gallery-command-retained-active-descendant-action-state.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_data_table_default_pagination_collection_metadata() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-data-table-default-pagination-collection-metadata.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_virtual_list_retained_collection_metadata_bounce() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-virtual-list-retained-collection-metadata-bounce.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_tree_retained_hierarchy_semantics_toggle() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-tree-retained-hierarchy-semantics-toggle.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_ai_file_tree_demo_toggle() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-ai-file-tree-demo-toggle.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_ai_file_tree_demo_actions() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-ai-file-tree-demo-actions.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_ai_file_tree_large_scroll() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-ai-file-tree-large-scroll.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_ai_file_tree_demo_screenshot_zinc_dark() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-ai-file-tree-demo-screenshot-zinc-dark.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_ai_conversation_demo_screenshot_zinc_dark() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-ai-conversation-demo-screenshot-zinc-dark.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_ai_conversation_demo_scroll_button() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-ai-conversation-demo-scroll-button.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_ai_transcript_torture_scroll() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-ai-transcript-torture-scroll.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_sonner_live_region_mutation() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-sonner-live-region-mutation.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_switch_read_only_action_state() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-switch-read-only-action-state.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_switch_read_only_dynamic_action_state() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-switch-read-only-dynamic-action-state.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_switch_command_gated_action_state() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-switch-command-gated-action-state.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_switch_choice_card_checked_state_mutation() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/switch/ui-gallery-switch-choice-card-checked-state-mutation.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_slider_numeric_action_state() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/slider/ui-gallery-slider-numeric-action-state.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_checkbox_table_mixed_state_action() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/checkbox/ui-gallery-checkbox-table-mixed-state-action.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_toggle_interaction_screenshots() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/toggle/ui-gallery-toggle-interaction-screenshots.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_card_description_no_early_wrap() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-card-description-no-early-wrap.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_markdown_span_link_gate_activate() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-markdown-span-link-gate-activate.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_typography_interactive_links_activation() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/typography/ui-gallery-typography-interactive-links-activation.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_avatar_dropdown_activate_open() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/avatar/ui-gallery-avatar-dropdown-activate-open.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_avatar_dropdown_focus_trigger() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/avatar/ui-gallery-avatar-dropdown-focus-trigger.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_avatar_dropdown_click_stable_open() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/avatar/ui-gallery-avatar-dropdown-click-stable-open.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_avatar_dropdown_activate_open_trigger() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/avatar/ui-gallery-avatar-dropdown-activate-open-trigger.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_table_retained_multi_sort_shift_click() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-table-retained-multi-sort-shift-click.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_table_retained_keyboard_typeahead() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-table-retained-keyboard-typeahead.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_table_retained_row_pinning_keep_pinned_true() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-table-retained-row-pinning-keep-pinned-true.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_table_retained_row_pinning_keep_pinned_false() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-table-retained-row-pinning-keep-pinned-false.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_table_retained_sort_select_scroll() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-table-retained-sort-select-scroll.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_table_retained_window_boundary_scroll() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-table-retained-window-boundary-scroll.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_data_table_retained_multi_sort_shift_click() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-data-table-retained-multi-sort-shift-click.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_data_table_retained_visibility_toggle() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-data-table-retained-visibility-toggle.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_data_table_retained_column_actions_menu() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-data-table-retained-column-actions-menu.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_data_table_retained_sort_select_scroll() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-data-table-retained-sort-select-scroll.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_data_table_retained_window_boundary_scroll() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-data-table-window-boundary-scroll-retained.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_data_table_view_cache_filter_shrink_vlist_inputs_change() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-view-cache-filter-shrink-vlist-inputs-change.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_view_cache_model_mutation_through_cache() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/view-cache/ui-gallery-view-cache-model-mutation-through-cache.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_view_cache_dynamic_text_mutation_through_cache() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/view-cache/ui-gallery-view-cache-dynamic-text-mutation-through-cache.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_hit_test_only_paint_cache_probe_sweep() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/diag/ui-gallery-hit-test-only-paint-cache-probe-sweep.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_data_table_listlike_pointer_selection() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-data-table-listlike-pointer-selection.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_data_table_retained_global_filter() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-data-table-retained-global-filter.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_data_table_retained_column_filter() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-data-table-retained-column-filter.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_data_table_retained_faceted_filter() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-data-table-retained-faceted-filter.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_data_table_retained_reset_filters() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-data-table-retained-reset-filters.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_data_table_retained_column_pinning_sticky_scroll() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-data-table-retained-column-pinning-sticky-scroll.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_data_table_retained_column_pinning_toggle() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-data-table-retained-column-pinning-toggle.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_data_table_guide_demo_select_and_row_actions() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-data-table-guide-demo-select-and-row-actions.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_data_table_guide_demo_checkbox_only_selection() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-data-table-guide-demo-checkbox-only-selection.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_select_commit_and_label_update() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/select/ui-gallery-select-commit-and-label-update.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_select_invalid_form_state() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/select/ui-gallery-select-invalid-form-state.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_form_submit_validation_semantics() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/form/ui-gallery-form-submit-validation-semantics.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_select_demo_open_layout() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/select/ui-gallery-select-demo-open-layout.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_scroll_area_expand_at_bottom() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scroll-area-expand-at-bottom.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_select_scrollable_placement_boundary() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/select/ui-gallery-select-scrollable-placement-boundary.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_combobox_responsive_resize_open_placement() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-responsive-resize-open-placement.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_combobox_placement_ownership_scroll_rtl() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-placement-ownership-scroll-rtl.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_resizable_multi_viewport_combobox_placement() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/resizable/ui-gallery-resizable-multi-viewport-combobox-placement.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_resizable_multi_viewport_select_placement() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/resizable/ui-gallery-resizable-multi-viewport-select-placement.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_resizable_view_cache_moving_combobox_root_boundary() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/resizable/ui-gallery-resizable-view-cache-moving-combobox-root-boundary.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_dialog_nested_combobox_modal_boundary() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/dialog/ui-gallery-dialog-nested-combobox-modal-boundary.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_combobox_typeahead_commit_banana() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-typeahead-commit-banana.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_tabs_selected_state_mutation() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/tabs/ui-gallery-tabs-selected-state-mutation.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_radio_group_checked_state_mutation() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery/radio-group/ui-gallery-radio-group-checked-state-mutation.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_data_table_toolbar_faceted_responsive() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-data-table-toolbar-faceted-responsive.json"
    ));
}

#[test]
fn script_v2_roundtrip_wait_bounds_stable() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    {
      "type": "wait_bounds_stable",
      "target": { "kind": "test_id", "id": "x" },
      "stable_frames": 4,
      "max_move_px": 0.5,
      "timeout_frames": 10
    }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_scroll_into_view_motion_check() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    {
      "type": "scroll_into_view",
      "container": { "kind": "test_id", "id": "content-scroll" },
      "target": { "kind": "test_id", "id": "rtl-section" },
      "motion_check": {
        "scroll_target": { "kind": "test_id", "id": "content-viewport" },
        "field": "y",
        "max_target_reverse_px": 1.25,
        "max_scroll_reverse_px": 1.25,
        "require_scroll_progress": true
      },
      "timeout_frames": 10
    }
  ]
}"#,
    );
}

#[test]
fn transport_message_roundtrip_envelope() {
    let message_1 = DiagTransportMessageV1 {
        schema_version: 1,
        r#type: "hello".to_string(),
        session_id: None,
        request_id: Some(1),
        payload: serde_json::json!({"client_kind":"tooling","capabilities":["inspect"]}),
    };

    let value_1 = serde_json::to_value(&message_1).expect("message must serialize");
    let message_2: DiagTransportMessageV1 =
        serde_json::from_value(value_1.clone()).expect("message must parse");
    let value_2 = serde_json::to_value(&message_2).expect("message must serialize again");

    assert_eq!(value_1, value_2);
}

#[test]
fn script_v2_roundtrip_click_modifiers() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    {
      "type": "click",
      "target": { "kind": "test_id", "id": "table_header_name" },
      "button": "left",
      "modifiers": { "shift": true }
    }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_platform_hover_detection_predicates() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    { "type": "wait_until", "predicate": { "kind": "platform_ui_window_hover_detection_is", "quality": "none" }, "timeout_frames": 1 },
    { "type": "assert", "predicate": { "kind": "known_window_count_is", "n": 1 } }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_click_count() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    {
      "type": "click",
      "target": { "kind": "test_id", "id": "x" },
      "click_count": 2
    }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_bounds_max_size_predicate() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    {
      "type": "wait_until",
      "predicate": {
        "kind": "bounds_max_size",
        "target": { "kind": "test_id", "id": "x" },
        "max_w_px": 100.0,
        "max_h_px": 20.0,
        "eps_px": 0.5
      },
      "timeout_frames": 1
    }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_text_font_stack_key_stable_predicate() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    {
      "type": "wait_until",
      "predicate": {
        "kind": "text_font_stack_key_stable",
        "stable_frames": 60
      },
      "timeout_frames": 120
    }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_font_catalog_populated_predicate() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    {
      "type": "wait_until",
      "predicate": {
        "kind": "font_catalog_populated"
      },
      "timeout_frames": 1800
    }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_system_font_rescan_idle_predicate() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    {
      "type": "wait_until",
      "predicate": {
        "kind": "system_font_rescan_idle"
      },
      "timeout_frames": 1800
    }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_set_window_outer_position() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    { "type": "set_window_outer_position", "x_px": 32.0, "y_px": 64.0 }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_set_window_outer_position_last_seen() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    {
      "type": "set_window_outer_position",
      "window": { "kind": "last_seen_other" },
      "x_px": 10.0,
      "y_px": 20.0
    }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_set_cursor_at_host_monitor() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    {
      "type": "set_cursor_at_host_monitor",
      "selector": "highest_scale_factor",
      "x_fraction": 0.5,
      "y_fraction": 0.5,
      "offset_x_px": 0.0,
      "offset_y_px": 0.0
    }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_set_window_inner_size_first_seen() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    {
      "type": "set_window_inner_size",
      "window": { "kind": "first_seen" },
      "width_px": 800.0,
      "height_px": 600.0
    }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_window_inner_size_predicate() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    {
      "type": "wait_until",
      "predicate": {
        "kind": "window_inner_size_approx_equal",
        "width_px": 375.0,
        "height_px": 240.0,
        "eps_px": 1.0
      },
      "timeout_frames": 60
    }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_set_window_style_hit_test_regions() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    {
      "type": "set_window_style",
      "style": {
        "hit_test": {
          "kind": "passthrough_regions",
          "regions": [
            { "kind": "rect", "x": 10.0, "y": 20.0, "width": 300.0, "height": 200.0 }
          ]
        }
      }
    }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_set_cursor_screen_pos() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    { "type": "set_cursor_screen_pos", "x_px": 100.0, "y_px": 120.0 }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_set_cursor_in_window_last_seen_other() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    { "type": "set_cursor_in_window", "window": { "kind": "last_seen_other" }, "x_px": 100.0, "y_px": 120.0 }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_click_window_target() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    { "type": "click", "window": { "kind": "last_seen_other" }, "target": { "kind": "test_id", "id": "x" } }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_drag_pointer_window_target() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    { "type": "drag_pointer", "window": { "kind": "last_seen_other" }, "target": { "kind": "test_id", "id": "x" }, "delta_x": 10.0, "delta_y": 0.0 }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_drag_to_window_target() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    {
      "type": "drag_to",
      "window": { "kind": "last_seen_other" },
      "from": { "kind": "test_id", "id": "a" },
      "to": { "kind": "test_id", "id": "b" }
    }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_pointer_down_window_target() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    { "type": "pointer_down", "window": { "kind": "last_seen_other" }, "target": { "kind": "test_id", "id": "x" } }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_pointer_move_window_target() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    { "type": "pointer_move", "window": { "kind": "last_seen_other" }, "delta_x": 10.0, "delta_y": 0.0 }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_pointer_up_window_target() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    { "type": "pointer_up", "window": { "kind": "last_seen_other" } }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_pointer_cancel_window_target() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    { "type": "pointer_cancel", "window": { "kind": "last_seen_other" } }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_wait_until_window_target() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    {
      "type": "wait_until",
      "window": { "kind": "last_seen_other" },
      "predicate": { "kind": "exists", "target": { "kind": "test_id", "id": "x" } },
      "timeout_frames": 60
    }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_assert_window_target() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    {
      "type": "assert",
      "window": { "kind": "last_seen_other" },
      "predicate": { "kind": "known_window_count_ge", "n": 2 }
    }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_dock_drag_active_is_predicate() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    { "type": "assert", "predicate": { "kind": "dock_drag_active_is", "active": false } }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_dock_viewport_capture_active_is_predicate() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    { "type": "assert", "predicate": { "kind": "dock_viewport_capture_active_is", "active": false } }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_dock_drag_payload_ghost_visible_predicate() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    { "type": "assert", "predicate": { "kind": "dock_drag_payload_ghost_visible_is", "visible": true } }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_dock_tab_strip_active_overflow_is_predicate() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    { "type": "assert", "predicate": { "kind": "dock_tab_strip_active_overflow_is", "overflow": true } }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_dock_tab_strip_active_visible_is_predicate() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    { "type": "assert", "predicate": { "kind": "dock_tab_strip_active_visible_is", "visible": true } }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_dock_drag_transparent_payload_applied_predicate() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    {
      "type": "assert",
      "predicate": { "kind": "dock_drag_transparent_payload_applied_is", "applied": true }
    }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_dock_drag_transparent_payload_hit_test_passthrough_applied_predicate() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    {
      "type": "assert",
      "predicate": { "kind": "dock_drag_transparent_payload_hit_test_passthrough_applied_is", "applied": true }
    }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_dock_drag_window_under_cursor_source_is_predicate() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    {
      "type": "assert",
      "predicate": { "kind": "dock_drag_window_under_cursor_source_is", "source": "platform" }
    }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_dock_drag_moving_window_is_predicate() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    {
      "type": "assert",
      "predicate": { "kind": "dock_drag_moving_window_is", "window": { "kind": "last_seen" } }
    }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_dock_drag_kind_is_predicate() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    {
      "type": "assert",
      "predicate": { "kind": "dock_drag_kind_is", "drag_kind": "dock_panel" }
    }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_dock_drag_window_under_moving_window_is_predicate() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    {
      "type": "assert",
      "predicate": { "kind": "dock_drag_window_under_moving_window_is", "window": { "kind": "first_seen" } }
    }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_dock_drag_window_under_moving_window_source_is_predicate() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    {
      "type": "assert",
      "predicate": { "kind": "dock_drag_window_under_moving_window_source_is", "source": "platform" }
    }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_dock_graph_signature_predicates() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    {
      "type": "assert",
      "predicate": {
        "kind": "dock_graph_signature_is",
        "signature": "dock(root=split(v,[tabs([a]),tabs([b])]);floatings=[])"
      }
    },
    {
      "type": "assert",
      "predicate": {
        "kind": "dock_graph_signature_contains",
        "needle": "tabs([a])"
      }
    },
    {
      "type": "assert",
      "predicate": {
        "kind": "dock_graph_signature_fingerprint64_is",
        "fingerprint64": 42
      }
    }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_raise_window_last_seen_other() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    { "type": "raise_window", "window": { "kind": "last_seen_other" } }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_drag_pointer_until_known_window_count() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    {
      "type": "drag_pointer_until",
      "target": { "kind": "test_id", "id": "x" },
      "delta_x": 10.0,
      "delta_y": 0.0,
      "predicate": { "kind": "known_window_count_ge", "n": 2 },
      "timeout_frames": 10
    }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_drag_pointer_until_window_target() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    {
      "type": "drag_pointer_until",
      "window": { "kind": "last_seen_other" },
      "target": { "kind": "test_id", "id": "x" },
      "delta_x": 10.0,
      "delta_y": 0.0,
      "predicate": { "kind": "known_window_count_ge", "n": 2 },
      "timeout_frames": 10
    }
  ]
}"#,
    );
}

#[test]
fn script_v2_roundtrip_drag_pointer_until_dock_drag_current_window_is() {
    assert_script_v2_roundtrip(
        r#"{
  "schema_version": 2,
  "steps": [
    {
      "type": "drag_pointer_until",
      "target": { "kind": "test_id", "id": "x" },
      "delta_x": 10.0,
      "delta_y": 0.0,
      "predicate": { "kind": "dock_drag_current_window_is", "window": { "kind": "last_seen_other" } },
      "timeout_frames": 10
    }
  ]
}"#,
    );
}
