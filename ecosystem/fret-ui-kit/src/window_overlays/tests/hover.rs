use super::*;

#[test]
fn hover_overlay_is_pointer_transparent_while_closing() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let base_open = app.models_mut().insert(false);
    let open = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // Base root (required so the window exists and rendering can proceed).
    let trigger =
        render_base_with_trigger(&mut ui, &mut app, &mut services, window, bounds, base_open);

    // Install a hover overlay layer that is still `present` but `open=false` (closing animation).
    begin_frame(&mut app, window);
    let hover_id = GlobalElementId(0x55);
    request_hover_overlay_for_window(
        &mut app,
        window,
        HoverOverlayRequest {
            id: hover_id,
            root_name: hover_overlay_root_name(hover_id),
            interactive: true,
            trigger,
            open,
            present: true,
            children: Vec::new(),
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let layer = app
        .with_global_mut(WindowOverlays::default, |overlays, _app| {
            overlays
                .hover_overlays
                .get(&(window, hover_id))
                .map(|p| p.layer)
        })
        .expect("hover overlay layer");

    let info = ui
        .debug_layers_in_paint_order()
        .into_iter()
        .find(|l| l.id == layer)
        .expect("hover debug layer info");

    assert!(info.visible);
    assert!(!info.blocks_underlay_input);
    assert!(
        !info.hit_testable,
        "expected hover overlays to become pointer-transparent during close transitions"
    );
    assert_eq!(
        info.pointer_occlusion,
        fret_ui::tree::PointerOcclusion::None
    );
    assert!(!info.wants_pointer_down_outside_events);
    assert!(!info.wants_pointer_move_events);
}

#[test]
fn hover_overlay_is_click_through_while_closing() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let _open = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // Base root (required so the window exists and rendering can proceed).
    begin_frame(&mut app, window);
    let base = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "base",
        |_| Vec::new(),
    );
    ui.set_root(base);

    // Install a hover overlay that is still present but non-interactive (closing animation).
    begin_frame(&mut app, window);
    let id = GlobalElementId(0xdead);
    let open = app.models_mut().insert(false);
    request_hover_overlay_for_window(
        &mut app,
        window,
        HoverOverlayRequest {
            id,
            root_name: hover_overlay_root_name(id),
            interactive: false,
            trigger: id,
            open,
            present: true,
            children: Vec::new(),
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let layer = app
        .with_global_mut(WindowOverlays::default, |overlays, _app| {
            overlays.hover_overlays.get(&(window, id)).map(|h| h.layer)
        })
        .expect("hover overlay layer");

    let info = ui
        .debug_layers_in_paint_order()
        .into_iter()
        .find(|l| l.id == layer)
        .expect("hover overlay debug layer info");

    assert!(info.visible);
    assert!(!info.blocks_underlay_input);
    assert!(!info.hit_testable);
    assert!(!info.wants_pointer_down_outside_events);
    assert!(!info.wants_pointer_move_events);

    let arbitration = crate::OverlayController::arbitration_snapshot(&ui);
    assert_eq!(
        arbitration.pointer_occlusion,
        fret_ui::tree::PointerOcclusion::None,
        "expected hover overlay close transition to be click-through"
    );
}
