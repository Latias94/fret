use fret_app::App;
use fret_core::{
    AppWindowId, Event, FrameId, Modifiers, MouseButton, Point, PointerEvent, PointerId,
    PointerType, Px, Rect, Size as CoreSize,
};
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
    api: fret_runtime::Model<shadcn::CarouselApiSnapshot>,
    opts: shadcn::CarouselOptions,
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
        "carousel-start-snap-draggable",
        move |cx| {
            let slides = (0..5).map(|_| cx.container(Default::default(), |_cx| vec![]));
            let carousel = shadcn::Carousel::new(slides)
                .opts(opts)
                .api_snapshot_model(api)
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

#[test]
fn carousel_start_snap_initializes_selected_index_once_snaps_are_measurable() {
    let window = AppWindowId::default();
    let bounds = window_bounds();

    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let api = app
        .models_mut()
        .insert(shadcn::CarouselApiSnapshot::default());
    let opts = shadcn::CarouselOptions {
        start_snap: 2,
        ..Default::default()
    };

    // Two+ frames are required because the carousel derives snap points from the previous layout
    // pass' measured geometry.
    for _ in 0..3 {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            api.clone(),
            opts,
        );
    }

    let snapshot = app.models().get_copied(&api).expect("api snapshot");
    assert_eq!(
        snapshot.selected_index, 2,
        "expected startSnap to select index 2 once snaps are measurable; snapshot={snapshot:?}"
    );
}

#[test]
fn carousel_draggable_false_disables_swipe_drag() {
    let window = AppWindowId::default();
    let bounds = window_bounds();

    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let api = app
        .models_mut()
        .insert(shadcn::CarouselApiSnapshot::default());
    let opts = shadcn::CarouselOptions::new().draggable(false);

    for _ in 0..3 {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            api.clone(),
            opts,
        );
    }

    let pointer_id = PointerId(0);
    let start = Point::new(Px(100.0), Px(60.0));
    let moved = Point::new(Px(20.0), Px(60.0));

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
            buttons: fret_core::MouseButtons {
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

    // Render once more so the API snapshot reflects any state changes.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        api.clone(),
        opts,
    );

    let snapshot = app.models().get_copied(&api).expect("api snapshot");
    assert_eq!(
        snapshot.selected_index, 0,
        "expected draggable=false to prevent swipe selection changes; snapshot={snapshot:?}"
    );
}
