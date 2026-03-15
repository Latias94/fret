use fret_app::App;
use fret_core::{
    AppWindowId, Event, FrameId, Modifiers, MouseButton, MouseButtons, Point, PointerEvent,
    PointerId, PointerType, Px, Rect, Size as CoreSize,
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

fn bounds_center(r: Rect) -> Point {
    Point::new(
        Px(r.origin.x.0 + r.size.width.0 * 0.5),
        Px(r.origin.y.0 + r.size.height.0 * 0.5),
    )
}

fn render_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    api: Model<shadcn::CarouselApiSnapshot>,
    slides_in_view: Model<shadcn::CarouselSlidesInViewSnapshot>,
    opts: shadcn::CarouselOptions,
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
        "carousel-slides-in-view",
        move |cx| {
            let slides = (0..5).map(|_| cx.container(Default::default(), |_cx| vec![]));
            let carousel = shadcn::Carousel::new(slides)
                .opts(opts)
                .api_snapshot_model(api)
                .slides_in_view_snapshot_model(slides_in_view)
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
    ui.layout_all(app, services, bounds, 1.0);
}

#[test]
fn carousel_slides_in_view_snapshot_emits_enter_and_leave() {
    let window = AppWindowId::default();
    let bounds = window_bounds();

    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let slides_in_view = app
        .models_mut()
        .insert(shadcn::CarouselSlidesInViewSnapshot::default());
    let api = app
        .models_mut()
        .insert(shadcn::CarouselApiSnapshot::default());

    let opts = shadcn::CarouselOptions::default()
        .embla_engine(true)
        .in_view_threshold(0.0)
        .in_view_margin_px(Px(0.0));

    for _ in 0..4 {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            api.clone(),
            slides_in_view.clone(),
            opts,
            Px(200.0),
            Px(200.0),
        );
    }

    let before = app
        .models_mut()
        .get_cloned(&slides_in_view)
        .expect("slides in view snapshot");
    assert!(
        before.slides_in_view.iter().any(|&idx| idx == 0),
        "expected initial slide to be in view; snapshot={before:?}"
    );

    let pointer_id = PointerId(0);
    let start = bounds_center(bounds);
    let moved = Point::new(Px(start.x.0 - 80.0), start.y);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id,
            position: start,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id,
            position: moved,
            buttons: MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id,
            position: moved,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );

    let mut seen_enter_1 = false;
    let mut seen_left_0 = false;
    let mut last_generation = before.generation;
    let mut final_snapshot = before.clone();

    for _ in 0..40 {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            api.clone(),
            slides_in_view.clone(),
            opts,
            Px(200.0),
            Px(200.0),
        );

        let cur = app
            .models_mut()
            .get_cloned(&slides_in_view)
            .expect("slides in view snapshot");
        final_snapshot = cur.clone();
        if cur.generation != last_generation {
            if cur.slides_enter_view.iter().any(|&idx| idx == 1) {
                seen_enter_1 = true;
            }
            if cur.slides_left_view.iter().any(|&idx| idx == 0) {
                seen_left_0 = true;
            }
            last_generation = cur.generation;
        }

        if seen_enter_1
            && seen_left_0
            && cur.slides_in_view.as_ref() == [1]
            && cur.slides_in_view.iter().all(|&idx| idx <= 4)
        {
            break;
        }
    }

    assert!(
        seen_enter_1,
        "expected slide 1 to enter view during next; final_snapshot={final_snapshot:?}"
    );
    assert!(
        seen_left_0,
        "expected slide 0 to leave view during next; final_snapshot={final_snapshot:?}"
    );
    assert_eq!(
        final_snapshot.slides_in_view.as_ref(),
        [1],
        "expected to settle with slide 1 in view; final_snapshot={final_snapshot:?}"
    );
}
