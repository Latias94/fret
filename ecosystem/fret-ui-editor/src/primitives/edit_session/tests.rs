use super::EditSession;

#[test]
fn changed_from_requires_an_active_session() {
    let session = EditSession::<String>::default();
    assert!(!session.changed_from(&"draft".to_string()));
}

#[test]
fn changed_from_reports_dirty_only_when_value_differs_from_pre_edit() {
    let mut session = EditSession::default();
    session.begin("before".to_string());

    assert!(!session.changed_from(&"before".to_string()));
    assert!(session.changed_from(&"after".to_string()));
}
