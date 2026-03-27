use super::*;

#[test]
fn context_menu_popup_opens_on_right_click_and_closes_on_outside_click() {
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

    let open = Rc::new(Cell::new(false));
    let open_out = open.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu",
        |cx| {
            crate::imui(cx, |ui| {
                let resp = ui.button("OK");
                open_out.set(ui.begin_popup_context_menu("ctx", resp, |ui| {
                    ui.text("Menu");
                }));
            })
        },
    );
    assert!(!open.get());

    let at = first_child_point(&ui, root);
    right_click_at(&mut ui, &mut app, &mut services, at);

    app.advance_frame();
    let open_out = open.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu",
        |cx| {
            crate::imui(cx, |ui| {
                let resp = ui.button("OK");
                open_out.set(ui.begin_popup_context_menu("ctx", resp, |ui| {
                    ui.text("Menu");
                }));
            })
        },
    );
    assert!(open.get());

    click_at(
        &mut ui,
        &mut app,
        &mut services,
        Point::new(Px(230.0), Px(110.0)),
    );

    app.advance_frame();
    let open_out = open.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu",
        |cx| {
            crate::imui(cx, |ui| {
                let resp = ui.button("OK");
                open_out.set(ui.begin_popup_context_menu("ctx", resp, |ui| {
                    ui.text("Menu");
                }));
            })
        },
    );
    assert!(!open.get());
}

#[test]
fn context_menu_popup_closes_if_trigger_disappears_for_a_frame() {
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

    let open = Rc::new(Cell::new(false));
    let open_state = Rc::new(Cell::new(false));
    let open_out = open.clone();
    let open_state_out = open_state.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-disappear",
        |cx| {
            crate::imui(cx, |ui| {
                let resp = ui.button("OK");
                open_out.set(ui.begin_popup_context_menu("ctx", resp, |ui| {
                    ui.text("Menu");
                }));
                let model = ui.popup_open_model("ctx");
                open_state_out.set(ui.cx_mut().app.models().get_copied(&model).unwrap_or(false));
            })
        },
    );
    assert!(!open.get());
    assert!(!open_state.get());

    let at = first_child_point(&ui, root);
    right_click_at(&mut ui, &mut app, &mut services, at);

    app.advance_frame();
    let open_out = open.clone();
    let open_state_out = open_state.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-disappear",
        |cx| {
            crate::imui(cx, |ui| {
                let resp = ui.button("OK");
                open_out.set(ui.begin_popup_context_menu("ctx", resp, |ui| {
                    ui.text("Menu");
                }));
                let model = ui.popup_open_model("ctx");
                open_state_out.set(ui.cx_mut().app.models().get_copied(&model).unwrap_or(false));
            })
        },
    );
    assert!(open.get());
    assert!(open_state.get());

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-disappear",
        |cx| {
            crate::imui(cx, |ui| {
                ui.text("Trigger disappeared");
            })
        },
    );

    app.advance_frame();
    let open_out = open.clone();
    let open_state_out = open_state.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-disappear",
        |cx| {
            crate::imui(cx, |ui| {
                let resp = ui.button("OK");
                open_out.set(ui.begin_popup_context_menu("ctx", resp, |_ui| {}));
                let model = ui.popup_open_model("ctx");
                open_state_out.set(ui.cx_mut().app.models().get_copied(&model).unwrap_or(false));
            })
        },
    );
    assert!(!open.get());
    assert!(!open_state.get());
}

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
                crate::imui(cx, |ui| {
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
                        over_hovered.set(over.core.hovered);
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

#[test]
fn context_menu_popup_item_click_closes_popup() {
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

    let open = Rc::new(Cell::new(false));
    let open_out = open.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-item-close",
        |cx| {
            crate::imui(cx, |ui| {
                let resp = ui.button("OK");
                open_out.set(ui.begin_popup_context_menu_with_options(
                    "ctx",
                    resp,
                    PopupMenuOptions {
                        estimated_size: Size::new(Px(120.0), Px(60.0)),
                        ..Default::default()
                    },
                    |ui| {
                        let open_model = ui.popup_open_model("ctx");
                        ui.menu_item_with_options(
                            "Close",
                            MenuItemOptions {
                                close_popup: Some(open_model),
                                test_id: Some(Arc::from("imui-popup-ctx-item-close")),
                                ..Default::default()
                            },
                        );
                    },
                ));
            })
        },
    );
    assert!(!open.get());

    let at = first_child_point(&ui, root);
    right_click_at(&mut ui, &mut app, &mut services, at);

    app.advance_frame();
    ui.request_semantics_snapshot();
    let open_out = open.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-item-close",
        |cx| {
            crate::imui(cx, |ui| {
                let resp = ui.button("OK");
                open_out.set(ui.begin_popup_context_menu_with_options(
                    "ctx",
                    resp,
                    PopupMenuOptions {
                        estimated_size: Size::new(Px(120.0), Px(60.0)),
                        ..Default::default()
                    },
                    |ui| {
                        let open_model = ui.popup_open_model("ctx");
                        ui.menu_item_with_options(
                            "Close",
                            MenuItemOptions {
                                close_popup: Some(open_model),
                                test_id: Some(Arc::from("imui-popup-ctx-item-close")),
                                ..Default::default()
                            },
                        );
                    },
                ));
            })
        },
    );
    assert!(open.get());

    let item_bounds = bounds_for_test_id(&ui, "imui-popup-ctx-item-close");
    let click_point = Point::new(
        Px(item_bounds.origin.x.0 + item_bounds.size.width.0 * 0.5),
        Px(item_bounds.origin.y.0 + item_bounds.size.height.0 * 0.5),
    );
    let hit = ui.debug_hit_test(click_point).hit.expect("hit node");
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let hit_test_id = snap
        .nodes
        .iter()
        .find(|n| n.id == hit)
        .and_then(|n| n.test_id.as_deref());
    assert_eq!(
        hit_test_id,
        Some("imui-popup-ctx-item-close"),
        "expected click to hit the menu item pressable"
    );

    click_at(&mut ui, &mut app, &mut services, click_point);

    app.advance_frame();
    let open_out = open.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-item-close",
        |cx| {
            crate::imui(cx, |ui| {
                let resp = ui.button("OK");
                open_out.set(ui.begin_popup_context_menu("ctx", resp, |_ui| {}));
            })
        },
    );
    assert!(!open.get());
}

