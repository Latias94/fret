use fret_app::App;
use fret_core::{
    AppWindowId, Event, FrameId, Modifiers, MouseButton, Point, PointerEvent, PointerType, Rect,
    SemanticsRole, Size as CoreSize,
};
use fret_runtime::Effect;
use fret_runtime::Model;
use fret_ui::tree::UiTree;
use fret_ui_kit::{LayoutRefinement, MetricRef, OverlayController, Space};

#[path = "support/style_aware_services.rs"]
mod style_aware_services;
use style_aware_services::StyleAwareServices;

fn window_bounds() -> Rect {
    Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        CoreSize::new(fret_core::Px(320.0), fret_core::Px(120.0)),
    )
}

fn effects_request_raf(effects: &[Effect], window: AppWindowId) -> bool {
    effects
        .iter()
        .any(|e| matches!(e, Effect::RequestAnimationFrame(w) if *w == window))
}

fn bounds_center(r: Rect) -> Point {
    Point::new(
        fret_core::Px(r.origin.x.0 + r.size.width.0 * 0.5),
        fret_core::Px(r.origin.y.0 + r.size.height.0 * 0.5),
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
    frame_id: FrameId,
    api: Model<fret_ui_shadcn::CarouselApiSnapshot>,
    opts: fret_ui_shadcn::CarouselOptions,
) -> Vec<Effect> {
    app.set_frame_id(frame_id);
    OverlayController::begin_frame(app, window);
    let root = fret_ui::declarative::render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "carousel-reduced-motion",
        |cx| {
            let slides = (0..5).map(|_| cx.container(Default::default(), |_cx| vec![]));
            let carousel = fret_ui_shadcn::Carousel::new(slides)
                .opts(opts)
                .api_snapshot_model(api)
                .track_start_neg_margin(Space::N0)
                .item_padding_start(Space::N0)
                .item_basis_main_px(fret_core::Px(200.0))
                .refine_layout(
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(fret_core::Px(200.0)))
                        .h_px(MetricRef::Px(fret_core::Px(120.0))),
                )
                .refine_viewport_layout(
                    LayoutRefinement::default().h_px(MetricRef::Px(fret_core::Px(120.0))),
                )
                .into_element(cx);
            vec![carousel]
        },
    );
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
    app.flush_effects()
}

#[test]
fn carousel_respects_reduced_motion_and_does_not_request_continuous_frames() {
    let window = AppWindowId::default();
    let bounds = window_bounds();

    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
    app.with_global_mut(fret_ui::elements::ElementRuntime::new, |rt, _app| {
        rt.set_window_prefers_reduced_motion(window, Some(true));
    });
    app.with_global_mut(fret_ui::ElementRuntime::default, |rt, _app| {
        rt.set_window_prefers_reduced_motion(window, Some(true));
    });

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let api: Model<fret_ui_shadcn::CarouselApiSnapshot> = app
        .models_mut()
        .insert(fret_ui_shadcn::CarouselApiSnapshot::default());
    let opts = fret_ui_shadcn::CarouselOptions::new().embla_engine(true);

    let mut before = None;
    for i in 1..=12_u64 {
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(i),
            api.clone(),
            opts,
        );
        let snap = app.models().get_copied(&api).expect("api snapshot");
        if snap.snap_count > 1 && snap.can_scroll_next {
            before = Some(snap);
            break;
        }
    }
    let before = before.expect("expected measurable snaps and enabled next control");
    assert!(
        before.snap_count > 1,
        "expected measurable snaps (>1); snapshot={before:?}"
    );

    let snap = ui.semantics_snapshot_arc().expect("semantics snapshot");
    let next = find_semantics_by_label(&snap, SemanticsRole::Button, "Next slide");
    click_center(&mut ui, &mut app, &mut services, bounds_center(next.bounds));

    let _ = render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(10),
        api.clone(),
        opts,
    );

    let after = app.models().get_copied(&api).expect("api snapshot");
    assert_eq!(
        after.selected_index, 1,
        "expected Next click to advance even under reduced motion; snapshot={after:?}"
    );

    let effects = render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(11),
        api.clone(),
        opts,
    );
    assert!(
        !effects_request_raf(&effects, window),
        "expected reduced-motion carousel to not request animation frames continuously; effects={effects:?}"
    );
}
