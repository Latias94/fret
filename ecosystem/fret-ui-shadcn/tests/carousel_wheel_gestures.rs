use fret_app::App;
use fret_core::{
    AppWindowId, Event, FrameId, Modifiers, Point, PointerEvent, PointerId, PointerType, Px, Rect,
    Size as CoreSize,
};
use fret_runtime::Model;
use fret_ui::tree::UiTree;
use fret_ui_kit::{LayoutRefinement, MetricRef, OverlayController, Space};
use fret_ui_shadcn::facade as shadcn;

#[path = "support/style_aware_services.rs"]
mod style_aware_services;
use style_aware_services::StyleAwareServices;

fn window_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(320.0), Px(200.0)),
    )
}

fn render_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    api: Model<shadcn::CarouselApiSnapshot>,
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
        "carousel-wheel",
        |cx| {
            let slides = (0..5).map(|_| cx.container(Default::default(), |_cx| vec![]));
            let carousel = shadcn::Carousel::new(slides)
                .api_snapshot_model(api)
                .wheel_gestures(shadcn::CarouselWheelGesturesConfig::new())
                .track_start_neg_margin(Space::N0)
                .item_padding_start(Space::N0)
                .item_basis_main_px(Px(200.0))
                .refine_layout(
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(200.0)))
                        .h_px(MetricRef::Px(Px(120.0))),
                )
                .refine_viewport_layout(LayoutRefinement::default().h_px(MetricRef::Px(Px(120.0))))
                .into_element(cx);
            vec![carousel]
        },
    );
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    ui.layout_all(app, services, bounds, 1.0);
}

fn dispatch_wheel(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    position: Point,
    delta: Point,
    modifiers: Modifiers,
) {
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(PointerEvent::Wheel {
            pointer_id: PointerId::default(),
            position,
            delta,
            modifiers,
            pointer_type: PointerType::Mouse,
        }),
    );
}

#[test]
fn carousel_wheel_gestures_can_step_next_and_prev() {
    let window = AppWindowId::default();
    let bounds = window_bounds();

    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let api = app
        .models_mut()
        .insert(shadcn::CarouselApiSnapshot::default());

    for _ in 0..3 {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            api.clone(),
        );
    }

    let snap = app.models_mut().read(&api, |v| *v).ok().unwrap_or_default();
    assert_eq!(snap.selected_index, 0);

    dispatch_wheel(
        &mut ui,
        &mut app,
        &mut services,
        Point::new(Px(10.0), Px(10.0)),
        Point::new(Px(-80.0), Px(0.0)),
        Modifiers::default(),
    );
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        api.clone(),
    );

    let snap = app.models_mut().read(&api, |v| *v).ok().unwrap_or_default();
    assert_eq!(snap.selected_index, 1);

    dispatch_wheel(
        &mut ui,
        &mut app,
        &mut services,
        Point::new(Px(10.0), Px(10.0)),
        Point::new(Px(80.0), Px(0.0)),
        Modifiers::default(),
    );
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        api.clone(),
    );

    let snap = app.models_mut().read(&api, |v| *v).ok().unwrap_or_default();
    assert_eq!(snap.selected_index, 0);
}