#[test]
fn context_menu_popup_keyboard_open_focuses_first_item_and_escape_restores_trigger_focus() {
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

    let open = Rc::new(Cell::new(false));
    let open_out = open.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-keyboard-open",
        |cx| {
            crate::imui(cx, |ui| {
                let resp = ui.button("OK");
                open_out.set(ui.begin_popup_context_menu_with_options(
                    "ctx",
                    resp,
                    PopupMenuOptions {
                        estimated_size: Size::new(Px(160.0), Px(90.0)),
                        ..Default::default()
                    },
                    |ui| {
                        ui.menu_item_with_options(
                            "Item A",
                            MenuItemOptions {
                                test_id: Some(Arc::from("imui-popup-ctx-item-a")),
                                ..Default::default()
                            },
                        );
                        ui.menu_item_with_options(
                            "Item B",
                            MenuItemOptions {
                                test_id: Some(Arc::from("imui-popup-ctx-item-b")),
                                ..Default::default()
                            },
                        );
                    },
                ));
            })
        },
    );
    assert!(!open.get());

    let at = first_child_point(&ui, root);
    click_at(&mut ui, &mut app, &mut services, at);
    let focus_before_open = ui.focus();
    assert!(
        focus_before_open.is_some(),
        "expected trigger to take focus"
    );

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::ContextMenu,
        Modifiers::default(),
    );

    app.advance_frame();
    ui.request_semantics_snapshot();
    let open_out = open.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-keyboard-open",
        |cx| {
            crate::imui(cx, |ui| {
                let resp = ui.button("OK");
                open_out.set(ui.begin_popup_context_menu_with_options(
                    "ctx",
                    resp,
                    PopupMenuOptions {
                        estimated_size: Size::new(Px(160.0), Px(90.0)),
                        ..Default::default()
                    },
                    |ui| {
                        ui.menu_item_with_options(
                            "Item A",
                            MenuItemOptions {
                                test_id: Some(Arc::from("imui-popup-ctx-item-a")),
                                ..Default::default()
                            },
                        );
                        ui.menu_item_with_options(
                            "Item B",
                            MenuItemOptions {
                                test_id: Some(Arc::from("imui-popup-ctx-item-b")),
                                ..Default::default()
                            },
                        );
                    },
                ));
            })
        },
    );
    assert!(open.get());

    let focus = ui.focus().expect("focus");
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let focused_test_id = snap
        .nodes
        .iter()
        .find(|n| n.id == focus)
        .and_then(|n| n.test_id.as_deref());
    assert_eq!(focused_test_id, Some("imui-popup-ctx-item-a"));

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::Escape,
        Modifiers::default(),
    );

    app.advance_frame();
    let open_out = open.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-keyboard-open",
        |cx| {
            crate::imui(cx, |ui| {
                let resp = ui.button("OK");
                open_out.set(ui.begin_popup_context_menu("ctx", resp, |_ui| {}));
            })
        },
    );
    assert!(!open.get());
    assert_eq!(ui.focus(), focus_before_open);
}

