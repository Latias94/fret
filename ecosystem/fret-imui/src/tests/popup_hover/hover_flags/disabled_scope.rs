use super::*;

#[test]
fn disabled_scope_blocks_underlay_and_suppresses_hover_and_click() {
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

    let under_clicked = Rc::new(Cell::new(false));
    let over_clicked = Rc::new(Cell::new(false));
    let over_hovered = Rc::new(Cell::new(false));
    let over_hovered_like_imgui = Rc::new(Cell::new(false));
    let over_hovered_allow_when_disabled = Rc::new(Cell::new(false));
    let over_id_before: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
        Rc::new(Cell::new(None));
    let over_id_after: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
        Rc::new(Cell::new(None));

    ui.request_semantics_snapshot();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-disabled-scope",
        |cx| {
            render_imui_disabled_scope_overlay_scene(
                cx,
                under_clicked.clone(),
                over_clicked.clone(),
                over_hovered.clone(),
                over_hovered_like_imgui.clone(),
                over_hovered_allow_when_disabled.clone(),
                over_id_before.clone(),
            )
        },
    );

    let over_bounds = bounds_for_test_id(&ui, "imui-overlay-item");
    let over_center = Point::new(
        Px(over_bounds.origin.x.0 + over_bounds.size.width.0 * 0.5),
        Px(over_bounds.origin.y.0 + over_bounds.size.height.0 * 0.5),
    );

    let under_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-underlay-item",
    );

    let hit = ui
        .debug_hit_test(over_center)
        .hit
        .expect("expected overlay center to hit a node");
    let path = ui.debug_node_path(hit);
    if std::env::var_os("FRET_DEBUG_IMUI_DISABLED_SCOPE_HOVER").is_some() {
        let kind = ui.debug_declarative_instance_kind(&mut app, window, hit);
        eprintln!("disabled_scope: over_center hit={hit:?} kind={kind:?} path={path:?}");
    }
    assert!(
        !path.contains(&under_node),
        "expected underlay to be skipped when covered by a disabled overlay"
    );

    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        over_center,
        MouseButtons::default(),
    );
    app.advance_frame();
    ui.request_semantics_snapshot();
    let _ = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-disabled-scope",
        |cx| {
            render_imui_disabled_scope_overlay_scene(
                cx,
                under_clicked.clone(),
                over_clicked.clone(),
                over_hovered.clone(),
                over_hovered_like_imgui.clone(),
                over_hovered_allow_when_disabled.clone(),
                over_id_after.clone(),
            )
        },
    );
    if std::env::var_os("FRET_DEBUG_IMUI_DISABLED_SCOPE_HOVER").is_some() {
        eprintln!(
            "disabled_scope: over_id_before={:?} over_id_after={:?}",
            over_id_before.get(),
            over_id_after.get()
        );
    }

    assert!(
        !over_hovered.get(),
        "expected disabled items to report hovered=false by default"
    );
    assert!(
        !over_hovered_like_imgui.get(),
        "expected hovered_like_imgui to be false when disabled"
    );
    assert!(
        over_hovered_allow_when_disabled.get(),
        "expected AllowWhenDisabled hovered query to be true over a disabled item"
    );

    click_at(&mut ui, &mut app, &mut services, over_center);
    app.advance_frame();
    ui.request_semantics_snapshot();
    let _ = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-disabled-scope",
        |cx| {
            let under_clicked = under_clicked.clone();
            let over_clicked = over_clicked.clone();
            let over_hovered = over_hovered.clone();
            let over_hovered_like_imgui = over_hovered_like_imgui.clone();
            let mut stack = fret_ui::element::StackProps::default();
            stack.layout.size.width = Length::Fill;
            let element = cx.stack_props(stack, |cx| {
                crate::imui_raw(cx, |ui| {
                    let under = ui.menu_item_with_options(
                        "Underlay",
                        MenuItemOptions {
                            test_id: Some(Arc::from("imui-underlay-item")),
                            ..Default::default()
                        },
                    );
                    under_clicked.set(under.clicked());

                    ui.disabled_scope(true, |ui| {
                        let over = ui.menu_item_with_options(
                            "Overlay",
                            MenuItemOptions {
                                test_id: Some(Arc::from("imui-overlay-item")),
                                ..Default::default()
                            },
                        );
                        over_clicked.set(over.clicked());
                        over_hovered.set(over.hovered());
                        over_hovered_like_imgui.set(over.hovered_like_imgui());
                    });
                })
            });
            vec![element].into()
        },
    );

    assert!(
        !over_clicked.get(),
        "expected disabled overlay item to suppress clicked()"
    );
    assert!(
        !under_clicked.get(),
        "expected disabled overlay item to block clicks from reaching the underlay"
    );

    // Keep `root` alive to ensure the overlay layer stack is present for debugging.
    let _ = root;
}
