use fret_app::App;
use fret_core::{AppWindowId, FrameId, Point, Px, Rect, Size as CoreSize};
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
    api: Model<Option<shadcn::CarouselApi>>,
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
        "carousel-api-handle",
        move |cx| {
            let slides = (0..5).map(|_| cx.container(Default::default(), |_cx| vec![]));
            let carousel = shadcn::Carousel::new(slides)
                .opts(opts)
                .api_handle_model(api)
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

fn render_frame_parts(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    api: Model<Option<shadcn::CarouselApi>>,
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
        "carousel-api-handle-parts",
        move |cx| {
            let items = (0..5)
                .map(|_| shadcn::CarouselItem::new(cx.container(Default::default(), |_cx| vec![])))
                .collect::<Vec<_>>();
            let carousel = shadcn::Carousel::default()
                .opts(opts)
                .api_handle_model(api)
                .track_start_neg_margin(Space::N0)
                .item_padding_start(Space::N0)
                .item_basis_main_px(item_basis_main_px)
                .refine_layout(
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(carousel_width))
                        .h_px(MetricRef::Px(Px(120.0))),
                )
                .refine_viewport_layout(LayoutRefinement::default().h_px(MetricRef::Px(Px(120.0))))
                .into_element_parts(
                    cx,
                    |_cx| shadcn::CarouselContent::new(items),
                    shadcn::CarouselPrevious::new(),
                    shadcn::CarouselNext::new(),
                );
            vec![carousel]
        },
    );
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    ui.layout_all(app, services, bounds, 1.0);
}

#[test]
fn carousel_api_handle_is_published_and_can_scroll_next() {
    let window = AppWindowId::default();
    let bounds = window_bounds();

    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let api = app.models_mut().insert(None::<shadcn::CarouselApi>);
    let opts = shadcn::CarouselOptions::default();

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

    let api_handle = app
        .models()
        .get_cloned(&api)
        .flatten()
        .expect("expected CarouselApi handle to be published");
    let before = api_handle.snapshot(&mut app);
    assert!(
        before.snap_count > 0,
        "expected measurable snaps; snapshot={before:?}"
    );

    let mut cursor = shadcn::CarouselEventCursor {
        select_generation: before.select_generation,
        reinit_generation: before.reinit_generation,
    };

    api_handle.scroll_next(&mut app);

    // One frame to process the queued command, plus a second frame for the observable
    // generation counters to converge if throttling applies.
    for _ in 0..2 {
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

    let after = api_handle.snapshot(&mut app);
    assert_eq!(
        after.selected_index, 1,
        "expected scroll_next to advance; snapshot={after:?}"
    );

    let events = api_handle.events_since(&mut app, &mut cursor);
    assert!(
        events
            .iter()
            .any(|ev| matches!(ev, shadcn::CarouselEvent::Select { selected_index: 1 })),
        "expected select event; events={events:?} snapshot={after:?}"
    );
}

#[test]
fn carousel_api_handle_is_published_when_using_parts_adapter() {
    let window = AppWindowId::default();
    let bounds = window_bounds();

    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let api = app.models_mut().insert(None::<shadcn::CarouselApi>);
    let opts = shadcn::CarouselOptions::default();

    for _ in 0..3 {
        render_frame_parts(
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

    let api_handle = app
        .models()
        .get_cloned(&api)
        .flatten()
        .expect("expected CarouselApi handle to be published");
    let snapshot = api_handle.snapshot(&mut app);
    assert!(
        snapshot.snap_count > 0,
        "expected measurable snaps; snapshot={snapshot:?}"
    );
}