#[test]
fn context_menu_popup_arrow_keys_move_focus_between_items() {
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

    let open = Rc::new(Cell::new(false));

    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-arrow-nav",
        |cx| {
            crate::imui(cx, |ui| {
                let resp = ui.button("OK");
                let open_out = open.clone();
                open_out.set(ui.begin_popup_context_menu_with_options(
                    "ctx",
                    resp,
                    PopupMenuOptions {
                        estimated_size: Size::new(Px(160.0), Px(90.0)),
                        ..Default::default()
                    },
                    |ui| {
                        ui.menu_item_with_options(
                            "Item A",
                            MenuItemOptions {
                                test_id: Some(Arc::from("imui-popup-ctx-item-a")),
                                ..Default::default()
                            },
                        );
                        ui.menu_item_with_options(
                            "Item B",
                            MenuItemOptions {
                                test_id: Some(Arc::from("imui-popup-ctx-item-b")),
                                ..Default::default()
                            },
                        );
                    },
                ));
            })
        },
    );
    assert!(!open.get());

    let at = first_child_point(&ui, root);
    click_at(&mut ui, &mut app, &mut services, at);
    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::ContextMenu,
        Modifiers::default(),
    );

    app.advance_frame();
    ui.request_semantics_snapshot();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-arrow-nav",
        |cx| {
            crate::imui(cx, |ui| {
                let resp = ui.button("OK");
                let open_out = open.clone();
                open_out.set(ui.begin_popup_context_menu_with_options(
                    "ctx",
                    resp,
                    PopupMenuOptions {
                        estimated_size: Size::new(Px(160.0), Px(90.0)),
                        ..Default::default()
                    },
                    |ui| {
                        ui.menu_item_with_options(
                            "Item A",
                            MenuItemOptions {
                                test_id: Some(Arc::from("imui-popup-ctx-item-a")),
                                ..Default::default()
                            },
                        );
                        ui.menu_item_with_options(
                            "Item B",
                            MenuItemOptions {
                                test_id: Some(Arc::from("imui-popup-ctx-item-b")),
                                ..Default::default()
                            },
                        );
                    },
                ));
            })
        },
    );
    assert!(open.get());

    let focus = ui.focus().expect("focus");
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let focused_test_id = snap
        .nodes
        .iter()
        .find(|n| n.id == focus)
        .and_then(|n| n.test_id.as_deref());
    assert_eq!(focused_test_id, Some("imui-popup-ctx-item-a"));

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::ArrowDown,
        Modifiers::default(),
    );

    app.advance_frame();
    ui.request_semantics_snapshot();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-arrow-nav",
        |cx| {
            crate::imui(cx, |ui| {
                let resp = ui.button("OK");
                let open_out = open.clone();
                open_out.set(ui.begin_popup_context_menu_with_options(
                    "ctx",
                    resp,
                    PopupMenuOptions {
                        estimated_size: Size::new(Px(160.0), Px(90.0)),
                        ..Default::default()
                    },
                    |ui| {
                        ui.menu_item_with_options(
                            "Item A",
                            MenuItemOptions {
                                test_id: Some(Arc::from("imui-popup-ctx-item-a")),
                                ..Default::default()
                            },
                        );
                        ui.menu_item_with_options(
                            "Item B",
                            MenuItemOptions {
                                test_id: Some(Arc::from("imui-popup-ctx-item-b")),
                                ..Default::default()
                            },
                        );
                    },
                ));
            })
        },
    );

    let focus = ui.focus().expect("focus");
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let focused_test_id = snap
        .nodes
        .iter()
        .find(|n| n.id == focus)
        .and_then(|n| n.test_id.as_deref());
    assert_eq!(focused_test_id, Some("imui-popup-ctx-item-b"));

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::ArrowUp,
        Modifiers::default(),
    );

    app.advance_frame();
    ui.request_semantics_snapshot();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-arrow-nav",
        |cx| {
            crate::imui(cx, |ui| {
                let resp = ui.button("OK");
                let open_out = open.clone();
                open_out.set(ui.begin_popup_context_menu_with_options(
                    "ctx",
                    resp,
                    PopupMenuOptions {
                        estimated_size: Size::new(Px(160.0), Px(90.0)),
                        ..Default::default()
                    },
                    |ui| {
                        ui.menu_item_with_options(
                            "Item A",
                            MenuItemOptions {
                                test_id: Some(Arc::from("imui-popup-ctx-item-a")),
                                ..Default::default()
                            },
                        );
                        ui.menu_item_with_options(
                            "Item B",
                            MenuItemOptions {
                                test_id: Some(Arc::from("imui-popup-ctx-item-b")),
                                ..Default::default()
                            },
                        );
                    },
                ));
            })
        },
    );

    let focus = ui.focus().expect("focus");
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let focused_test_id = snap
        .nodes
        .iter()
        .find(|n| n.id == focus)
        .and_then(|n| n.test_id.as_deref());
    assert_eq!(focused_test_id, Some("imui-popup-ctx-item-a"));
}

