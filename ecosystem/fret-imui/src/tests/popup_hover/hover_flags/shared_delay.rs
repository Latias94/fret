use super::*;

#[test]
fn no_shared_delay_disables_window_scoped_hover_delay_sharing() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(220.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let hovered_b_shared = Rc::new(Cell::new(false));
    let hovered_b_no_shared = Rc::new(Cell::new(false));
    let b_stationary_met = Rc::new(Cell::new(false));
    let b_delay_short_met = Rc::new(Cell::new(false));
    let b_delay_short_shared_met = Rc::new(Cell::new(false));
    let id_a: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> = Rc::new(Cell::new(None));
    let id_b: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> = Rc::new(Cell::new(None));

    ui.request_semantics_snapshot();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-shared-hover-delay",
        |cx| {
            render_imui_shared_hover_delay_scene(
                cx,
                id_a.clone(),
                hovered_b_shared.clone(),
                hovered_b_no_shared.clone(),
                b_stationary_met.clone(),
                b_delay_short_met.clone(),
                b_delay_short_shared_met.clone(),
                id_b.clone(),
            )
        },
    );

    let id_a_value = id_a.get().expect("expected A to have a GlobalElementId");
    let id_b_value = id_b.get().expect("expected B to have a GlobalElementId");
    assert_ne!(
        id_a_value, id_b_value,
        "expected A and B to have distinct ids"
    );

    let a_bounds = bounds_for_test_id(&ui, "imui-shared-delay-a");
    let a_center = Point::new(
        Px(a_bounds.origin.x.0 + a_bounds.size.width.0 * 0.5),
        Px(a_bounds.origin.y.0 + a_bounds.size.height.0 * 0.5),
    );
    let b_bounds = bounds_for_test_id(&ui, "imui-shared-delay-b");
    let b_center = Point::new(
        Px(b_bounds.origin.x.0 + b_bounds.size.width.0 * 0.5),
        Px(b_bounds.origin.y.0 + b_bounds.size.height.0 * 0.5),
    );

    // Hover A long enough to meet the shared short-delay timer.
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        a_center,
        MouseButtons::default(),
    );
    app.advance_frame();
    ui.request_semantics_snapshot();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-shared-hover-delay",
        |cx| {
            render_imui_shared_hover_delay_scene(
                cx,
                id_a.clone(),
                hovered_b_shared.clone(),
                hovered_b_no_shared.clone(),
                b_stationary_met.clone(),
                b_delay_short_met.clone(),
                b_delay_short_shared_met.clone(),
                id_b.clone(),
            )
        },
    );

    let kind_stationary = fnv1a64(b"fret-ui-kit.imui.hover.timer.stationary.v1");
    let kind_delay_short = fnv1a64(b"fret-ui-kit.imui.hover.timer.delay_short.v1");
    let kind_delay_normal = fnv1a64(b"fret-ui-kit.imui.hover.timer.delay_normal.v1");

    let local_tokens = [
        hover_timer_token_for(kind_stationary, id_a_value),
        hover_timer_token_for(kind_delay_short, id_a_value),
        hover_timer_token_for(kind_delay_normal, id_a_value),
        hover_timer_token_for(kind_stationary, id_b_value),
        hover_timer_token_for(kind_delay_short, id_b_value),
        hover_timer_token_for(kind_delay_normal, id_b_value),
    ];
    let local_tokens: std::collections::HashSet<TimerToken> = local_tokens.into_iter().collect();

    let pending = pending_nonrepeating_timer_tokens(&app);
    let shared_tokens: Vec<TimerToken> = pending
        .into_iter()
        .filter(|token| !local_tokens.contains(token))
        .collect();
    assert!(
        shared_tokens.len() >= 2,
        "expected shared hover delay timers to be scheduled; shared_tokens={shared_tokens:?}"
    );

    let dispatched_shared = dispatch_timer_tokens(&mut ui, &mut app, &mut services, &shared_tokens);
    assert_eq!(
        dispatched_shared,
        shared_tokens.len(),
        "expected to dispatch all shared delay timers"
    );

    app.advance_frame();
    ui.request_semantics_snapshot();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-shared-hover-delay",
        |cx| {
            render_imui_shared_hover_delay_scene(
                cx,
                id_a.clone(),
                hovered_b_shared.clone(),
                hovered_b_no_shared.clone(),
                b_stationary_met.clone(),
                b_delay_short_met.clone(),
                b_delay_short_shared_met.clone(),
                id_b.clone(),
            )
        },
    );

    // Move to B: with shared delay enabled, B should only need the stationary timer to fire.
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        b_center,
        MouseButtons::default(),
    );
    app.advance_frame();
    ui.request_semantics_snapshot();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-shared-hover-delay",
        |cx| {
            render_imui_shared_hover_delay_scene(
                cx,
                id_a.clone(),
                hovered_b_shared.clone(),
                hovered_b_no_shared.clone(),
                b_stationary_met.clone(),
                b_delay_short_met.clone(),
                b_delay_short_shared_met.clone(),
                id_b.clone(),
            )
        },
    );

    if std::env::var_os("FRET_DEBUG_IMUI_SHARED_HOVER_DELAY").is_some() {
        eprintln!(
            "shared_hover_delay: before_stationary hovered_b_shared={} hovered_b_no_shared={} stationary_met={} delay_short_met={} delay_short_shared_met={}",
            hovered_b_shared.get(),
            hovered_b_no_shared.get(),
            b_stationary_met.get(),
            b_delay_short_met.get(),
            b_delay_short_shared_met.get(),
        );
    }

    assert!(
        !hovered_b_shared.get() && !hovered_b_no_shared.get(),
        "expected B hovered query to be false before the stationary timer fires"
    );

    let id_b_value = id_b.get().expect("expected B to have a GlobalElementId");
    let stationary_token_b = hover_timer_token_for(kind_stationary, id_b_value);
    let delay_short_token_b = hover_timer_token_for(kind_delay_short, id_b_value);

    let pending = pending_nonrepeating_timer_tokens(&app);
    assert!(
        pending.contains(&stationary_token_b),
        "expected B stationary timer to be scheduled"
    );
    assert!(
        pending.contains(&delay_short_token_b),
        "expected B local delay-short timer to be scheduled"
    );

    let dispatched = dispatch_timer_tokens(&mut ui, &mut app, &mut services, &[stationary_token_b]);
    assert_eq!(
        dispatched, 1,
        "expected to dispatch exactly the stationary timer for B"
    );
    assert!(
        pending_nonrepeating_timer_tokens(&app).contains(&delay_short_token_b),
        "expected B local delay-short timer to remain pending"
    );

    app.advance_frame();
    ui.request_semantics_snapshot();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-shared-hover-delay",
        |cx| {
            render_imui_shared_hover_delay_scene(
                cx,
                id_a.clone(),
                hovered_b_shared.clone(),
                hovered_b_no_shared.clone(),
                b_stationary_met.clone(),
                b_delay_short_met.clone(),
                b_delay_short_shared_met.clone(),
                id_b.clone(),
            )
        },
    );

    assert!(
        hovered_b_shared.get(),
        "expected shared delay to allow DELAY_SHORT hover after stationary is met"
    );
    assert!(
        !hovered_b_no_shared.get(),
        "expected NO_SHARED_DELAY to require the local delay-short timer"
    );
}
