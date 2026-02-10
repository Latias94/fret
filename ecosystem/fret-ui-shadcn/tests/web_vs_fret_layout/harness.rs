use super::*;

pub(crate) fn assert_close_px(label: &str, actual: Px, expected: f32, tol: f32) {
    let delta = (actual.0 - expected).abs();
    assert!(
        delta <= tol,
        "{label}: expected≈{expected} (±{tol}) got={}",
        actual.0
    );
}

pub(crate) fn assert_rgba_close(label: &str, actual: Rgba, expected: Rgba, tol: f32) {
    let dr = (actual.r - expected.r).abs();
    let dg = (actual.g - expected.g).abs();
    let db = (actual.b - expected.b).abs();
    let da = (actual.a - expected.a).abs();
    assert!(
        dr <= tol && dg <= tol && db <= tol && da <= tol,
        "{label}: expected≈({:.3},{:.3},{:.3},{:.3}) got=({:.3},{:.3},{:.3},{:.3}) tol={tol}",
        expected.r,
        expected.g,
        expected.b,
        expected.a,
        actual.r,
        actual.g,
        actual.b,
        actual.a
    );
}

pub(crate) fn run_fret_root(
    bounds: Rect,
    f: impl FnOnce(&mut fret_ui::ElementContext<'_, App>) -> Vec<fret_ui::element::AnyElement>,
) -> fret_core::SemanticsSnapshot {
    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices::default();

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        f,
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot")
}

pub(crate) fn run_fret_root_frames(
    bounds: Rect,
    frames: usize,
    mut render: impl FnMut(&mut fret_ui::ElementContext<'_, App>) -> Vec<fret_ui::element::AnyElement>,
) -> fret_core::SemanticsSnapshot {
    assert!(frames > 0, "frames must be > 0");

    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices::default();

    let mut root: Option<NodeId> = None;
    let mut snapshot: Option<fret_core::SemanticsSnapshot> = None;

    for frame in 0..frames {
        let root_node = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "web-vs-fret-layout",
            |cx| render(cx),
        );

        root.get_or_insert(root_node);
        if frame == 0 {
            ui.set_root(root_node);
        }

        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        snapshot = ui.semantics_snapshot().cloned();

        // Runner-owned clocks are normally advanced by the platform event loop. In tests we
        // advance them explicitly so frame-lagged layout queries (ADR 1170) can settle.
        let next_frame = fret_runtime::FrameId(app.frame_id().0.saturating_add(1));
        let next_tick = fret_runtime::TickId(app.tick_id().0.saturating_add(1));
        app.set_frame_id(next_frame);
        app.set_tick_id(next_tick);
    }

    snapshot.expect("expected semantics snapshot")
}

pub(crate) fn run_fret_root_with_services(
    bounds: Rect,
    services: &mut dyn fret_core::UiServices,
    f: impl FnOnce(&mut fret_ui::ElementContext<'_, App>) -> Vec<fret_ui::element::AnyElement>,
) -> fret_core::SemanticsSnapshot {
    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        services,
        window,
        bounds,
        "web-vs-fret-layout",
        f,
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, services, bounds, 1.0);

    ui.semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot")
}

pub(crate) fn run_fret_root_frames_with_services(
    bounds: Rect,
    services: &mut dyn fret_core::UiServices,
    frames: usize,
    mut render: impl FnMut(&mut fret_ui::ElementContext<'_, App>) -> Vec<fret_ui::element::AnyElement>,
) -> fret_core::SemanticsSnapshot {
    assert!(frames > 0, "frames must be > 0");

    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let mut root: Option<NodeId> = None;
    let mut snapshot: Option<fret_core::SemanticsSnapshot> = None;

    for frame in 0..frames {
        let root_node = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            services,
            window,
            bounds,
            "web-vs-fret-layout",
            |cx| render(cx),
        );

        root.get_or_insert(root_node);
        if frame == 0 {
            ui.set_root(root_node);
        }

        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, services, bounds, &mut scene, 1.0);

        snapshot = ui.semantics_snapshot().cloned();

        // Runner-owned clocks are normally advanced by the platform event loop. In tests we
        // advance them explicitly so frame-lagged layout queries (ADR 1170) can settle.
        let next_frame = fret_runtime::FrameId(app.frame_id().0.saturating_add(1));
        let next_tick = fret_runtime::TickId(app.tick_id().0.saturating_add(1));
        app.set_frame_id(next_frame);
        app.set_tick_id(next_tick);
    }

    snapshot.expect("expected semantics snapshot")
}