#[test]
fn menu_item_checkbox_stamps_semantics_checked_state() {
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

    let open = Rc::new(Cell::new(false));

    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-menu-item-checkbox-semantics",
        |cx| {
            crate::imui(cx, |ui| {
                let resp = ui.button("OK");
                let open_out = open.clone();
                open_out.set(ui.begin_popup_context_menu_with_options(
                    "ctx",
                    resp,
                    PopupMenuOptions {
                        estimated_size: Size::new(Px(160.0), Px(90.0)),
                        ..Default::default()
                    },
                    |ui| {
                        ui.menu_item_checkbox_with_options(
                            "Flag",
                            true,
                            MenuItemOptions {
                                test_id: Some(Arc::from("imui-popup-ctx-item-flag")),
                                ..Default::default()
                            },
                        );
                    },
                ));
            })
        },
    );
    assert!(!open.get());

    let at = first_child_point(&ui, root);
    click_at(&mut ui, &mut app, &mut services, at);
    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::ContextMenu,
        Modifiers::default(),
    );

    app.advance_frame();
    ui.request_semantics_snapshot();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-menu-item-checkbox-semantics",
        |cx| {
            crate::imui(cx, |ui| {
                let resp = ui.button("OK");
                let open_out = open.clone();
                open_out.set(ui.begin_popup_context_menu_with_options(
                    "ctx",
                    resp,
                    PopupMenuOptions {
                        estimated_size: Size::new(Px(160.0), Px(90.0)),
                        ..Default::default()
                    },
                    |ui| {
                        ui.menu_item_checkbox_with_options(
                            "Flag",
                            true,
                            MenuItemOptions {
                                test_id: Some(Arc::from("imui-popup-ctx-item-flag")),
                                ..Default::default()
                            },
                        );
                    },
                ));
            })
        },
    );
    assert!(open.get());

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let node = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("imui-popup-ctx-item-flag"))
        .expect("checkbox node");
    assert_eq!(node.role, SemanticsRole::MenuItemCheckbox);
    assert_eq!(node.flags.checked, Some(true));
}

