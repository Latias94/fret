use std::sync::Arc;

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
        CoreSize::new(fret_core::Px(320.0), fret_core::Px(160.0)),
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

fn rect_overlap_area(a: Rect, b: Rect) -> f32 {
    let ax2 = a.origin.x.0 + a.size.width.0;
    let ay2 = a.origin.y.0 + a.size.height.0;
    let bx2 = b.origin.x.0 + b.size.width.0;
    let by2 = b.origin.y.0 + b.size.height.0;

    let ix1 = a.origin.x.0.max(b.origin.x.0);
    let iy1 = a.origin.y.0.max(b.origin.y.0);
    let ix2 = ax2.min(bx2);
    let iy2 = ay2.min(by2);

    let iw = (ix2 - ix1).max(0.0);
    let ih = (iy2 - iy1).max(0.0);
    iw * ih
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
    click_count_model: Model<u32>,
) -> Vec<Effect> {
    app.set_frame_id(frame_id);
    OverlayController::begin_frame(app, window);
    let root = fret_ui::declarative::render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "carousel-drag-free-prevent-click",
        |cx| {
            let label: Arc<str> = Arc::from("Slide Button");
            let slides = (0..5).map(|_i| {
                let click_count_model = click_count_model.clone();
                let on_activate = Arc::new(
                    move |host: &mut dyn fret_ui::action::UiActionHost, _cx, _reason| {
                        let _ = host.models_mut().update(&click_count_model, |v| {
                            *v = v.saturating_add(1);
                        });
                    },
                );
                cx.container(Default::default(), |cx| {
                    vec![
                        fret_ui_shadcn::Button::new(label.clone())
                            .on_activate(on_activate)
                            .into_element(cx),
                    ]
                })
            });
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
fn carousel_drag_free_mouse_click_while_moving_is_prevented() {
    let window = AppWindowId::default();
    let bounds = window_bounds();

    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let api: Model<fret_ui_shadcn::CarouselApiSnapshot> = app
        .models_mut()
        .insert(fret_ui_shadcn::CarouselApiSnapshot::default());
    let click_count_model: Model<u32> = app.models_mut().insert(0);

    let opts = fret_ui_shadcn::CarouselOptions::new()
        .embla_engine(true)
        .drag_free(true);

    let mut ready = None;
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
            click_count_model.clone(),
        );
        let snap = app.models().get_copied(&api).expect("api snapshot");
        if snap.snap_count > 1 && snap.can_scroll_next {
            ready = Some(snap);
            break;
        }
    }
    let _ = ready.expect("expected measurable snaps and enabled next control");

    // Prove slide content is normally clickable before starting motion.
    let viewport_rect = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        CoreSize::new(fret_core::Px(200.0), fret_core::Px(120.0)),
    );
    let pick_slide_button_center = |snap: &fret_core::SemanticsSnapshot| -> Point {
        let candidates = snap
            .nodes
            .iter()
            .filter(|n| {
                n.role == SemanticsRole::Button && n.label.as_deref() == Some("Slide Button")
            })
            .collect::<Vec<_>>();
        assert!(
            !candidates.is_empty(),
            "expected slide buttons in semantics"
        );
        let best = candidates
            .into_iter()
            .max_by(|a, b| {
                rect_overlap_area(a.bounds, viewport_rect)
                    .total_cmp(&rect_overlap_area(b.bounds, viewport_rect))
            })
            .expect("best slide button");
        assert!(
            rect_overlap_area(best.bounds, viewport_rect) > 0.0,
            "expected at least one slide button to overlap the viewport"
        );
        bounds_center(best.bounds)
    };

    let snap = ui.semantics_snapshot_arc().expect("semantics snapshot");
    let slide_button_center = pick_slide_button_center(&snap);
    click_center(&mut ui, &mut app, &mut services, slide_button_center);
    let count = app
        .models()
        .get_copied(&click_count_model)
        .expect("click count");
    assert_eq!(count, 1, "expected slide content to be clickable");
    let _ = app.models_mut().update(&click_count_model, |v| *v = 0_u32);

    // Start an Embla-style move (creates the engine and enters embla_settling).
    let snap = ui.semantics_snapshot_arc().expect("semantics snapshot");
    let next = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Next slide"))
        .expect("Next slide button");
    click_center(&mut ui, &mut app, &mut services, bounds_center(next.bounds));

    // Let the engine run for a couple of frames so `abs(target - location) >= 2` is true.
    let effects1 = render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(50),
        api.clone(),
        opts,
        click_count_model.clone(),
    );
    let _effects2 = render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(51),
        api.clone(),
        opts,
        click_count_model.clone(),
    );
    assert!(
        effects_request_raf(&effects1, window),
        "expected embla_settling carousel to request animation frames"
    );

    // While moving, a mouse click should stop motion but not activate slide content.
    let snap = ui.semantics_snapshot_arc().expect("semantics snapshot");
    let slide_button_center = pick_slide_button_center(&snap);
    click_center(&mut ui, &mut app, &mut services, slide_button_center);

    let count = app
        .models()
        .get_copied(&click_count_model)
        .expect("click count");
    assert_eq!(
        count, 0,
        "expected dragFree click-to-stop to prevent activating slide content"
    );

    // After stopping, a normal click should work again.
    let _ = render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(60),
        api.clone(),
        opts,
        click_count_model.clone(),
    );
    let snap = ui.semantics_snapshot_arc().expect("semantics snapshot");
    let slide_button_center = pick_slide_button_center(&snap);
    click_center(&mut ui, &mut app, &mut services, slide_button_center);
    let count = app
        .models()
        .get_copied(&click_count_model)
        .expect("click count");
    assert_eq!(count, 1, "expected subsequent click to activate normally");
}
