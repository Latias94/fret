use fret_app::App;
use fret_core::{AppWindowId, FrameId, Point, Px, Rect, Size as CoreSize};
use fret_runtime::Model;
use fret_ui::tree::UiTree;
use fret_ui_kit::{LayoutRefinement, MetricRef, OverlayController, Space};
use std::time::Duration;

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
    api: Model<Option<fret_ui_shadcn::CarouselAutoplayApi>>,
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
        "carousel-autoplay-api-handle",
        move |cx| {
            let slides = (0..5).map(|_| cx.container(Default::default(), |_cx| vec![]));
            let carousel = fret_ui_shadcn::Carousel::new(slides)
                .opts(opts)
                .plugins([fret_ui_shadcn::CarouselPlugin::Autoplay(
                    fret_ui_shadcn::CarouselAutoplayConfig::new(Duration::from_millis(2000))
                        .pause_on_hover(false)
                        .reset_on_hover_leave(false),
                )])
                .autoplay_api_handle_model(api)
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
fn carousel_autoplay_api_handle_publishes_and_accepts_stop_reset() {
    let window = AppWindowId::default();
    let bounds = window_bounds();

    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let api = app
        .models_mut()
        .insert(None::<fret_ui_shadcn::CarouselAutoplayApi>);
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

    let handle = app
        .models()
        .get_cloned(&api)
        .flatten()
        .expect("expected CarouselAutoplayApi handle to be published");

    let snap = handle.snapshot(&mut app);
    assert!(
        snap.playing,
        "expected autoplay to be playing after initial mount; snapshot={snap:?}"
    );
    assert!(
        snap.time_until_next.is_some(),
        "expected time_until_next to be present while playing; snapshot={snap:?}"
    );

    handle.stop(&mut app);

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

    let snap = handle.snapshot(&mut app);
    assert!(
        snap.paused_external,
        "expected stop() to pause externally; snapshot={snap:?}"
    );
    assert!(
        !snap.playing,
        "expected stop() to clear playing flag; snapshot={snap:?}"
    );

    handle.reset(&mut app);

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

    let snap = handle.snapshot(&mut app);
    assert!(
        !snap.paused_external,
        "expected reset() to clear external pause; snapshot={snap:?}"
    );
}

#[test]
fn carousel_autoplay_api_handle_accepts_pause_and_play() {
    let window = AppWindowId::default();
    let bounds = window_bounds();

    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let api = app
        .models_mut()
        .insert(None::<fret_ui_shadcn::CarouselAutoplayApi>);
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

    let handle = app
        .models()
        .get_cloned(&api)
        .flatten()
        .expect("expected CarouselAutoplayApi handle to be published");

    handle.pause(&mut app);
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

    let snap = handle.snapshot(&mut app);
    assert!(
        snap.paused_external && !snap.playing,
        "expected pause() to stop the timer and mark paused_external; snapshot={snap:?}"
    );

    handle.play(&mut app);
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

    let snap = handle.snapshot(&mut app);
    assert!(
        !snap.paused_external && snap.playing,
        "expected play() to resume autoplay; snapshot={snap:?}"
    );
    assert!(
        snap.time_until_next.is_some(),
        "expected time_until_next to be present after resuming; snapshot={snap:?}"
    );
}