#[test]
fn drop_popup_scope_closes_and_forgets_internal_state() {
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

    let model = app.models_mut().insert(None::<Arc<str>>);
    let items = vec![Arc::<str>::from("Alpha"), Arc::<str>::from("Beta")];
    let popup_scope_id: Arc<str> = Arc::from("imui-drop-popup-scope");

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drop-popup-scope",
        |cx| {
            crate::imui(cx, |ui| {
                let _ = ui.select_model_with_options(
                    "Mode",
                    &model,
                    &items,
                    SelectOptions {
                        test_id: Some(Arc::from("imui-drop-popup-trigger")),
                        popup_scope_id: Some(popup_scope_id.clone()),
                        ..Default::default()
                    },
                );
            })
        },
    );

    let trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-drop-popup-trigger",
    );
    click_at(&mut ui, &mut app, &mut services, trigger);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drop-popup-scope",
        |cx| {
            crate::imui(cx, |ui| {
                let _ = ui.select_model_with_options(
                    "Mode",
                    &model,
                    &items,
                    SelectOptions {
                        test_id: Some(Arc::from("imui-drop-popup-trigger")),
                        popup_scope_id: Some(popup_scope_id.clone()),
                        ..Default::default()
                    },
                );
            })
        },
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-popup-imui-drop-popup-scope",
    ));

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drop-popup-scope",
        |cx| {
            crate::imui(cx, |ui| {
                ui.drop_popup_scope(popup_scope_id.as_ref());
                let _ = ui.select_model_with_options(
                    "Mode",
                    &model,
                    &items,
                    SelectOptions {
                        test_id: Some(Arc::from("imui-drop-popup-trigger")),
                        popup_scope_id: Some(popup_scope_id.clone()),
                        ..Default::default()
                    },
                );
            })
        },
    );
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-popup-imui-drop-popup-scope",
    ));
}

#[test]
fn popup_closes_after_one_frame_without_keep_alive() {
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

    let popup_id = "imui-popup-auto-close";
    let anchor = Rect::new(Point::new(Px(12.0), Px(12.0)), Size::new(Px(1.0), Px(1.0)));

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-auto-close",
        |cx| {
            crate::imui(cx, |ui| {
                ui.open_popup_at(popup_id, anchor);
                // Intentionally do not call `begin_popup_menu*` this frame.
            })
        },
    );

    app.advance_frame();
    let open_state = Rc::new(Cell::new(false));
    let open_state_out = open_state.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-auto-close",
        |cx| {
            crate::imui(cx, |ui| {
                let open = ui.popup_open_model(popup_id);
                open_state_out.set(ui.cx_mut().app.models().get_copied(&open).unwrap_or(false));
            })
        },
    );

    assert!(open_state.get());

    app.advance_frame();
    let opened = Rc::new(Cell::new(false));
    let open_state = Rc::new(Cell::new(false));
    let opened_out = opened.clone();
    let open_state_out = open_state.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-auto-close",
        |cx| {
            crate::imui(cx, |ui| {
                opened_out.set(ui.begin_popup_menu(popup_id, None, |_ui| {}));
                let open = ui.popup_open_model(popup_id);
                open_state_out.set(ui.cx_mut().app.models().get_copied(&open).unwrap_or(false));
            })
        },
    );

    assert!(!opened.get());
    assert!(!open_state.get());
}

