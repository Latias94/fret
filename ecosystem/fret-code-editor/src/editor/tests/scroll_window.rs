use super::*;

#[test]
fn editor_viewport_wheel_scroll_updates_inner_window_without_bounds_drift() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    ui.set_debug_enabled(true);
    let mut services = FakeServices::default();
    let text = (0..240)
        .map(|idx| format!("fn line_{idx:03}() {{ println!(\"scroll {idx:03}\"); }}"))
        .collect::<Vec<_>>()
        .join("\n");
    let handle = CodeEditorHandle::new(text);

    app.set_tick_id(TickId(1));
    app.set_frame_id(FrameId(1));
    render_editor_scroll_audit_frame(&mut ui, &mut app, &mut services, window, handle.clone());
    app.set_tick_id(TickId(2));
    app.set_frame_id(FrameId(2));
    render_editor_scroll_audit_frame(&mut ui, &mut app, &mut services, window, handle.clone());

    let snap = ui
        .semantics_snapshot()
        .expect("semantics snapshot before wheel");
    let before_bounds = bounds_by_test_id(&ui, &snap, "code-editor-scroll-audit-viewport");
    let before = windowed_rows_telemetry(&app, window);
    assert!(
        before.visible_count > 0,
        "expected visible rows before wheel, telemetry={before:?}"
    );
    assert!(
        before.offset_y.0.abs() <= 0.01,
        "expected initial inner viewport offset near zero, telemetry={before:?}"
    );
    assert!(
        (before_bounds.size.height.0 - before.viewport_height.0).abs() <= 0.01,
        "expected viewport test_id bounds to match visible viewport height: bounds={before_bounds:?} telemetry={before:?}"
    );
    assert!(
        before.content_height.0 > before_bounds.size.height.0 + 0.01,
        "expected content to be taller than the visible viewport in this regression fixture: bounds={before_bounds:?} telemetry={before:?}"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Wheel {
            position: center_of(before_bounds),
            delta: Point::new(Px(0.0), Px(-120.0)),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    app.set_tick_id(TickId(3));
    app.set_frame_id(FrameId(3));
    render_editor_scroll_audit_frame(&mut ui, &mut app, &mut services, window, handle);

    let snap = ui
        .semantics_snapshot()
        .expect("semantics snapshot after wheel");
    let after_bounds = bounds_by_test_id(&ui, &snap, "code-editor-scroll-audit-viewport");
    let after = windowed_rows_telemetry(&app, window);
    assert!(
        after.offset_y.0 > before.offset_y.0 + 0.01,
        "expected wheel to advance editor inner viewport offset: before={before:?} after={after:?}"
    );
    assert!(
        after.visible_start.unwrap_or(0) >= before.visible_start.unwrap_or(0),
        "expected visible row window to stay monotonic after wheel: before={before:?} after={after:?}"
    );
    assert!(
        (after_bounds.origin.x.0 - before_bounds.origin.x.0).abs() <= 0.01
            && (after_bounds.origin.y.0 - before_bounds.origin.y.0).abs() <= 0.01
            && (after_bounds.size.width.0 - before_bounds.size.width.0).abs() <= 0.01
            && (after_bounds.size.height.0 - before_bounds.size.height.0).abs() <= 0.01,
        "expected editor viewport bounds to stay stable while inner scroll moves: before={before_bounds:?} after={after_bounds:?}"
    );
}
