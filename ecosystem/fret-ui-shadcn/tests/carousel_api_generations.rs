use fret_app::App;
use fret_core::{
    AppWindowId, Event, FrameId, Modifiers, MouseButton, Point, PointerEvent, PointerType, Px,
    Rect, SemanticsRole, Size as CoreSize,
};
use fret_runtime::Model;
use fret_ui::tree::UiTree;
use fret_ui_kit::{LayoutRefinement, MetricRef, OverlayController, Space};

#[path = "support/style_aware_services.rs"]
mod style_aware_services;
use style_aware_services::StyleAwareServices;

fn window_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(320.0), Px(200.0)),
    )
}

fn bounds_center(r: Rect) -> Point {
    Point::new(
        Px(r.origin.x.0 + r.size.width.0 * 0.5),
        Px(r.origin.y.0 + r.size.height.0 * 0.5),
    )
}

fn click_center(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    center: Point,
) {
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId(0),
            position: center,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId(0),
            position: center,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );
}

fn find_semantics_by_label<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
    role: SemanticsRole,
    label: &str,
) -> &'a fret_core::SemanticsNode {
    snap.nodes
        .iter()
        .find(|n| n.role == role && n.label.as_deref() == Some(label))
        .unwrap_or_else(|| panic!("missing semantics node role={role:?} label={label:?}"))
}

fn render_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    api: Model<fret_ui_shadcn::CarouselApiSnapshot>,
    opts: fret_ui_shadcn::CarouselOptions,
    carousel_width: Px,
    item_basis_main_px: Px,
) {
    let next_frame = FrameId(app.frame_id().0.saturating_add(1));
    app.set_frame_id(next_frame);

    OverlayController::begin_frame(app, window);
    let root = fret_ui::declarative::render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "carousel-api-generations",
        move |cx| {
            let slides = (0..5).map(|_| cx.container(Default::default(), |_cx| vec![]));
            let carousel = fret_ui_shadcn::Carousel::new(slides)
                .opts(opts)
                .api_snapshot_model(api)
                .track_start_neg_margin(Space::N0)
                .item_padding_start(Space::N0)
                .item_basis_main_px(item_basis_main_px)
                .refine_layout(
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(carousel_width))
                        .h_px(MetricRef::Px(Px(120.0))),
                )
                .refine_viewport_layout(LayoutRefinement::default().h_px(MetricRef::Px(Px(120.0))))
                .into_element(cx);
            vec![carousel]
        },
    );
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
}

#[test]
fn carousel_api_select_generation_increments_on_next_click() {
    let window = AppWindowId::default();
    let bounds = window_bounds();

    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let api = app
        .models_mut()
        .insert(fret_ui_shadcn::CarouselApiSnapshot::default());
    let opts = fret_ui_shadcn::CarouselOptions::default();

    for _ in 0..3 {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            api.clone(),
            opts,
            Px(200.0),
            Px(200.0),
        );
    }

    let before = app.models().get_copied(&api).expect("api snapshot");
    assert!(
        before.snap_count > 0,
        "expected measurable snaps; snapshot={before:?}"
    );

    let snap = ui.semantics_snapshot_arc().expect("semantics snapshot");
    let next = find_semantics_by_label(&snap, SemanticsRole::Button, "Next slide");
    click_center(&mut ui, &mut app, &mut services, bounds_center(next.bounds));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        api.clone(),
        opts,
        Px(200.0),
        Px(200.0),
    );

    let after = app.models().get_copied(&api).expect("api snapshot");
    assert_eq!(
        after.selected_index, 1,
        "expected next click to advance; snapshot={after:?}"
    );
    assert_eq!(
        after.select_generation,
        before.select_generation.saturating_add(1),
        "expected select_generation to increment exactly once; before={before:?} after={after:?}"
    );
}

#[test]
fn carousel_api_reinit_generation_increments_on_geometry_change() {
    let window = AppWindowId::default();
    let bounds = window_bounds();

    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let api = app
        .models_mut()
        .insert(fret_ui_shadcn::CarouselApiSnapshot::default());
    let opts = fret_ui_shadcn::CarouselOptions::default();

    for _ in 0..3 {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            api.clone(),
            opts,
            Px(200.0),
            Px(200.0),
        );
    }

    let before = app.models().get_copied(&api).expect("api snapshot");
    assert!(
        before.snap_count > 0,
        "expected measurable snaps; snapshot={before:?}"
    );

    // The recipe derives geometry from the previous layout pass (`last_bounds_for_element`), so we
    // need at least two frames to observe a size change. Additionally, the observable `reInit`
    // signal is throttled, so we allow a few extra frames for the increment to become visible.
    let mut after = before;
    for _ in 0..8 {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            api.clone(),
            opts,
            Px(200.0),
            Px(240.0),
        );
        after = app.models().get_copied(&api).expect("api snapshot");
        if after.reinit_generation > before.reinit_generation {
            break;
        }
    }

    assert_eq!(
        after.reinit_generation,
        before.reinit_generation.saturating_add(1),
        "expected reinit_generation to increment once on geometry change; before={before:?} after={after:?}"
    );
}

#[test]
fn carousel_api_reinit_generation_throttles_during_continuous_geometry_changes() {
    let window = AppWindowId::default();
    let bounds = window_bounds();

    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let api = app
        .models_mut()
        .insert(fret_ui_shadcn::CarouselApiSnapshot::default());
    let opts = fret_ui_shadcn::CarouselOptions::default();

    for _ in 0..3 {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            api.clone(),
            opts,
            Px(200.0),
            Px(200.0),
        );
    }

    let before = app.models().get_copied(&api).expect("api snapshot");
    assert!(
        before.snap_count > 0,
        "expected measurable snaps; snapshot={before:?}"
    );

    // Simulate an interactive resize/geometry churn by alternating the item basis each frame.
    // We only require that re-init signals are throttled (not emitted on every frame).
    for i in 0..16 {
        let basis = if (i % 2) == 0 { Px(200.0) } else { Px(240.0) };
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            api.clone(),
            opts,
            Px(200.0),
            basis,
        );
    }

    let after = app.models().get_copied(&api).expect("api snapshot");
    let delta = after
        .reinit_generation
        .saturating_sub(before.reinit_generation);
    assert!(
        delta >= 1,
        "expected at least one re-init during geometry churn; before={before:?} after={after:?}"
    );
    assert!(
        delta <= 6,
        "expected re-init to be throttled during continuous geometry churn; before={before:?} after={after:?}"
    );
}
