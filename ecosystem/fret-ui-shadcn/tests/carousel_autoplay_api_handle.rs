use fret_app::App;
use fret_core::{AppWindowId, Event, FrameId, Point, Px, Rect, Size as CoreSize, TimerToken};
use fret_runtime::{Effect, Model};
use fret_ui::tree::UiTree;
use fret_ui_kit::{LayoutRefinement, MetricRef, OverlayController, Space};
use std::sync::Arc;
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

fn take_last_timer_after(app: &mut App, window: AppWindowId) -> Option<(TimerToken, Duration)> {
    app.flush_effects()
        .into_iter()
        .filter_map(|e| match e {
            Effect::SetTimer {
                window: Some(w),
                token,
                after,
                ..
            } if w == window => Some((token, after)),
            _ => None,
        })
        .last()
}

fn flush_contains_set_timer(app: &mut App, window: AppWindowId) -> bool {
    app.flush_effects().into_iter().any(|e| match e {
        Effect::SetTimer {
            window: Some(w), ..
        } if w == window => true,
        _ => false,
    })
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

#[test]
fn carousel_autoplay_stop_on_last_snap_stops_after_reaching_last_snap() {
    let window = AppWindowId::default();
    let bounds = window_bounds();

    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let autoplay = app
        .models_mut()
        .insert(None::<fret_ui_shadcn::CarouselAutoplayApi>);
    let opts = fret_ui_shadcn::CarouselOptions::default().loop_enabled(false);

    fn render_stop_on_last_snap_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        autoplay: Model<Option<fret_ui_shadcn::CarouselAutoplayApi>>,
        opts: fret_ui_shadcn::CarouselOptions,
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
            "carousel-autoplay-stop-on-last-snap",
            move |cx| {
                let slides = (0..5).map(|_| cx.container(Default::default(), |_cx| vec![]));
                let carousel = fret_ui_shadcn::Carousel::new(slides)
                    .opts(opts)
                    .plugins([fret_ui_shadcn::CarouselPlugin::Autoplay(
                        fret_ui_shadcn::CarouselAutoplayConfig::new(Duration::from_millis(10))
                            .pause_on_hover(false)
                            .reset_on_hover_leave(false)
                            .stop_on_last_snap(true),
                    )])
                    .autoplay_api_handle_model(autoplay)
                    .track_start_neg_margin(Space::N0)
                    .item_padding_start(Space::N0)
                    .item_basis_main_px(Px(200.0))
                    .refine_layout(
                        LayoutRefinement::default()
                            .w_px(MetricRef::Px(Px(200.0)))
                            .h_px(MetricRef::Px(Px(120.0))),
                    )
                    .refine_viewport_layout(
                        LayoutRefinement::default().h_px(MetricRef::Px(Px(120.0))),
                    )
                    .into_element(cx);
                vec![carousel]
            },
        );
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.layout_all(app, services, bounds, 1.0);
    }

    for _ in 0..3 {
        render_stop_on_last_snap_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            autoplay.clone(),
            opts,
        );
    }

    let handle = app
        .models()
        .get_cloned(&autoplay)
        .flatten()
        .expect("expected CarouselAutoplayApi handle to be published");

    let (token, _) = take_last_timer_after(&mut app, window)
        .expect("expected autoplay to arm a timer via Effect::SetTimer");

    for _ in 0..10 {
        ui.dispatch_event(&mut app, &mut services, &Event::Timer { token });

        render_stop_on_last_snap_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            autoplay.clone(),
            opts,
        );

        let snap = handle.snapshot(&mut app);
        if snap.stopped_by_last_snap {
            assert!(
                !snap.playing,
                "expected stop_on_last_snap to clear playing; snapshot={snap:?}"
            );
            assert!(
                !flush_contains_set_timer(&mut app, window),
                "expected stop_on_last_snap to stop rescheduling timers"
            );
            return;
        }

        // Drain any rescheduled timer effects so they don't accumulate across the loop and
        // trip the "no reschedule after stopping" assertion.
        let _ = app.flush_effects();
    }

    panic!(
        "expected autoplay to stop by last snap within the tick budget; snapshot={:?}",
        handle.snapshot(&mut app)
    );
}

#[test]
fn carousel_autoplay_per_snap_delays_reschedule_timer_after_with_expected_values() {
    let window = AppWindowId::default();
    let bounds = window_bounds();

    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let autoplay = app
        .models_mut()
        .insert(None::<fret_ui_shadcn::CarouselAutoplayApi>);
    let opts = fret_ui_shadcn::CarouselOptions::default().loop_enabled(false);

    for _ in 0..3 {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            autoplay.clone(),
            opts,
            Px(200.0),
            Px(200.0),
        );
    }

    let handle = app
        .models()
        .get_cloned(&autoplay)
        .flatten()
        .expect("expected CarouselAutoplayApi handle to be published");

    handle.stop(&mut app);
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        autoplay.clone(),
        opts,
        Px(200.0),
        Px(200.0),
    );
    let _ = app.flush_effects();

    let delays: Arc<[Duration]> = Arc::from([
        Duration::from_millis(11),
        Duration::from_millis(22),
        Duration::from_millis(33),
        Duration::from_millis(44),
        Duration::from_millis(55),
    ]);
    handle.set_delays_store(app.models_mut(), delays.clone());
    handle.play(&mut app);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        autoplay.clone(),
        opts,
        Px(200.0),
        Px(200.0),
    );
    let (token, after0) = take_last_timer_after(&mut app, window)
        .expect("expected autoplay to arm a timer after play()");
    assert_eq!(
        after0, delays[0],
        "expected initial timer to use per-snap delay for selected index 0"
    );

    ui.dispatch_event(&mut app, &mut services, &Event::Timer { token });
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        autoplay.clone(),
        opts,
        Px(200.0),
        Px(200.0),
    );
    let (_token, after1) = take_last_timer_after(&mut app, window)
        .expect("expected autoplay to reschedule timer after first tick");
    assert_eq!(
        after1, delays[1],
        "expected rescheduled timer to use per-snap delay for target index 1"
    );
}
