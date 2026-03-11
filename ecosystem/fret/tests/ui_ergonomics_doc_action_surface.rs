const UI_ERGONOMICS_AND_INTEROP: &str =
    include_str!("../../../docs/ui-ergonomics-and-interop.md");

#[test]
fn ui_ergonomics_doc_uses_grouped_action_surface_names() {
    assert!(UI_ERGONOMICS_AND_INTEROP.contains("`cx.actions().locals::<A>(...)`"));
    assert!(UI_ERGONOMICS_AND_INTEROP.contains("`cx.actions().models::<A>(...)`"));
    assert!(UI_ERGONOMICS_AND_INTEROP.contains("`cx.actions().transient::<A>(...)`"));
    assert!(UI_ERGONOMICS_AND_INTEROP.contains("raw `on_action_notify`"));
    assert!(!UI_ERGONOMICS_AND_INTEROP.contains("`on_action_notify_locals`"));
    assert!(!UI_ERGONOMICS_AND_INTEROP.contains("`on_action_notify_models`"));
    assert!(!UI_ERGONOMICS_AND_INTEROP.contains("`on_action_notify_transient`"));
    assert!(!UI_ERGONOMICS_AND_INTEROP.contains("single-model aliases"));
}
