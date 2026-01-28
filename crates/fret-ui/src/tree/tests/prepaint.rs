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

struct PrepaintCountStack {
    calls: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

impl<H: UiHost> Widget<H> for PrepaintCountStack {
    fn prepaint(&mut self, _cx: &mut crate::widget::PrepaintCx<'_, H>) {
        use std::sync::atomic::Ordering;
        self.calls.fetch_add(1, Ordering::SeqCst);
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        for &child in cx.children {
            let _ = cx.layout_in(child, cx.bounds);
        }
        cx.available
    }
}

#[test]
fn prepaint_hook_runs_for_view_cache_root_even_when_reusing_interaction_cache() {
    use std::sync::atomic::Ordering;

    let mut app = crate::test_host::TestHost::new();
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let root = ui.create_node(TestStack::default());
    let cache_root = ui.create_node(PrepaintCountStack {
        calls: calls.clone(),
    });
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
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(ui.debug_stats().interaction_cache_hits, 0);

    app.advance_frame();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert!(ui.debug_stats().interaction_cache_hits >= 1);
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

struct PrepaintActionStack;

impl<H: UiHost> Widget<H> for PrepaintActionStack {
    fn prepaint(&mut self, cx: &mut crate::widget::PrepaintCx<'_, H>) {
        cx.invalidate_self(crate::widget::Invalidation::Paint);
        cx.invalidate_self(crate::widget::Invalidation::HitTestOnly);
        cx.request_redraw();
        cx.request_animation_frame();
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        for &child in cx.children {
            let _ = cx.layout_in(child, cx.bounds);
        }
        cx.available
    }
}

#[test]
fn prepaint_actions_are_exported_to_debug_snapshot() {
    let mut app = crate::test_host::TestHost::new();
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let root = ui.create_node(TestStack::default());
    let cache_root = ui.create_node(PrepaintActionStack);
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

    let actions = ui.debug_prepaint_actions();
    assert!(
        actions
            .iter()
            .any(|a| a.kind == crate::tree::UiDebugPrepaintActionKind::Invalidate),
        "expected at least one prepaint invalidate action"
    );
    assert!(
        actions
            .iter()
            .any(|a| a.kind == crate::tree::UiDebugPrepaintActionKind::RequestRedraw),
        "expected at least one prepaint request_redraw action"
    );
    assert!(
        actions
            .iter()
            .any(|a| a.kind == crate::tree::UiDebugPrepaintActionKind::RequestAnimationFrame),
        "expected at least one prepaint request_animation_frame action"
    );
}
