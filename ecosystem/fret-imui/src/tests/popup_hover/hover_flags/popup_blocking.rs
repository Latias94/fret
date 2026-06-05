use super::*;

#[test]
fn hovered_allow_when_blocked_by_popup_reads_underlay_hit_test() {
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

    let popup_opened = Rc::new(Cell::new(false));
    let under_hovered_default = Rc::new(Cell::new(false));
    let under_hovered_allow_when_blocked = Rc::new(Cell::new(false));
    let under_hovered_raw = Rc::new(Cell::new(false));
    let under_hovered_raw_below_barrier = Rc::new(Cell::new(false));

    let popup_id = "imui-hovered-allow-when-blocked-popup";

    ui.request_semantics_snapshot();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-hovered-allow-when-blocked-popup",
        |cx| {
            render_imui_popup_modal_barrier_hover_scene(
                cx,
                popup_id,
                true,
                popup_opened.clone(),
                under_hovered_default.clone(),
                under_hovered_allow_when_blocked.clone(),
                under_hovered_raw.clone(),
                under_hovered_raw_below_barrier.clone(),
            )
        },
    );
    assert!(
        popup_opened.get(),
        "expected popup to be opened on first frame"
    );

    let under_bounds = bounds_for_test_id(&ui, "imui-underlay-item");
    let under_center = Point::new(
        Px(under_bounds.origin.x.0 + under_bounds.size.width.0 * 0.5),
        Px(under_bounds.origin.y.0 + under_bounds.size.height.0 * 0.5),
    );

    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        under_center,
        MouseButtons::default(),
    );

    if std::env::var_os("FRET_DEBUG_IMUI_HOVER_BLOCKED_BY_POPUP").is_some() {
        let dbg = ui.debug_hit_test(under_center);
        eprintln!(
            "allow_when_blocked_by_popup: hit={:?} barrier_root={:?} active_layer_roots={:?}",
            dbg.hit, dbg.barrier_root, dbg.active_layer_roots
        );
        if let Some(hit) = dbg.hit {
            let kind = ui.debug_declarative_instance_kind(&mut app, window, hit);
            let path = ui.debug_node_path(hit);
            eprintln!("allow_when_blocked_by_popup: hit kind={kind:?} path={path:?}");
        }
        let layers = ui.debug_layers_in_paint_order();
        eprintln!("allow_when_blocked_by_popup: layers={layers:?}");
    }

    app.advance_frame();
    ui.request_semantics_snapshot();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-hovered-allow-when-blocked-popup",
        |cx| {
            render_imui_popup_modal_barrier_hover_scene(
                cx,
                popup_id,
                false,
                popup_opened.clone(),
                under_hovered_default.clone(),
                under_hovered_allow_when_blocked.clone(),
                under_hovered_raw.clone(),
                under_hovered_raw_below_barrier.clone(),
            )
        },
    );

    assert!(popup_opened.get(), "expected popup to remain open");
    assert!(
        !under_hovered_default.get(),
        "expected underlay hovered=false when blocked by a popup"
    );
    assert!(
        !under_hovered_raw.get(),
        "expected raw hovered=false when blocked by a popup (active layers)"
    );
    assert!(
        under_hovered_raw_below_barrier.get(),
        "expected below-barrier raw hovered to be true over the underlay"
    );
    assert!(
        under_hovered_allow_when_blocked.get(),
        "expected AllowWhenBlockedByPopup hovered query to be true over the underlay"
    );
}
