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
) {
    let next_frame = FrameId(app.frame_id().0.saturating_add(1));
    app.set_frame_id(next_frame);

    OverlayController::begin_frame(app, window);
    let root =
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "carousel", |cx| {
            let slides = (0..2).map(|_| cx.container(Default::default(), |_cx| vec![]));
            let carousel = fret_ui_shadcn::Carousel::new(slides)
                .opts(opts)
                // Narrow basis reproduces the UI gallery "cannotLoop" configuration.
                .item_basis_main_px(Px(240.0))
                .api_snapshot_model(api)
                .track_start_neg_margin(Space::N4)
                .item_padding_start(Space::N4)
                .refine_layout(
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(320.0)))
                        .h_px(MetricRef::Px(Px(120.0))),
                )
                .refine_viewport_layout(LayoutRefinement::default().h_px(MetricRef::Px(Px(120.0))))
                .into_element(cx);
            vec![carousel]
        });
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    ui.layout_all(app, services, bounds, 1.0);
}

#[test]
fn carousel_loop_downgrades_when_cannot_loop_even_without_embla_engine() {
    let window = AppWindowId::default();

    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let api = app
        .models_mut()
        .insert(fret_ui_shadcn::CarouselApiSnapshot::default());

    let mut opts = fret_ui_shadcn::CarouselOptions::default();
    opts.loop_enabled = true;
    opts.embla_engine = false;

    for _ in 0..6 {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            window_bounds(Px(360.0)),
            api.clone(),
            opts,
        );
    }

    let snapshot = app.models().get_copied(&api).expect("api snapshot");
    assert!(
        snapshot.snap_count > 1,
        "expected measurable snaps; snapshot={snapshot:?}"
    );
    assert!(
        !snapshot.can_scroll_prev && snapshot.can_scroll_next,
        "expected cannotLoop downgrade to behave like loop=false at index=0; snapshot={snapshot:?}"
    );
}

#[test]
fn carousel_loop_downgrade_keeps_end_controls_disabled_without_embla_engine() {
    let window = AppWindowId::default();

    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let api = app
        .models_mut()
        .insert(fret_ui_shadcn::CarouselApiSnapshot::default());

    let mut opts = fret_ui_shadcn::CarouselOptions::default();
    opts.loop_enabled = true;
    opts.embla_engine = false;
    opts.start_snap = 1;

    for _ in 0..6 {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            window_bounds(Px(360.0)),
            api.clone(),
            opts,
        );
    }

    let snapshot = app.models().get_copied(&api).expect("api snapshot");
    assert!(
        snapshot.snap_count > 1,
        "expected measurable snaps; snapshot={snapshot:?}"
    );
    assert!(
        snapshot.can_scroll_prev && !snapshot.can_scroll_next,
        "expected cannotLoop downgrade to behave like loop=false at last index; snapshot={snapshot:?}"
    );
}
