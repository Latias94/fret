use super::*;

#[test]
fn prepaint_interaction_cache_replays_for_clean_view_cache_root() {
    let mut app = crate::test_host::TestHost::new();
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let root = ui.create_node(TestStack::default());
    let cache_root = ui.create_node(TestStack::default());
    let leaf = ui.create_node(TestStack::default());
    ui.set_root(root);
    ui.add_child(root, cache_root);
    ui.add_child(cache_root, leaf);

    ui.set_node_view_cache_flags(cache_root, true, false, false);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );

    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert_eq!(ui.debug_stats().interaction_cache_hits, 0);

    app.advance_frame();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let stats = ui.debug_stats();
    assert!(stats.interaction_cache_hits >= 1);
    assert!(stats.interaction_cache_replayed_records > 0);
}
