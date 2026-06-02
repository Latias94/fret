use super::*;

#[test]
fn menu_group_filters_to_stable_human_labeled_columns() {
    let columns = [
        TableColumn::fill("Name###name"),
        TableColumn::unlabeled(TableColumnWidth::px(Px(64.0))).with_id("actions"),
        TableColumn::px("###internal", Px(48.0)),
        TableColumn::px("State###state", Px(80.0)),
    ];

    assert_eq!(menu_column_id(&columns[0]).as_deref(), Some("name"));
    assert_eq!(visible_menu_label(&columns[0]), Some("Name"));
    assert_eq!(menu_column_id(&columns[1]).as_deref(), Some("actions"));
    assert!(visible_menu_label(&columns[1]).is_none());
    assert_eq!(menu_column_id(&columns[2]).as_deref(), Some("internal"));
    assert!(visible_menu_label(&columns[2]).is_none());
    assert_eq!(visible_menu_label(&columns[3]), Some("State"));
}

#[test]
fn menu_group_test_id_suffix_uses_stable_column_id_slug() {
    assert_eq!(menu_test_id_suffix("asset-status", 7), "asset-status");
    assert_eq!(menu_test_id_suffix("Asset Status!", 7), "asset-status");
    assert_eq!(menu_test_id_suffix("###", 7), "7");
}
