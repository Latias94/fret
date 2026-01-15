use super::*;

#[test]
fn view_cache_invalidation_stops_at_boundary_for_paint() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);

    let root = ui.create_node(TestStack::default());
    let a = ui.create_node(TestStack::default());
    let b = ui.create_node(TestStack::default());
    let c = ui.create_node(TestStack::default());

    ui.set_root(root);
    ui.set_children(root, vec![a]);
    ui.set_children(a, vec![b]);
    ui.set_children(b, vec![c]);

    for id in [root, a, b, c] {
        ui.nodes[id].invalidation.clear();
    }
    ui.nodes[b].view_cache.enabled = true;
    ui.nodes[b].view_cache.contained_layout = true;

    ui.invalidate(c, Invalidation::Paint);

    assert!(ui.nodes[c].invalidation.paint);
    assert!(ui.nodes[b].invalidation.paint);
    assert!(!ui.nodes[a].invalidation.paint);
    assert!(!ui.nodes[root].invalidation.paint);
}

#[test]
fn view_cache_disables_paint_cache_for_non_boundary_nodes() {
    let mut app = crate::test_host::TestHost::new();

    let paints = Arc::new(AtomicUsize::new(0));
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);
    ui.set_paint_cache_enabled(true);

    let node = ui.create_node(CountingPaintWidget {
        paints: paints.clone(),
    });
    ui.set_root(node);

    let mut services = FakeUiServices;
    let mut scene = Scene::default();
    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(40.0)),
    );

    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(paints.load(Ordering::SeqCst), 1);

    ui.ingest_paint_cache_source(&mut scene);
    scene.clear();

    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(paints.load(Ordering::SeqCst), 2);
}

#[test]
fn view_cache_allows_paint_cache_for_boundary_nodes() {
    let mut app = crate::test_host::TestHost::new();

    let paints = Arc::new(AtomicUsize::new(0));
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);
    ui.set_paint_cache_enabled(true);

    let node = ui.create_node(CountingPaintWidget {
        paints: paints.clone(),
    });
    ui.nodes[node].view_cache.enabled = true;
    ui.nodes[node].view_cache.contained_layout = true;
    ui.set_root(node);

    let mut services = FakeUiServices;
    let mut scene = Scene::default();
    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(40.0)),
    );

    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(paints.load(Ordering::SeqCst), 1);

    ui.ingest_paint_cache_source(&mut scene);
    scene.clear();

    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(paints.load(Ordering::SeqCst), 1);
}

#[test]
fn view_cache_runs_contained_relayout_for_invalidated_boundaries() {
    let mut app = crate::test_host::TestHost::new();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);

    let root = ui.create_node(TestStack::default());
    let boundary = ui.create_node(TestStack::default());
    ui.nodes[boundary].view_cache.enabled = true;
    ui.nodes[boundary].view_cache.contained_layout = true;

    ui.set_root(root);
    ui.set_children(root, vec![boundary]);

    let root_bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(40.0)),
    );

    ui.nodes[root].bounds = root_bounds;
    ui.nodes[root].measured_size = root_bounds.size;
    ui.nodes[root].invalidation.layout = false;

    ui.nodes[boundary].bounds = root_bounds;
    ui.nodes[boundary].invalidation.layout = true;

    let mut services = FakeUiServices;
    ui.layout_all(&mut app, &mut services, root_bounds, 1.0);
    assert!(!ui.nodes[boundary].invalidation.layout);
}
