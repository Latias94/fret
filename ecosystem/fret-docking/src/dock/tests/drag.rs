use super::*;

#[test]
fn dock_drag_suppresses_viewport_hover_and_wheel_forwarding() {
    let mut harness = DockViewportHarness::new();
    harness.layout();

    harness
        .app
        .set_global(fret_runtime::WindowInteractionDiagnosticsStore::default());

    harness.app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(0),
        DRAG_KIND_DOCK_PANEL,
        harness.window,
        Point::new(Px(12.0), Px(12.0)),
        DockPanelDragPayload {
            panel: PanelKey::new("core.viewport"),
            grab_offset: Point::new(Px(0.0), Px(0.0)),
            start_tick: fret_runtime::TickId(0),
            tear_off_requested: false,
            tear_off_oob_start_frame: None,
            dock_previews_enabled: true,
        },
    );
    if let Some(drag) = harness.app.drag_mut(fret_core::PointerId(0)) {
        drag.dragging = true;
    }

    // Ensure the dock interaction state is publishable to diagnostics (so suppression is
    // debuggable without relying on logs).
    harness.layout();
    let _ = harness.app.take_effects();

    let position = harness.viewport_point();

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Wheel {
            position,
            delta: Point::new(Px(0.0), Px(12.0)),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let effects = harness.app.take_effects();
    assert!(
        !effects
            .iter()
            .any(|e| matches!(e, Effect::ViewportInput(_))),
        "dock drag must suppress viewport hover/wheel forwarding (ADR 0072), got: {effects:?}",
    );

    let dock = harness
        .app
        .global::<fret_runtime::WindowInteractionDiagnosticsStore>()
        .and_then(|store| store.docking_for_window(harness.window, harness.app.frame_id()))
        .expect("expected docking interaction diagnostics to be published for the window/frame");
    assert!(
        dock.dock_drag.is_some(),
        "expected dock drag to be recorded as the suppression reason, got: {dock:?}"
    );
}
#[test]
fn dock_drag_records_drop_target_diagnostics_for_inner_left_hint_rect() {
    let mut harness = DockViewportHarness::new();
    harness.layout();

    harness
        .app
        .set_global(fret_runtime::WindowInteractionDiagnosticsStore::default());

    harness.app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(0),
        DRAG_KIND_DOCK_PANEL,
        harness.window,
        Point::new(Px(12.0), Px(12.0)),
        DockPanelDragPayload {
            panel: PanelKey::new("core.viewport"),
            grab_offset: Point::new(Px(0.0), Px(0.0)),
            start_tick: fret_runtime::TickId(0),
            tear_off_requested: false,
            tear_off_oob_start_frame: None,
            dock_previews_enabled: true,
        },
    );
    if let Some(drag) = harness.app.drag_mut(fret_core::PointerId(0)) {
        drag.dragging = true;
    }

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let (_chrome, dock_bounds) = dock_space_regions(bounds);
    let left_rect = dock_hint_rects_with_font(dock_bounds, Px(13.0), false)
        .into_iter()
        .find_map(|(zone, rect)| (zone == DropZone::Left).then_some(rect))
        .expect("expected inner left rect");
    let position = Point::new(
        Px(left_rect.origin.x.0 + left_rect.size.width.0 * 0.5),
        Px(left_rect.origin.y.0 + left_rect.size.height.0 * 0.5),
    );

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::InternalDrag(InternalDragEvent {
            position,
            kind: InternalDragKind::Over,
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
        }),
    );

    harness.layout();

    let dock = harness
        .app
        .global::<fret_runtime::WindowInteractionDiagnosticsStore>()
        .and_then(|store| store.docking_for_window(harness.window, harness.app.frame_id()))
        .expect("expected docking interaction diagnostics to be published for the window/frame");
    let diag = dock
        .dock_drop_resolve
        .as_ref()
        .expect("expected drop target diagnostics to be published");
    assert_eq!(
        diag.source,
        fret_runtime::DockDropResolveSource::InnerHintRect
    );
    let resolved = diag.resolved.expect("expected a resolved dock target");
    assert_eq!(resolved.zone, DropZone::Left);
    assert!(!resolved.outer);
    assert!(
        diag.candidates.iter().any(|c| {
            c.kind == fret_runtime::DockDropCandidateRectKind::InnerHintRect
                && c.zone == Some(DropZone::Left)
                && c.rect == left_rect
        }),
        "expected diagnostics to include the inner left hint rect, got: {:?}",
        diag.candidates
    );
}
#[test]
fn pointer_occlusion_blocks_viewport_hover_and_down_but_allows_wheel_forwarding() {
    struct HitTestTransparent;

    impl<H: UiHost> Widget<H> for HitTestTransparent {
        fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
            false
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let mut harness = DockViewportHarness::new();
    harness.layout();

    // Install a window-level pointer occlusion layer (Radix `disableOutsidePointerEvents`-like).
    let overlay_root = harness.ui.create_node_retained(HitTestTransparent);
    let overlay_layer = harness.ui.push_overlay_root_ex(overlay_root, false, true);
    harness.ui.set_layer_pointer_occlusion(
        overlay_layer,
        fret_ui::tree::PointerOcclusion::BlockMouseExceptScroll,
    );
    harness.layout();
    let _ = harness.app.take_effects();

    let position = harness.viewport_point();

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let effects = harness.app.take_effects();
    assert!(
        !effects
            .iter()
            .any(|e| matches!(e, Effect::ViewportInput(_))),
        "expected pointer occlusion to suppress viewport hover forwarding, got: {effects:?}",
    );

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let effects = harness.app.take_effects();
    assert!(
        !effects
            .iter()
            .any(|e| matches!(e, Effect::ViewportInput(_))),
        "expected pointer occlusion to suppress viewport capture start, got: {effects:?}",
    );
    assert_eq!(
        harness.ui.captured_for(fret_core::PointerId(0)),
        None,
        "expected pointer occlusion to prevent viewport capture from requesting pointer capture"
    );

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Wheel {
            position,
            delta: Point::new(Px(0.0), Px(12.0)),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let effects = harness.app.take_effects();
    let Some(Effect::ViewportInput(input)) = effects
        .iter()
        .find(|e| matches!(e, Effect::ViewportInput(_)))
    else {
        panic!(
            "expected wheel forwarding to still emit a ViewportInput under pointer occlusion, got: {effects:?}"
        );
    };
    assert!(
        matches!(input.kind, fret_core::ViewportInputKind::Wheel { .. }),
        "expected wheel forwarding to remain active under BlockMouseExceptScroll, got: {input:?}",
    );
}
#[test]
fn foreign_pointer_capture_suppresses_viewport_capture_start() {
    struct CaptureOverlay;

    impl<H: UiHost> Widget<H> for CaptureOverlay {
        fn hit_test(&self, _bounds: Rect, position: Point) -> bool {
            position.x.0 >= 0.0
                && position.y.0 >= 0.0
                && position.x.0 <= 20.0
                && position.y.0 <= 20.0
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }

        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if matches!(event, Event::Pointer(fret_core::PointerEvent::Down { .. })) {
                cx.capture_pointer(cx.node);
                cx.stop_propagation();
            }
        }
    }

    let mut harness = DockViewportHarness::new();
    harness.layout();

    let overlay_root = harness.ui.create_node_retained(CaptureOverlay);
    let _overlay_layer = harness.ui.push_overlay_root_ex(overlay_root, false, true);
    harness.layout();

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(2.0), Px(2.0)),
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(1),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(
        harness.ui.captured_for(fret_core::PointerId(1)),
        Some(overlay_root),
        "expected overlay to capture pointer 1"
    );

    // Advance a frame so docking can observe the runtime arbitration snapshot.
    harness.layout();
    let _ = harness.app.take_effects();

    let position = harness.viewport_point();
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let effects = harness.app.take_effects();
    assert!(
        !effects
            .iter()
            .any(|e| matches!(e, Effect::ViewportInput(_))),
        "expected foreign pointer capture to suppress viewport capture start, got: {effects:?}",
    );
    assert_eq!(
        harness.ui.captured_for(fret_core::PointerId(0)),
        None,
        "expected foreign pointer capture to prevent viewport capture from requesting pointer capture"
    );
}
#[test]
fn pending_dock_drag_suppresses_viewport_hover_and_wheel_forwarding() {
    let mut harness = DockViewportHarness::new();
    harness.layout();

    let tab_pos = harness.tab_point(0);

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: tab_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let _ = harness.app.take_effects();

    assert!(
        harness.app.drag(fret_core::PointerId(0)).is_none(),
        "pending tab press should not start a cross-window drag session yet",
    );

    let position = harness.viewport_point();
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position,
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Wheel {
            position,
            delta: Point::new(Px(0.0), Px(12.0)),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let effects = harness.app.take_effects();
    assert!(
        !effects
            .iter()
            .any(|e| matches!(e, Effect::ViewportInput(_))),
        "pending dock drag must suppress viewport hover/wheel forwarding (ADR 0072), got: {effects:?}",
    );

    let activate_pos = Point::new(Px(tab_pos.x.0 + 20.0), tab_pos.y);
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position: activate_pos,
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let drag = harness
        .app
        .drag(fret_core::PointerId(0))
        .and_then(|d| d.payload::<DockPanelDragPayload>().map(|_| d))
        .expect("expected pending dock drag to create a DragSession after activation");
    assert!(
        drag.dragging,
        "expected drag session to start in dragging state"
    );
}
#[test]
fn pending_dock_drag_does_not_start_drag_session_on_pointer_up_before_activation() {
    let mut harness = DockViewportHarness::new();
    harness.layout();

    let tab_pos = harness.tab_point(0);
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: tab_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let _ = harness.app.take_effects();

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Up {
            position: tab_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let _ = harness.app.take_effects();

    assert!(
        harness.app.drag(fret_core::PointerId(0)).is_none(),
        "pending dock drag must not create a drag session if released before activation",
    );

    // After releasing, viewport hover forwarding should resume.
    let position = harness.viewport_point();
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let effects = harness.app.take_effects();
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::ViewportInput(_))),
        "expected viewport hover forwarding after pending drag is released, got: {effects:?}",
    );
}
#[test]
fn pending_dock_drag_clears_on_pointer_cancel() {
    let mut harness = DockViewportHarness::new();
    harness.layout();

    let tab_pos = harness.tab_point(0);
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: tab_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let _ = harness.app.take_effects();

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::PointerCancel(fret_core::PointerCancelEvent {
            pointer_id: fret_core::PointerId(0),
            position: Some(tab_pos),
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
            reason: fret_core::PointerCancelReason::LeftWindow,
        }),
    );
    let _ = harness.app.take_effects();

    assert!(
        harness.app.drag(fret_core::PointerId(0)).is_none(),
        "pending dock drag must not create a drag session on cancel",
    );

    let position = harness.viewport_point();
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let effects = harness.app.take_effects();
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::ViewportInput(_))),
        "expected viewport hover forwarding after pending drag cancel, got: {effects:?}",
    );
}
#[test]
fn pending_dock_drag_arbitration_is_pointer_keyed() {
    let mut harness = DockViewportHarness::new();
    harness.layout();

    let tab_pos = harness.tab_point(0);
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: tab_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let _ = harness.app.take_effects();

    let position = harness.viewport_point();
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(1),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let effects = harness.app.take_effects();
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::ViewportInput(_))),
        "pending dock drag for one pointer must not suppress viewport hover for other pointers, got: {effects:?}",
    );
}
#[test]
fn docking_tab_drag_threshold_is_configurable_via_settings() {
    let mut harness = DockViewportHarness::new();
    harness.layout();

    harness
        .app
        .set_global(fret_runtime::DockingInteractionSettings {
            tab_drag_threshold: Px(1000.0),
            ..Default::default()
        });

    let tab_pos = harness.tab_point(0);
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: tab_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let _ = harness.app.take_effects();

    let move_pos = Point::new(Px(tab_pos.x.0 + 40.0), tab_pos.y);
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position: move_pos,
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let _ = harness.app.take_effects();

    assert!(
        harness.app.drag(fret_core::PointerId(0)).is_none(),
        "expected large threshold to prevent activation",
    );

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Up {
            position: move_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let _ = harness.app.take_effects();

    harness
        .app
        .set_global(fret_runtime::DockingInteractionSettings {
            tab_drag_threshold: Px(0.0),
            ..Default::default()
        });

    let tab_pos = harness.tab_point(0);
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: tab_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let _ = harness.app.take_effects();

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position: tab_pos,
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let drag = harness
        .app
        .drag(fret_core::PointerId(0))
        .and_then(|d| d.payload::<DockPanelDragPayload>().map(|_| d));
    assert!(
        drag.is_some_and(|d| d.dragging),
        "expected zero threshold to activate immediately on first move",
    );
}
#[test]
fn dock_drag_latches_dock_preview_policy_on_activation() {
    let settings = fret_runtime::DockingInteractionSettings {
        tab_drag_threshold: Px(0.0),
        drag_inversion: fret_runtime::DockDragInversionSettings {
            modifier: fret_runtime::DockDragInversionModifier::Ctrl,
            policy: fret_runtime::DockDragInversionPolicy::DockOnlyWhenModifier,
        },
        ..Default::default()
    };

    // Case 1: drag starts without modifier, then modifier changes during drag.
    // The "dock previews enabled" flag must be latched at activation, not recomputed per event.
    {
        let mut harness = DockViewportHarness::new();
        harness.layout();
        harness.app.set_global(settings);

        let tab_pos = harness.tab_point(0);
        harness.ui.dispatch_event(
            &mut harness.app,
            &mut harness.text,
            &Event::Pointer(fret_core::PointerEvent::Down {
                position: tab_pos,
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: fret_core::PointerId(0),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let _ = harness.app.take_effects();

        harness.ui.dispatch_event(
            &mut harness.app,
            &mut harness.text,
            &Event::Pointer(fret_core::PointerEvent::Move {
                position: tab_pos,
                buttons: fret_core::MouseButtons {
                    left: true,
                    ..Default::default()
                },
                modifiers: Modifiers::default(),
                pointer_id: fret_core::PointerId(0),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let _ = harness.app.take_effects();

        let dock_previews_enabled = harness
            .app
            .drag(fret_core::PointerId(0))
            .and_then(|d| {
                d.payload::<DockPanelDragPayload>()
                    .map(|p| p.dock_previews_enabled)
            })
            .expect("expected an active dock drag session");
        assert!(
            !dock_previews_enabled,
            "expected docking previews to be disabled without modifier"
        );

        let position = Point::new(Px(400.0), Px(300.0));
        harness.ui.dispatch_event(
            &mut harness.app,
            &mut harness.text,
            &Event::InternalDrag(InternalDragEvent {
                position,
                kind: InternalDragKind::Over,
                modifiers: Modifiers {
                    ctrl: true,
                    ..Default::default()
                },
                pointer_id: fret_core::PointerId(0),
            }),
        );

        let hover = harness
            .app
            .global::<DockManager>()
            .and_then(|d| d.hover.clone());
        assert!(
            matches!(hover, Some(DockDropTarget::Float { window }) if window == harness.window),
            "expected latched preview state to keep hover as Float even when modifier changes, got: {hover:?}",
        );
    }

    // Case 2: drag starts with modifier, then modifier is released. Must remain dock-enabled.
    {
        let mut harness = DockViewportHarness::new();
        harness.layout();
        harness.app.set_global(settings);

        let tab_pos = harness.tab_point(0);
        harness.ui.dispatch_event(
            &mut harness.app,
            &mut harness.text,
            &Event::Pointer(fret_core::PointerEvent::Down {
                position: tab_pos,
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers {
                    ctrl: true,
                    ..Default::default()
                },
                click_count: 1,
                pointer_id: fret_core::PointerId(0),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let _ = harness.app.take_effects();

        harness.ui.dispatch_event(
            &mut harness.app,
            &mut harness.text,
            &Event::Pointer(fret_core::PointerEvent::Move {
                position: tab_pos,
                buttons: fret_core::MouseButtons {
                    left: true,
                    ..Default::default()
                },
                modifiers: Modifiers {
                    ctrl: true,
                    ..Default::default()
                },
                pointer_id: fret_core::PointerId(0),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let _ = harness.app.take_effects();

        let dock_previews_enabled = harness
            .app
            .drag(fret_core::PointerId(0))
            .and_then(|d| {
                d.payload::<DockPanelDragPayload>()
                    .map(|p| p.dock_previews_enabled)
            })
            .expect("expected an active dock drag session");
        assert!(
            dock_previews_enabled,
            "expected docking previews to be enabled with modifier"
        );

        let position = Point::new(Px(400.0), Px(300.0));
        harness.ui.dispatch_event(
            &mut harness.app,
            &mut harness.text,
            &Event::InternalDrag(InternalDragEvent {
                position,
                kind: InternalDragKind::Over,
                modifiers: Modifiers::default(),
                pointer_id: fret_core::PointerId(0),
            }),
        );

        let hover = harness
            .app
            .global::<DockManager>()
            .and_then(|d| d.hover.clone());
        assert!(
            matches!(hover, Some(DockDropTarget::Dock(_))),
            "expected latched preview state to keep hover as Dock even when modifier is released, got: {hover:?}",
        );
    }
}
#[test]
fn dock_drag_requests_animation_frames_while_dragging() {
    let mut harness = DockViewportHarness::new();
    harness.layout();

    harness.app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(0),
        DRAG_KIND_DOCK_PANEL,
        harness.window,
        Point::new(Px(12.0), Px(12.0)),
        DockPanelDragPayload {
            panel: PanelKey::new("core.viewport"),
            grab_offset: Point::new(Px(0.0), Px(0.0)),
            start_tick: fret_runtime::TickId(0),
            tear_off_requested: false,
            tear_off_oob_start_frame: None,
            dock_previews_enabled: true,
        },
    );
    if let Some(drag) = harness.app.drag_mut(fret_core::PointerId(0)) {
        drag.dragging = true;
    }
    let _ = harness.app.take_effects();

    harness.layout();

    let effects = harness.app.take_effects();
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::RequestAnimationFrame(w) if *w == harness.window)),
        "expected dock drag to request animation frames, got: {effects:?}",
    );
}
#[test]
fn dock_drag_suppresses_viewport_capture_start_for_other_pointer() {
    let mut harness = DockViewportHarness::new();
    harness.layout();

    harness.app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(7),
        DRAG_KIND_DOCK_PANEL,
        harness.window,
        Point::new(Px(12.0), Px(12.0)),
        DockPanelDragPayload {
            panel: PanelKey::new("core.viewport"),
            grab_offset: Point::new(Px(0.0), Px(0.0)),
            start_tick: fret_runtime::TickId(0),
            tear_off_requested: false,
            tear_off_oob_start_frame: None,
            dock_previews_enabled: true,
        },
    );
    let _ = harness.app.take_effects();

    let down_pos = harness.viewport_point();
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: down_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let effects = harness.app.take_effects();
    assert!(
        !effects
            .iter()
            .any(|e| matches!(e, Effect::ViewportInput(_))),
        "expected viewport capture not to start during dock drag, got: {effects:?}"
    );
    assert_eq!(
        harness.ui.captured_for(fret_core::PointerId(0)),
        None,
        "expected viewport capture not to request pointer capture during dock drag"
    );
}
#[test]
fn dock_drag_only_requests_tear_off_after_stable_oob_frame() {
    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node_retained(DockSpace::new(window));
    ui.set_root(root);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.with_global_mut(DockManager::default, |dock, _app| {
        let tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![PanelKey::new("core.hierarchy")],
            active: 0,
        });
        dock.graph.set_window_root(window, tabs);
        dock.panels.insert(
            PanelKey::new("core.hierarchy"),
            DockPanel {
                title: "Hierarchy".to_string(),
                color: Color::TRANSPARENT,
                viewport: None,
            },
        );
    });

    app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(0),
        DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(24.0), Px(12.0)),
        DockPanelDragPayload {
            panel: PanelKey::new("core.hierarchy"),
            grab_offset: Point::new(Px(0.0), Px(0.0)),
            start_tick: fret_runtime::TickId(0),
            tear_off_requested: false,
            tear_off_oob_start_frame: None,
            dock_previews_enabled: true,
        },
    );
    if let Some(drag) = app.drag_mut(fret_core::PointerId(0)) {
        drag.dragging = true;
    }

    let mut text = FakeTextService;
    let size = Size::new(Px(800.0), Px(600.0));
    let outside = Point::new(Px(-32.0), Px(12.0));

    // Frame N: first OOB hover should not request a tear-off yet (debounce).
    app.advance_frame();
    let _ = ui.layout(&mut app, &mut text, root, size, 1.0);
    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::InternalDrag(InternalDragEvent {
            position: outside,
            kind: InternalDragKind::Over,
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
        }),
    );
    let effects = app.take_effects();
    assert!(
        !effects.iter().any(|e| matches!(
            e,
            Effect::Dock(DockOp::RequestFloatPanelToNewWindow { panel, .. })
                if *panel == PanelKey::new("core.hierarchy")
        )),
        "expected no tear-off request on first OOB over, got: {effects:?}"
    );

    // Frame N+1: still OOB -> request tear-off.
    app.advance_frame();
    let _ = ui.layout(&mut app, &mut text, root, size, 1.0);
    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::InternalDrag(InternalDragEvent {
            position: outside,
            kind: InternalDragKind::Over,
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
        }),
    );
    let effects = app.take_effects();
    assert!(
        effects.iter().any(|e| matches!(
            e,
            Effect::Dock(DockOp::RequestFloatPanelToNewWindow { panel, .. })
                if *panel == PanelKey::new("core.hierarchy")
        )),
        "expected tear-off request after stable OOB, got: {effects:?}"
    );
}
#[test]
fn dock_drag_over_floating_title_bar_resolves_center_dock_target() {
    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node_retained(DockSpace::new(window));
    ui.set_root(root);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.with_global_mut(DockManager::default, |dock, _app| {
        let main_tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![PanelKey::new("core.hierarchy")],
            active: 0,
        });
        dock.graph.set_window_root(window, main_tabs);

        let floating_tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![PanelKey::new("core.inspector")],
            active: 0,
        });
        let floating = dock.graph.insert_node(DockNode::Floating {
            child: floating_tabs,
        });
        dock.graph
            .floating_windows_mut(window)
            .push(fret_core::DockFloatingWindow {
                floating,
                rect: Rect::new(
                    Point::new(Px(180.0), Px(140.0)),
                    Size::new(Px(320.0), Px(240.0)),
                ),
            });

        dock.panels.insert(
            PanelKey::new("core.hierarchy"),
            DockPanel {
                title: "Hierarchy".to_string(),
                color: Color::TRANSPARENT,
                viewport: None,
            },
        );
        dock.panels.insert(
            PanelKey::new("core.inspector"),
            DockPanel {
                title: "Inspector".to_string(),
                color: Color::TRANSPARENT,
                viewport: None,
            },
        );
    });

    app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(0),
        DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(24.0), Px(12.0)),
        DockPanelDragPayload {
            panel: PanelKey::new("core.hierarchy"),
            grab_offset: Point::new(Px(0.0), Px(0.0)),
            start_tick: fret_runtime::TickId(0),
            tear_off_requested: false,
            tear_off_oob_start_frame: None,
            dock_previews_enabled: true,
        },
    );
    if let Some(drag) = app.drag_mut(fret_core::PointerId(0)) {
        drag.dragging = true;
    }

    let mut text = FakeTextService;
    let size = Size::new(Px(800.0), Px(600.0));
    let _ = ui.layout(&mut app, &mut text, root, size, 1.0);

    // DockSpace's floating chrome constants (space.rs):
    // border=1, title_h=22 -> title bar is at y in [outer.y + 1, outer.y + 23).
    let title_bar_pos = Point::new(Px(200.0), Px(152.0));
    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::InternalDrag(InternalDragEvent {
            position: title_bar_pos,
            kind: InternalDragKind::Over,
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
        }),
    );

    let hover = app
        .global::<DockManager>()
        .and_then(|dock| dock.hover.clone());
    assert!(
        matches!(
            hover,
            Some(DockDropTarget::Dock(t))
                if t.zone == DropZone::Center && t.insert_index.is_none() && !t.outer && !t.explicit
        ),
        "expected center dock target when hovering floating title bar, got: {hover:?}"
    );
}
#[test]
fn dock_drag_requires_explicit_target_or_hint_rects() {
    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node_retained(DockSpace::new(window));
    ui.set_root(root);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.with_global_mut(DockManager::default, |dock, _app| {
        let tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![PanelKey::new("target.panel")],
            active: 0,
        });
        dock.graph.set_window_root(window, tabs);
        dock.panels.insert(
            PanelKey::new("target.panel"),
            DockPanel {
                title: "Target".to_string(),
                color: Color::TRANSPARENT,
                viewport: None,
            },
        );
        dock.panels.insert(
            PanelKey::new("drag.panel"),
            DockPanel {
                title: "Dragged".to_string(),
                color: Color::TRANSPARENT,
                viewport: None,
            },
        );
        dock.hover = None;
    });

    let mut text = FakeTextService;
    let size = Size::new(Px(800.0), Px(600.0));
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), size);
    ui.layout(&mut app, &mut text, root, size, 1.0);
    let mut scene = Scene::default();
    ui.paint(&mut app, &mut text, root, bounds, &mut scene, 1.0);

    app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(0),
        DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(12.0), Px(12.0)),
        DockPanelDragPayload {
            panel: PanelKey::new("drag.panel"),
            grab_offset: Point::new(Px(0.0), Px(0.0)),
            start_tick: fret_runtime::TickId(0),
            tear_off_requested: false,
            tear_off_oob_start_frame: None,
            dock_previews_enabled: true,
        },
    );
    if let Some(drag) = app.drag_mut(fret_core::PointerId(0)) {
        drag.dragging = true;
        drag.current_window = window;
    }

    let (_chrome, dock_bounds) = dock_space_regions(bounds);
    let (_tab_bar, content) = split_tab_bar(dock_bounds);

    // 1) Cursor in content but far from center pads: no hover (explicit gating).
    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::InternalDrag(InternalDragEvent {
            position: Point::new(Px(content.origin.x.0 + 6.0), Px(content.origin.y.0 + 6.0)),
            kind: InternalDragKind::Over,
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
        }),
    );
    let hover = app.global::<DockManager>().and_then(|d| d.hover.clone());
    assert!(
        hover.is_none(),
        "expected no hover when not over tab bar or explicit drop rects, got: {hover:?}",
    );

    // 2) Cursor over center pad: hover becomes Dock(Center).
    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::InternalDrag(InternalDragEvent {
            position: Point::new(
                Px(dock_bounds.origin.x.0 + dock_bounds.size.width.0 * 0.5),
                Px(dock_bounds.origin.y.0 + dock_bounds.size.height.0 * 0.5),
            ),
            kind: InternalDragKind::Over,
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
        }),
    );
    let hover = app.global::<DockManager>().and_then(|d| d.hover.clone());
    assert!(
        matches!(hover, Some(DockDropTarget::Dock(t)) if t.zone == DropZone::Center),
        "expected center pad to produce a dock hover target, got: {hover:?}",
    );
}
#[test]
fn dock_drag_auto_scrolls_tab_bar_near_edges() {
    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node_retained(DockSpace::new(window));
    ui.set_root(root);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let tabs_node = app.with_global_mut(DockManager::default, |dock, _app| {
        let tabs: Vec<PanelKey> = (0..30)
            .map(|i| PanelKey::new(format!("core.tab_{i}")))
            .collect();
        let tabs_node = dock.graph.insert_node(DockNode::Tabs { tabs, active: 0 });
        dock.graph.set_window_root(window, tabs_node);

        dock.panels.insert(
            PanelKey::new("drag.panel"),
            DockPanel {
                title: "Dragged".to_string(),
                color: Color::TRANSPARENT,
                viewport: None,
            },
        );

        dock.hover = None;
        tabs_node
    });

    let mut text = FakeTextService;
    let size = Size::new(Px(800.0), Px(600.0));
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), size);
    ui.layout(&mut app, &mut text, root, size, 1.0);
    let mut scene = Scene::default();
    ui.paint(&mut app, &mut text, root, bounds, &mut scene, 1.0);

    app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(0),
        DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(12.0), Px(12.0)),
        DockPanelDragPayload {
            panel: PanelKey::new("drag.panel"),
            grab_offset: Point::new(Px(0.0), Px(0.0)),
            start_tick: fret_runtime::TickId(0),
            tear_off_requested: false,
            tear_off_oob_start_frame: None,
            dock_previews_enabled: true,
        },
    );
    if let Some(drag) = app.drag_mut(fret_core::PointerId(0)) {
        drag.dragging = true;
        drag.current_window = window;
    }

    let (_chrome, dock_bounds) = dock_space_regions(bounds);
    let (tab_bar, _content) = split_tab_bar(dock_bounds);
    let pos_right = Point::new(
        Px(tab_bar.origin.x.0 + tab_bar.size.width.0 - 2.0),
        Px(tab_bar.origin.y.0 + 6.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::InternalDrag(InternalDragEvent {
            position: pos_right,
            kind: InternalDragKind::Over,
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
        }),
    );

    let hover = app.global::<DockManager>().and_then(|d| d.hover.clone());
    let first_ix = match hover {
        Some(DockDropTarget::Dock(t)) => {
            assert_eq!(t.tabs, tabs_node);
            t.insert_index.expect("expected a tab insert index")
        }
        other => panic!("expected a tab-bar dock hover target, got: {other:?}"),
    };

    let mut ix_after_scroll = first_ix;
    for _ in 0..6 {
        app.advance_frame();
        ui.dispatch_event(
            &mut app,
            &mut text,
            &Event::InternalDrag(InternalDragEvent {
                position: pos_right,
                kind: InternalDragKind::Over,
                modifiers: Modifiers::default(),
                pointer_id: fret_core::PointerId(0),
            }),
        );
        let hover = app.global::<DockManager>().and_then(|d| d.hover.clone());
        if let Some(DockDropTarget::Dock(t)) = hover {
            ix_after_scroll = t.insert_index.expect("expected insert index");
        }
    }

    assert!(
        ix_after_scroll > first_ix,
        "expected auto-scroll at the right edge to increase the insert index, before={first_ix}, after={ix_after_scroll}",
    );

    let pos_left = Point::new(Px(tab_bar.origin.x.0 + 2.0), Px(tab_bar.origin.y.0 + 6.0));
    let mut ix_after_scroll_back = ix_after_scroll;
    for _ in 0..6 {
        app.advance_frame();
        ui.dispatch_event(
            &mut app,
            &mut text,
            &Event::InternalDrag(InternalDragEvent {
                position: pos_left,
                kind: InternalDragKind::Over,
                modifiers: Modifiers::default(),
                pointer_id: fret_core::PointerId(0),
            }),
        );
        let hover = app.global::<DockManager>().and_then(|d| d.hover.clone());
        if let Some(DockDropTarget::Dock(t)) = hover {
            ix_after_scroll_back = t.insert_index.expect("expected insert index");
        }
    }

    assert!(
        ix_after_scroll_back < ix_after_scroll,
        "expected auto-scroll at the left edge to decrease the insert index, before={ix_after_scroll}, after={ix_after_scroll_back}",
    );
}
