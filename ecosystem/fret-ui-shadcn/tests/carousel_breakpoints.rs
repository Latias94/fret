use fret_app::App;
use fret_core::{AppWindowId, FrameId, Point, Px, Rect, Size as CoreSize};
use fret_runtime::Model;
use fret_ui::tree::UiTree;
use fret_ui_kit::{LayoutRefinement, MetricRef, OverlayController, Space};

#[path = "support/style_aware_services.rs"]
mod style_aware_services;
use style_aware_services::StyleAwareServices;

fn window_bounds(width: Px) -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(width, Px(200.0)),
    )
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
        "carousel-breakpoints",
        move |cx| {
            let slides = (0..5).map(|_| cx.container(Default::default(), |_cx| vec![]));
            let carousel = fret_ui_shadcn::Carousel::new(slides)
                .opts(opts)
                .breakpoint(
                    Px(300.0),
                    fret_ui_shadcn::CarouselOptionsPatch {
                        loop_enabled: Some(true),
                        ..Default::default()
                    },
                )
                .api_snapshot_model(api)
                .track_start_neg_margin(Space::N0)
                .item_padding_start(Space::N0)
                .item_basis_main_px(Px(200.0))
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
fn carousel_breakpoints_toggle_loop_enabled_by_viewport_width() {
    let window = AppWindowId::default();

    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let api = app
        .models_mut()
        .insert(fret_ui_shadcn::CarouselApiSnapshot::default());
    let opts = fret_ui_shadcn::CarouselOptions::default();

    // Below breakpoint: loop remains disabled, so can_scroll_prev should be false at index=0.
    for _ in 0..3 {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            window_bounds(Px(360.0)),
            api.clone(),
            opts,
            Px(280.0),
        );
    }
    let below = app.models().get_copied(&api).expect("api snapshot");
    assert!(
        below.snap_count > 0,
        "expected measurable snaps; snapshot={below:?}"
    );
    assert!(
        !below.can_scroll_prev,
        "expected loop=false below breakpoint; snapshot={below:?}"
    );

    // Above breakpoint: loop is enabled, so can_scroll_prev should become true after the viewport
    // width observation settles (the recipe uses the previous layout pass).
    let mut above = below;
    for _ in 0..6 {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            window_bounds(Px(360.0)),
            api.clone(),
            opts,
            Px(320.0),
        );
        above = app.models().get_copied(&api).expect("api snapshot");
        if above.can_scroll_prev {
            break;
        }
    }
    assert!(
        above.can_scroll_prev,
        "expected loop=true above breakpoint; below={below:?} above={above:?}"
    );
}
