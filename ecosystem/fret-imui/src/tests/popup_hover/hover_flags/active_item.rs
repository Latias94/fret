use super::*;

#[test]
fn hovered_allow_when_blocked_by_active_item_allows_hover_while_other_item_is_active() {
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

    let b_core_hovered = Rc::new(Cell::new(false));
    let b_blocked_by_active_item = Rc::new(Cell::new(false));
    let b_hovered_default = Rc::new(Cell::new(false));
    let b_hovered_allow_when_blocked = Rc::new(Cell::new(false));
    let a_hovered = Rc::new(Cell::new(false));
    let a_focused = Rc::new(Cell::new(false));

    ui.request_semantics_snapshot();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-active-item-blocks-hover",
        |cx| {
            render_imui_active_item_blocks_hover_scene(
                cx,
                a_hovered.clone(),
                a_focused.clone(),
                b_core_hovered.clone(),
                b_blocked_by_active_item.clone(),
                b_hovered_default.clone(),
                b_hovered_allow_when_blocked.clone(),
            )
        },
    );

    let a_bounds = bounds_for_test_id(&ui, "imui-active-item-a");
    let a_center = Point::new(
        Px(a_bounds.origin.x.0 + a_bounds.size.width.0 * 0.5),
        Px(a_bounds.origin.y.0 + a_bounds.size.height.0 * 0.5),
    );
    let b_bounds = bounds_for_test_id(&ui, "imui-active-item-b");
    let b_center = Point::new(
        Px(b_bounds.origin.x.0 + b_bounds.size.width.0 * 0.5),
        Px(b_bounds.origin.y.0 + b_bounds.size.height.0 * 0.5),
    );

    if std::env::var_os("FRET_DEBUG_IMUI_ACTIVE_ITEM_BLOCKS_HOVER").is_some() {
        for (name, center) in [("a", a_center), ("b", b_center)] {
            let dbg = ui.debug_hit_test(center);
            eprintln!(
                "active_item_blocks_hover: {name} center={center:?} hit={:?} barrier_root={:?} active_layer_roots={:?}",
                dbg.hit, dbg.barrier_root, dbg.active_layer_roots
            );
            if let Some(hit) = dbg.hit {
                let kind = ui.debug_declarative_instance_kind(&mut app, window, hit);
                let path = ui.debug_node_path(hit);
                eprintln!("active_item_blocks_hover: {name} hit kind={kind:?} path={path:?}");
            }
        }
    }

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
        "imui-active-item-blocks-hover",
        |cx| {
            render_imui_active_item_blocks_hover_scene(
                cx,
                a_hovered.clone(),
                a_focused.clone(),
                b_core_hovered.clone(),
                b_blocked_by_active_item.clone(),
                b_hovered_default.clone(),
                b_hovered_allow_when_blocked.clone(),
            )
        },
    );

    pointer_down_at(&mut ui, &mut app, &mut services, a_center);
    app.advance_frame();
    ui.request_semantics_snapshot();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-active-item-blocks-hover",
        |cx| {
            render_imui_active_item_blocks_hover_scene(
                cx,
                a_hovered.clone(),
                a_focused.clone(),
                b_core_hovered.clone(),
                b_blocked_by_active_item.clone(),
                b_hovered_default.clone(),
                b_hovered_allow_when_blocked.clone(),
            )
        },
    );

    assert!(
        a_focused.get(),
        "expected A to be focused after pointer-down"
    );

    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        b_center,
        MouseButtons {
            left: true,
            ..Default::default()
        },
    );
    app.advance_frame();
    ui.request_semantics_snapshot();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-active-item-blocks-hover",
        |cx| {
            render_imui_active_item_blocks_hover_scene(
                cx,
                a_hovered.clone(),
                a_focused.clone(),
                b_core_hovered.clone(),
                b_blocked_by_active_item.clone(),
                b_hovered_default.clone(),
                b_hovered_allow_when_blocked.clone(),
            )
        },
    );

    assert!(
        b_core_hovered.get(),
        "expected B to be hovered by hit-test under the pointer"
    );
    assert!(
        b_blocked_by_active_item.get(),
        "expected B to be blocked by active-item suppression while A is active"
    );
    assert!(
        !b_hovered_default.get(),
        "expected hovered query to be suppressed while another item is active"
    );
    assert!(
        b_hovered_allow_when_blocked.get(),
        "expected AllowWhenBlockedByActiveItem hovered query to be true under the pointer"
    );

    pointer_up_at_with_is_click(&mut ui, &mut app, &mut services, b_center, false);
}