pub(crate) fn run_fret_root_with_ui(
    bounds: Rect,
    f: impl FnOnce(&mut fret_ui::ElementContext<'_, App>) -> Vec<fret_ui::element::AnyElement>,
) -> (UiTree<App>, fret_core::SemanticsSnapshot, NodeId) {
    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices::default();

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        f,
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    (ui, snap, root)
}

pub(crate) fn render_and_paint_in_bounds(
    bounds: Rect,
    render: impl FnOnce(&mut fret_ui::ElementContext<'_, App>) -> Vec<fret_ui::element::AnyElement>,
) -> (fret_core::SemanticsSnapshot, Scene) {
    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    // Use style-aware text metrics so painted/layout-derived geometry is comparable to web goldens.
    // `FakeServices` intentionally returns constant 10x10 text metrics and will distort layout.
    let mut services = StyleAwareServices::default();

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        render,
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    (snap, scene)
}

pub(crate) fn run_fret_root_with_ui_and_services(
    bounds: Rect,
    services: &mut dyn fret_core::UiServices,
    f: impl FnOnce(&mut fret_ui::ElementContext<'_, App>) -> Vec<fret_ui::element::AnyElement>,
) -> (UiTree<App>, fret_core::SemanticsSnapshot, NodeId) {
    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        services,
        window,
        bounds,
        "web-vs-fret-layout",
        f,
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, services, bounds, 1.0);

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    (ui, snap, root)
}

pub(crate) fn run_fret_root_frames_with_ui_and_services(
    bounds: Rect,
    services: &mut dyn fret_core::UiServices,
    frames: usize,
    mut render: impl FnMut(&mut fret_ui::ElementContext<'_, App>) -> Vec<fret_ui::element::AnyElement>,
) -> (UiTree<App>, fret_core::SemanticsSnapshot, NodeId) {
    assert!(frames > 0, "frames must be > 0");

    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let mut root: Option<NodeId> = None;
    let mut snapshot: Option<fret_core::SemanticsSnapshot> = None;

    for frame in 0..frames {
        let root_node = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            services,
            window,
            bounds,
            "web-vs-fret-layout",
            |cx| render(cx),
        );

        root.get_or_insert(root_node);
        if frame == 0 {
            ui.set_root(root_node);
        }

        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, services, bounds, &mut scene, 1.0);

        snapshot = ui.semantics_snapshot().cloned();

        // Runner-owned clocks are normally advanced by the platform event loop. In tests we
        // advance them explicitly so frame-lagged layout queries (ADR 1170) can settle.
        let next_frame = fret_runtime::FrameId(app.frame_id().0.saturating_add(1));
        let next_tick = fret_runtime::TickId(app.tick_id().0.saturating_add(1));
        app.set_frame_id(next_frame);
        app.set_tick_id(next_tick);
    }

    let root = root.expect("expected root node");
    let snap = snapshot.expect("expected semantics snapshot");
    (ui, snap, root)
}

pub(crate) fn find_semantics<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
    role: SemanticsRole,
    label: Option<&str>,
) -> Option<&'a fret_core::SemanticsNode> {
    snap.nodes.iter().find(|n| {
        if n.role != role {
            return false;
        }
        if let Some(label) = label {
            return n.label.as_deref() == Some(label);
        }
        true
    })
}

pub(crate) fn find_by_test_id<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
    id: &str,
) -> &'a fret_core::SemanticsNode {
    snap.nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some(id))
        .unwrap_or_else(|| panic!("missing semantics node with test_id={id:?}"))
}

pub(crate) fn assert_panel_x_w_match(
    web_name: &str,
    label: &str,
    fret: &Rect,
    web: WebRect,
    tol: f32,
) {
    assert_close_px(&format!("{web_name} {label} x"), fret.origin.x, web.x, tol);
    assert_close_px(
        &format!("{web_name} {label} w"),
        fret.size.width,
        web.w,
        tol,
    );
}
