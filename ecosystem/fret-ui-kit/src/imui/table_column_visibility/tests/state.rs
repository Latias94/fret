use super::*;

#[test]
fn visibility_state_applies_runtime_overrides_by_stable_column_id() {
    let columns = vec![
        TableColumn::fill("Name###name"),
        TableColumn::px("Status###status", Px(96.0)),
        TableColumn::px("Owner###owner", Px(88.0)),
    ];
    let state = ImUiTableColumnVisibilityState::new([
        (Arc::from("status"), false),
        (Arc::from("owner"), true),
    ]);

    let applied = state.apply_to_columns(&columns);

    assert!(applied[0].visible());
    assert!(!applied[1].visible());
    assert!(applied[2].visible());
    assert_eq!(applied[1].id(), Some("status"));
    assert_eq!(state.visibility_for("status"), Some(false));
}

#[test]
fn visibility_state_leaves_unlisted_and_unidentified_columns_at_declared_visibility() {
    let columns = vec![
        TableColumn::fill("Name###name"),
        TableColumn::px("Static Hidden###hidden", Px(96.0)).hidden(),
        TableColumn::unlabeled(TableColumnWidth::px(Px(64.0))),
    ];
    let state = ImUiTableColumnVisibilityState::new([(Arc::from("name"), false)]);

    let applied = state.apply_to_columns(&columns);

    assert!(!applied[0].visible());
    assert!(!applied[1].visible());
    assert!(applied[2].visible());
}

#[test]
fn visibility_state_toggle_uses_current_override_or_default_visibility() {
    let mut state = ImUiTableColumnVisibilityState::default();

    assert!(!state.toggle("status", true));
    assert_eq!(state.visibility_for("status"), Some(false));
    assert!(state.toggle("status", true));
    assert_eq!(state.visibility_for("status"), Some(true));
    assert_eq!(state.remove("status"), Some(true));
    assert!(state.visibility_for("status").is_none());
}

#[test]
fn visibility_state_snapshot_roundtrips_stable_column_ids() {
    let state = ImUiTableColumnVisibilityState::new([
        (Arc::from("status"), false),
        (Arc::from("owner"), true),
    ]);

    let snapshot = state.snapshot();
    let encoded = serde_json::to_string(&snapshot).expect("snapshot should serialize");
    let decoded: TableColumnVisibilitySnapshot =
        serde_json::from_str(&encoded).expect("snapshot should deserialize");
    let restored = ImUiTableColumnVisibilityState::from_snapshot(decoded);

    assert_eq!(snapshot.columns().len(), 2);
    assert_eq!(snapshot.columns()[0].id(), "status");
    assert!(!snapshot.columns()[0].visible());
    assert_eq!(restored.visibility_for("status"), Some(false));
    assert_eq!(restored.visibility_for("owner"), Some(true));
}

#[test]
fn visibility_state_snapshot_restore_ignores_empty_ids_and_last_entry_wins() {
    let snapshot = TableColumnVisibilitySnapshot {
        columns: vec![
            TableColumnVisibilityEntry::new("", false),
            TableColumnVisibilityEntry::new("status", false),
            TableColumnVisibilityEntry::new("status", true),
            TableColumnVisibilityEntry::new("owner", false),
        ],
    };

    let mut state = ImUiTableColumnVisibilityState::new([("stale", false)]);
    state.replace_from_snapshot(snapshot);

    assert_eq!(state.len(), 2);
    assert!(state.visibility_for("").is_none());
    assert!(state.visibility_for("stale").is_none());
    assert_eq!(state.visibility_for("status"), Some(true));
    assert_eq!(state.visibility_for("owner"), Some(false));
}
