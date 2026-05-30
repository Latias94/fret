use super::{enum_select_selection_commit_policy, sanitize_test_id_segment};

#[test]
fn enum_select_item_test_id_segment_is_stable_ascii() {
    assert_eq!(sanitize_test_id_segment("Lit"), "lit");
    assert_eq!(
        sanitize_test_id_segment("Material / Matcap"),
        "material-matcap"
    );
    assert_eq!(sanitize_test_id_segment("  "), "item");
}

#[test]
fn enum_select_commit_policy_does_not_toggle_selected_to_none() {
    let policy = enum_select_selection_commit_policy();

    assert!(!policy.toggle_selected_to_none);
    assert!(policy.close_on_commit);
    assert!(policy.clear_query_on_commit);
}
