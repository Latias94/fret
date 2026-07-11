use fret_diag_protocol::{UiCommandDispatchTraceEntryV1, UiCommandDispatchTraceQueryV1};

#[test]
fn command_dispatch_trace_entry_has_a_forward_compatible_construction_path() {
    let mut entry = UiCommandDispatchTraceEntryV1::for_command(7, 42, "workspace.tab.close");
    entry.action_id = Some("workspace.tab.close".to_string());
    entry.target = Some("pane-a/doc-a".to_string());
    entry.applied = Some(false);
    entry.blocked_dirty_close = Some(true);
    entry.source_kind = "pointer".to_string();
    entry.source_test_id = Some("workspace.tab.close-button".to_string());

    let json = serde_json::to_value(&entry).expect("trace entry serializes");
    assert_eq!(json["step_index"], 7);
    assert_eq!(json["frame_id"], 42);
    assert_eq!(json["command"], "workspace.tab.close");
    assert_eq!(json["action_id"], "workspace.tab.close");
    assert_eq!(json["blocked_dirty_close"], true);

    let decoded: UiCommandDispatchTraceEntryV1 =
        serde_json::from_value(json).expect("trace entry round-trips");
    assert_eq!(decoded.target.as_deref(), Some("pane-a/doc-a"));
}

#[test]
fn command_dispatch_trace_query_has_a_forward_compatible_construction_path() {
    let mut query = UiCommandDispatchTraceQueryV1::for_command("workspace.tab.close");
    query.action_id = Some("workspace.tab.close".to_string());
    query.applied = Some(true);
    query.handled_by_driver = Some(true);

    let json = serde_json::to_value(&query).expect("trace query serializes");
    assert_eq!(json["command"], "workspace.tab.close");
    assert_eq!(json["action_id"], "workspace.tab.close");
    assert_eq!(json["applied"], true);
    assert_eq!(json["handled_by_driver"], true);
    assert!(json.get("source_test_id").is_none());
}
