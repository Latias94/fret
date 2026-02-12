use fret_diag_protocol::{DiagTransportMessageV1, UiActionScriptV1, UiActionScriptV2};

fn assert_script_v1_roundtrip(json: &str) {
    let script_1: UiActionScriptV1 = serde_json::from_str(json).expect("script v1 must parse");
    assert_eq!(script_1.schema_version, 1);

    let value_1 = serde_json::to_value(&script_1).expect("script v1 must serialize");
    let script_2: UiActionScriptV1 =
        serde_json::from_value(value_1.clone()).expect("script v1 must parse after serialize");
    let value_2 = serde_json::to_value(&script_2).expect("script v1 must serialize again");

    assert_eq!(value_1, value_2);
}

fn assert_script_v2_roundtrip(json: &str) {
    let script_1: UiActionScriptV2 = serde_json::from_str(json).expect("script v2 must parse");
    assert_eq!(script_1.schema_version, 2);

    let value_1 = serde_json::to_value(&script_1).expect("script v2 must serialize");
    let script_2: UiActionScriptV2 =
        serde_json::from_value(value_1.clone()).expect("script v2 must parse after serialize");
    let value_2 = serde_json::to_value(&script_2).expect("script v2 must serialize again");

    assert_eq!(value_1, value_2);
}

#[test]
fn script_v1_roundtrip_todo_baseline() {
    assert_script_v1_roundtrip(include_str!(
        "../../../tools/diag-scripts/todo-baseline.json"
    ));
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
fn script_v2_roundtrip_command_palette_shortcut_primary() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-command-palette-shortcut-primary.json"
    ));
}

#[test]
fn script_v2_roundtrip_chart_torture_pan_zoom() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-chart-torture-pan-zoom.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_button_with_icon_non_overlap() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-button-with-icon-non-overlap.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_input_group_text_non_overlap() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-input-group-text-non-overlap.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_card_description_no_early_wrap() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-card-description-no-early-wrap.json"
    ));
}

#[test]
fn script_v2_roundtrip_ui_gallery_table_retained_multi_sort_shift_click() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-table-retained-multi-sort-shift-click.json"
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
