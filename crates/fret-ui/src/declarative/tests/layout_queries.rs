use super::*;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

#[test]
fn layout_query_bounds_are_frame_lagged_and_invalidate_view_cache_next_frame() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let width = Arc::new(Mutex::new(Px(80.0)));
    let reads = Arc::new(Mutex::new(Vec::<Option<Px>>::new()));
    let cached_reads = Arc::new(Mutex::new(Vec::<Option<Px>>::new()));
    let renders = Arc::new(AtomicUsize::new(0));
    let mut renders_after_settle: usize = 0;

    let mut root: Option<NodeId> = None;

    for frame in 0..4 {
        if frame == 2 {
            *width.lock().unwrap() = Px(140.0);
        }

        let width = width.clone();
        let reads = reads.clone();
        let cached_reads = cached_reads.clone();
        let renders_for_cache = renders.clone();

        let root_node = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "layout-query-frame-lag",
            move |cx| {
                let w = *width.lock().unwrap();

                let mut region_id: Option<crate::elements::GlobalElementId> = None;
                let region = cx.layout_query_region_with_id(
                    crate::element::LayoutQueryRegionProps::default(),
                    |cx, id| {
                        region_id = Some(id);

                        let mut container = crate::element::ContainerProps::default();
                        container.layout.size.width = Length::Px(w);
                        container.layout.size.height = Length::Px(Px(20.0));

                        vec![cx.container(container, |cx| vec![cx.text("region")])]
                    },
                );
                let region_id = region_id.expect("layout query region id should be recorded");

                let snapshot = cx
                    .layout_query_bounds(region_id, Invalidation::Layout)
                    .map(|r| r.size.width);
                reads.lock().unwrap().push(snapshot);

                let cached = cx.view_cache(crate::element::ViewCacheProps::default(), move |cx| {
                    renders_for_cache.fetch_add(1, Ordering::SeqCst);
                    let snapshot = cx
                        .layout_query_bounds(region_id, Invalidation::Layout)
                        .map(|r| r.size.width);
                    cached_reads.lock().unwrap().push(snapshot);
                    vec![cx.text("cached")]
                });

                vec![region, cached]
            },
        );

        root.get_or_insert(root_node);
        if frame == 0 {
            ui.set_root(root_node);
        }

        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        app.advance_frame();

        if frame == 1 {
            renders_after_settle = renders.load(Ordering::SeqCst);
        }
        if frame == 2 {
            assert_eq!(
                renders.load(Ordering::SeqCst),
                renders_after_settle,
                "expected view cache reuse in the same frame the region changes (revision is applied after layout)"
            );
        }
    }

    assert_eq!(
        renders.load(Ordering::SeqCst),
        renders_after_settle + 1,
        "expected layout query region size change to invalidate the cached subtree on the next frame"
    );

    let reads = reads.lock().unwrap();
    assert_eq!(reads.len(), 4);

    let width0 = Px(80.0);
    let width1 = Px(140.0);

    let frame1 = reads[1].expect("expected bounds to be available on frame 1");
    assert!(
        (frame1.0 - width0.0).abs() < 0.01,
        "expected frame 1 to see frame 0 bounds"
    );

    let frame2 = reads[2].expect("expected bounds to be available on frame 2");
    assert!(
        (frame2.0 - width0.0).abs() < 0.01,
        "expected frame 2 to see the previous frame bounds (frame lag)"
    );

    let frame3 = reads[3].expect("expected bounds to be available on frame 3");
    assert!(
        (frame3.0 - width1.0).abs() < 0.01,
        "expected frame 3 to see frame 2 bounds"
    );

    let cached_reads = cached_reads.lock().unwrap();
    let last = cached_reads
        .last()
        .copied()
        .expect("expected cached subtree to render at least once");
    let last = last.expect("expected cached subtree to see bounds after settle");
    assert!(
        (last.0 - width1.0).abs() < 0.01,
        "expected cached subtree to observe the resized region after invalidation"
    );
}
