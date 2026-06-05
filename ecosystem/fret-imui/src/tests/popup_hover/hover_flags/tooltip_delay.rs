use super::*;

#[test]
fn hovered_for_tooltip_requires_stationary_and_delay_short_even_when_disabled() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let hovered_for_tooltip = Rc::new(Cell::new(false));
    let hovered_raw = Rc::new(Cell::new(false));
    let stationary_met = Rc::new(Cell::new(false));
    let delay_short_met = Rc::new(Cell::new(false));
    let delay_normal_met = Rc::new(Cell::new(false));

    ui.request_semantics_snapshot();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-hover-for-tooltip-disabled",
        |cx| {
            render_imui_disabled_scope_tooltip_hover_scene(
                cx,
                hovered_for_tooltip.clone(),
                hovered_raw.clone(),
                stationary_met.clone(),
                delay_short_met.clone(),
                delay_normal_met.clone(),
            )
        },
    );

    let target_bounds = bounds_for_test_id(&ui, "imui-tooltip-target");
    let target_center = Point::new(
        Px(target_bounds.origin.x.0 + target_bounds.size.width.0 * 0.5),
        Px(target_bounds.origin.y.0 + target_bounds.size.height.0 * 0.5),
    );

    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        target_center,
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
        "imui-hover-for-tooltip-disabled",
        |cx| {
            render_imui_disabled_scope_tooltip_hover_scene(
                cx,
                hovered_for_tooltip.clone(),
                hovered_raw.clone(),
                stationary_met.clone(),
                delay_short_met.clone(),
                delay_normal_met.clone(),
            )
        },
    );

    assert!(
        hovered_raw.get(),
        "expected raw hovered to be true when disabled"
    );
    assert!(
        !hovered_for_tooltip.get(),
        "expected ForTooltip to be false before delay timers fire"
    );
    assert!(
        !stationary_met.get() && !delay_short_met.get(),
        "expected hover delay state to be unset before timers fire"
    );

    let dispatched = dispatch_all_timers(&mut ui, &mut app, &mut services);
    assert!(
        dispatched >= 3,
        "expected hover timers to be scheduled; dispatched={dispatched}"
    );

    app.advance_frame();
    ui.request_semantics_snapshot();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-hover-for-tooltip-disabled",
        |cx| {
            render_imui_disabled_scope_tooltip_hover_scene(
                cx,
                hovered_for_tooltip.clone(),
                hovered_raw.clone(),
                stationary_met.clone(),
                delay_short_met.clone(),
                delay_normal_met.clone(),
            )
        },
    );

    assert!(
        stationary_met.get() && delay_short_met.get(),
        "expected stationary and short delay to be met after timers dispatch"
    );
    assert!(
        hovered_for_tooltip.get(),
        "expected ForTooltip hovered query to be true after timers dispatch"
    );
    assert!(
        delay_normal_met.get(),
        "expected normal delay to be met after timers dispatch (best-effort)"
    );
}
