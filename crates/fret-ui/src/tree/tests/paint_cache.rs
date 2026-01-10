use super::*;

#[test]
fn paint_cache_replays_ops_when_node_translates() {
    let mut app = crate::test_host::TestHost::new();

    let paints = Arc::new(AtomicUsize::new(0));
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_paint_cache_enabled(true);

    let node = ui.create_node(CountingPaintWidget {
        paints: paints.clone(),
    });
    ui.set_root(node);

    let mut services = FakeUiServices;
    let mut scene = Scene::default();

    let bounds_a = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(40.0)),
    );
    ui.paint_all(&mut app, &mut services, bounds_a, &mut scene, 1.0);
    assert_eq!(paints.load(Ordering::SeqCst), 1);
    assert_eq!(scene.ops_len(), 1);

    ui.ingest_paint_cache_source(&mut scene);
    scene.clear();

    let bounds_b = Rect::new(
        Point::new(fret_core::Px(20.0), fret_core::Px(15.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(40.0)),
    );
    ui.paint_all(&mut app, &mut services, bounds_b, &mut scene, 1.0);
    assert_eq!(paints.load(Ordering::SeqCst), 1);
    assert_eq!(scene.ops_len(), 3);

    match (scene.ops()[0], scene.ops()[1], scene.ops()[2]) {
        (
            SceneOp::PushTransform { transform },
            SceneOp::Quad { rect, .. },
            SceneOp::PopTransform,
        ) => {
            assert_eq!(transform.tx, bounds_b.origin.x.0 - bounds_a.origin.x.0);
            assert_eq!(transform.ty, bounds_b.origin.y.0 - bounds_a.origin.y.0);
            assert_eq!(rect, bounds_a);
        }
        _ => panic!("expected push-transform + quad + pop-transform ops"),
    }
}

#[test]
fn paint_cache_is_cleared_when_caching_is_disabled_for_a_node() {
    let mut app = crate::test_host::TestHost::new();

    let paints = Arc::new(AtomicUsize::new(0));
    let use_transform = Arc::new(AtomicBool::new(false));

    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_paint_cache_enabled(true);

    let node = ui.create_node(ToggleTransformPaintWidget {
        paints: paints.clone(),
        use_transform: use_transform.clone(),
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

    // Cache hit: paint is skipped and previous ops are replayed.
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(paints.load(Ordering::SeqCst), 1);

    ui.ingest_paint_cache_source(&mut scene);
    scene.clear();

    // Disable caching for the node (render transform present).
    use_transform.store(true, Ordering::SeqCst);
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(paints.load(Ordering::SeqCst), 2);

    ui.ingest_paint_cache_source(&mut scene);
    scene.clear();

    // Re-enable caching: should *not* replay the stale cache entry from the pre-transform frame.
    use_transform.store(false, Ordering::SeqCst);
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(paints.load(Ordering::SeqCst), 3);
}

struct ToggleTransformPaintWidget {
    paints: Arc<AtomicUsize>,
    use_transform: Arc<AtomicBool>,
}

impl<H: UiHost> Widget<H> for ToggleTransformPaintWidget {
    fn render_transform(&self, _bounds: Rect) -> Option<Transform2D> {
        self.use_transform
            .load(Ordering::SeqCst)
            .then_some(Transform2D::IDENTITY)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.paints.fetch_add(1, Ordering::SeqCst);
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: Color::TRANSPARENT,
            border: Edges::default(),
            border_color: Color::TRANSPARENT,
            corner_radii: Corners::default(),
        });
    }
}