#[test]
fn popup_modal_default_outside_press_does_not_close_and_escape_closes() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(200.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let popup_id = "imui-popup-modal-default";
    let modal_test_id = format!("imui-popup-modal-{popup_id}");
    let opened = Rc::new(Cell::new(false));
    let open_state = Rc::new(Cell::new(false));
    let bootstrap_open = Rc::new(Cell::new(true));

    let opened_out = opened.clone();
    let open_state_out = open_state.clone();
    let bootstrap_open_out = bootstrap_open.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-modal-default-outside",
        |cx| {
            crate::imui(cx, |ui| {
                if bootstrap_open_out.replace(false) {
                    ui.open_popup(popup_id);
                }
                opened_out.set(ui.begin_popup_modal_with_options(
                    popup_id,
                    None,
                    PopupModalOptions {
                        size: Size::new(Px(160.0), Px(96.0)),
                        ..Default::default()
                    },
                    |ui| {
                        ui.text("Modal");
                    },
                ));
                let open = ui.popup_open_model(popup_id);
                open_state_out.set(ui.cx_mut().app.models().get_copied(&open).unwrap_or(false));
            })
        },
    );

    assert!(opened.get());
    assert!(open_state.get());
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        modal_test_id.as_str(),
    ));

    click_at(
        &mut ui,
        &mut app,
        &mut services,
        Point::new(Px(8.0), Px(8.0)),
    );

    app.advance_frame();
    let opened_out = opened.clone();
    let open_state_out = open_state.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-modal-default-outside",
        |cx| {
            crate::imui(cx, |ui| {
                opened_out.set(ui.begin_popup_modal_with_options(
                    popup_id,
                    None,
                    PopupModalOptions {
                        size: Size::new(Px(160.0), Px(96.0)),
                        ..Default::default()
                    },
                    |ui| {
                        ui.text("Modal");
                    },
                ));
                let open = ui.popup_open_model(popup_id);
                open_state_out.set(ui.cx_mut().app.models().get_copied(&open).unwrap_or(false));
            })
        },
    );

    assert!(opened.get());
    assert!(open_state.get());
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        modal_test_id.as_str(),
    ));

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::Escape,
        Modifiers::default(),
    );

    app.advance_frame();
    let opened_out = opened.clone();
    let open_state_out = open_state.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-modal-default-outside",
        |cx| {
            crate::imui(cx, |ui| {
                opened_out.set(ui.begin_popup_modal_with_options(
                    popup_id,
                    None,
                    PopupModalOptions {
                        size: Size::new(Px(160.0), Px(96.0)),
                        ..Default::default()
                    },
                    |ui| {
                        ui.text("Modal");
                    },
                ));
                let open = ui.popup_open_model(popup_id);
                open_state_out.set(ui.cx_mut().app.models().get_copied(&open).unwrap_or(false));
            })
        },
    );

    app.advance_frame();
    let opened_out = opened.clone();
    let open_state_out = open_state.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-modal-outside-close",
        |cx| {
            crate::imui(cx, |ui| {
                opened_out.set(ui.begin_popup_modal_with_options(
                    popup_id,
                    None,
                    PopupModalOptions {
                        size: Size::new(Px(160.0), Px(96.0)),
                        close_on_outside_press: true,
                    },
                    |ui| {
                        ui.text("Modal");
                    },
                ));
                let open = ui.popup_open_model(popup_id);
                open_state_out.set(ui.cx_mut().app.models().get_copied(&open).unwrap_or(false));
            })
        },
    );

    assert!(!opened.get());
    assert!(!open_state.get());
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        modal_test_id.as_str(),
    ));
}

#[test]
fn popup_modal_can_close_on_outside_press_when_enabled() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(200.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let popup_id = "imui-popup-modal-outside-close";
    let modal_test_id = format!("imui-popup-modal-{popup_id}");
    let opened = Rc::new(Cell::new(false));
    let open_state = Rc::new(Cell::new(false));
    let bootstrap_open = Rc::new(Cell::new(true));

    let opened_out = opened.clone();
    let open_state_out = open_state.clone();
    let bootstrap_open_out = bootstrap_open.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-modal-outside-close",
        |cx| {
            crate::imui(cx, |ui| {
                if bootstrap_open_out.replace(false) {
                    ui.open_popup(popup_id);
                }
                opened_out.set(ui.begin_popup_modal_with_options(
                    popup_id,
                    None,
                    PopupModalOptions {
                        size: Size::new(Px(160.0), Px(96.0)),
                        close_on_outside_press: true,
                    },
                    |ui| {
                        ui.text("Modal");
                    },
                ));
                let open = ui.popup_open_model(popup_id);
                open_state_out.set(ui.cx_mut().app.models().get_copied(&open).unwrap_or(false));
            })
        },
    );

    assert!(opened.get());
    assert!(open_state.get());
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        modal_test_id.as_str(),
    ));

    click_at(
        &mut ui,
        &mut app,
        &mut services,
        Point::new(Px(8.0), Px(8.0)),
    );

    app.advance_frame();
    let opened_out = opened.clone();
    let open_state_out = open_state.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-modal-outside-close",
        |cx| {
            crate::imui(cx, |ui| {
                opened_out.set(ui.begin_popup_modal_with_options(
                    popup_id,
                    None,
                    PopupModalOptions {
                        size: Size::new(Px(160.0), Px(96.0)),
                        close_on_outside_press: true,
                    },
                    |ui| {
                        ui.text("Modal");
                    },
                ));
                let open = ui.popup_open_model(popup_id);
                open_state_out.set(ui.cx_mut().app.models().get_copied(&open).unwrap_or(false));
            })
        },
    );

    assert!(!opened.get());
    assert!(!open_state.get());
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        modal_test_id.as_str(),
    ));
}
