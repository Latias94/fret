use super::*;

#[test]
fn fret_menubar_menu_tracks_trigger_when_underlay_scrolls() {
    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let mut services = StyleAwareServices::default();

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(480.0), Px(400.0)),
    );

    let scroll_handle = ScrollHandle::default();
    let file_test_id = "scroll-underlay-menubar-file";

    let render = |cx: &mut ElementContext<'_, App>| {
        use fret_ui_shadcn::menubar::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

        let scroll_handle = scroll_handle.clone();

        let mut scroll_layout = LayoutStyle::default();
        scroll_layout.size.width = Length::Fill;
        scroll_layout.size.height = Length::Fill;
        scroll_layout.overflow = fret_ui::element::Overflow::Clip;

        vec![cx.scroll(
            fret_ui::element::ScrollProps {
                layout: scroll_layout,
                axis: fret_ui::element::ScrollAxis::Y,
                scroll_handle: Some(scroll_handle),
                ..Default::default()
            },
            move |cx| {
                vec![cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Px(Px(1200.0));
                            layout
                        },
                        ..Default::default()
                    },
                    move |cx| {
                        vec![cx.container(
                            ContainerProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.position = fret_ui::element::PositionStyle::Absolute;
                                    layout.inset.left = Some(Px(16.0));
                                    layout.inset.top = Some(Px(160.0));
                                    layout.size.width = Length::Px(Px(320.0));
                                    layout.size.height = Length::Px(Px(32.0));
                                    layout
                                },
                                ..Default::default()
                            },
                            move |cx| {
                                vec![
                                    Menubar::new(vec![
                                        MenubarMenu::new("File").test_id(file_test_id).entries(
                                            vec![
                                                MenubarEntry::Item(MenubarItem::new("New")),
                                                MenubarEntry::Item(MenubarItem::new("Open")),
                                                MenubarEntry::Item(MenubarItem::new("Exit")),
                                            ],
                                        ),
                                        MenubarMenu::new("Edit").entries(vec![MenubarEntry::Item(
                                            MenubarItem::new("Undo"),
                                        )]),
                                    ])
                                    .into_element(cx),
                                ]
                            },
                        )]
                    },
                )]
            },
        )]
    };

    // Frame 1: mount closed and locate the trigger.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        render,
    );

    let snap0 = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let file_trigger = find_semantics_by_test_id(&snap0, file_test_id).expect("file trigger");
    let file_center = Point::new(
        Px(file_trigger.bounds.origin.x.0 + file_trigger.bounds.size.width.0 * 0.5),
        Px(file_trigger.bounds.origin.y.0 + file_trigger.bounds.size.height.0 * 0.5),
    );

    // Click the menubar trigger to open the menu.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: file_center,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: file_center,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    // Frame 2+: open and settle motion to avoid interpreting the open animation as scroll drift.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        false,
        render,
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(3 + tick),
            request_semantics,
            render,
        );
    }
    let _ = app.flush_effects();

    let snap_before = ui
        .semantics_snapshot()
        .expect("semantics snapshot (before scroll)")
        .clone();
    let trigger_before = find_semantics_by_test_id(&snap_before, file_test_id).expect("trigger");
    let menu_before = snap_before
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Menu)
        .expect("menu semantics (before scroll)");

    let dx_before = menu_before.bounds.origin.x.0 - trigger_before.bounds.origin.x.0;
    let dy_before = menu_before.bounds.origin.y.0 - trigger_before.bounds.origin.y.0;

    // Scroll the underlay (wheel over the scroll viewport, not over the menu panel).
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Wheel {
            pointer_id: fret_core::PointerId::default(),
            position: Point::new(Px(10.0), Px(10.0)),
            delta: Point::new(Px(0.0), Px(-80.0)),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
    assert!(
        scroll_handle.offset().y.0 > 0.01,
        "expected scroll handle offset to update after wheel; y={}",
        scroll_handle.offset().y.0
    );

    // Frame N: apply the scroll and paint once so scroll transforms update visual bounds caches.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(3 + settle_frames),
        false,
        render,
    );
    let mut scene = fret_core::Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let effects = app.flush_effects();
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::Redraw(w) if *w == window)),
        "expected a follow-up redraw after scroll to re-anchor overlays; effects={effects:?}",
    );
    for effect in effects {
        match effect {
            Effect::Redraw(_) => {}
            other => app.push_effect(other),
        }
    }

    // Frame N+1: expected to re-anchor the menu to the scrolled trigger.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(4 + settle_frames),
        true,
        render,
    );

    let snap_after = ui
        .semantics_snapshot()
        .expect("semantics snapshot (after scroll)")
        .clone();
    let trigger_after = find_semantics_by_test_id(&snap_after, file_test_id).expect("trigger");
    let menu_after = snap_after
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Menu)
        .expect("menu semantics (after scroll)");

    assert!(
        (trigger_after.bounds.origin.y.0 - trigger_before.bounds.origin.y.0).abs() > 1.0,
        "expected trigger to move under scroll (before_y={} after_y={})",
        trigger_before.bounds.origin.y.0,
        trigger_after.bounds.origin.y.0
    );

    let dx_after = menu_after.bounds.origin.x.0 - trigger_after.bounds.origin.x.0;
    let dy_after = menu_after.bounds.origin.y.0 - trigger_after.bounds.origin.y.0;

    assert_close(
        "menubar menu anchor dx stable under scroll",
        dx_after,
        dx_before,
        1.0,
    );
    assert_close(
        "menubar menu anchor dy stable under scroll",
        dy_after,
        dy_before,
        1.0,
    );
}
#[test]
fn web_vs_fret_menubar_demo_overlay_placement_matches() {
    let web = read_web_golden_open("menubar-demo");
    let theme = web_theme(&web);

    let web_trigger = web_find_by_data_slot_and_state(&theme.root, "menubar-trigger", "open")
        .unwrap_or_else(|| {
            find_first(&theme.root, &|n| {
                n.attrs
                    .get("data-slot")
                    .is_some_and(|v| v.as_str() == "menubar-trigger")
            })
            .expect("web trigger slot=menubar-trigger")
        });

    let menu_label = web_trigger.text.as_deref().unwrap_or("File");

    let web_portal_index = theme
        .portals
        .iter()
        .position(|n| n.attrs.get("role").is_some_and(|v| v == "menu"))
        .expect("web portal role=menu");
    let web_portal_leaf = &theme.portals[web_portal_index];
    let web_portal = theme
        .portal_wrappers
        .get(web_portal_index)
        .unwrap_or(web_portal_leaf);

    let web_side = find_attr_in_subtree(web_portal_leaf, "data-side")
        .or_else(|| find_attr_in_subtree(web_portal, "data-side"))
        .and_then(parse_side)
        .unwrap_or_else(|| infer_side(web_trigger.rect, web_portal.rect));
    let web_align = find_attr_in_subtree(web_portal_leaf, "data-align")
        .or_else(|| find_attr_in_subtree(web_portal, "data-align"))
        .and_then(parse_align)
        .unwrap_or_else(|| infer_align(web_side, web_trigger.rect, web_portal.rect));

    let expected_gap = rect_main_gap(web_side, web_trigger.rect, web_portal.rect);
    let expected_cross = rect_cross_delta(web_side, web_align, web_trigger.rect, web_portal.rect);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);
    let view_bookmarks_bar: Model<bool> = app.models_mut().insert(false);
    let view_full_urls: Model<bool> = app.models_mut().insert(true);
    let profile_value: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("benoit")));

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let menubar = build_menubar_demo(
                cx,
                view_bookmarks_bar.clone(),
                view_full_urls.clone(),
                profile_value.clone(),
            );
            vec![pad_root(cx, Px(0.0), menubar)]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some(menu_label))
        .unwrap_or_else(|| panic!("fret menubar trigger semantics ({menu_label})"));
    let click_point = Point::new(
        Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
        Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let menubar = build_menubar_demo(
                    cx,
                    view_bookmarks_bar.clone(),
                    view_full_urls.clone(),
                    profile_value.clone(),
                );
                vec![pad_root(cx, Px(0.0), menubar)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some(menu_label))
        .unwrap_or_else(|| panic!("fret menubar trigger semantics ({menu_label})"));

    let expected_portal_w = web_portal.rect.w;
    let expected_portal_h = web_portal.rect.h;

    let fret_trigger = WebRect {
        x: trigger.bounds.origin.x.0,
        y: trigger.bounds.origin.y.0,
        w: trigger.bounds.size.width.0,
        h: trigger.bounds.size.height.0,
    };

    let debug = std::env::var("FRET_DEBUG_OVERLAY_PLACEMENT")
        .ok()
        .is_some_and(|v| v == "1");

    let portal = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::Menu)
        .min_by(|a, b| {
            let rect_a = WebRect {
                x: a.bounds.origin.x.0,
                y: a.bounds.origin.y.0,
                w: a.bounds.size.width.0,
                h: a.bounds.size.height.0,
            };
            let rect_b = WebRect {
                x: b.bounds.origin.x.0,
                y: b.bounds.origin.y.0,
                w: b.bounds.size.width.0,
                h: b.bounds.size.height.0,
            };

            let score_for = |r: WebRect| {
                let gap = rect_main_gap(web_side, fret_trigger, r);
                let cross = rect_cross_delta(web_side, web_align, fret_trigger, r);
                let size = (r.w - expected_portal_w).abs() + (r.h - expected_portal_h).abs();
                (gap - expected_gap).abs() + (cross - expected_cross).abs() + 0.05 * size
            };

            let score_a = score_for(rect_a);
            let score_b = score_for(rect_b);
            score_a
                .partial_cmp(&score_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .expect("fret menubar portal semantics (Menu)");

    if debug {
        let candidates: Vec<_> = snap
            .nodes
            .iter()
            .filter(|n| n.role == SemanticsRole::Menu)
            .collect();
        eprintln!("menubar-demo fret Menu candidates: {}", candidates.len());
        for (idx, n) in candidates.iter().enumerate().take(8) {
            eprintln!("  [{idx}] bounds={:?} label={:?}", n.bounds, n.label);
        }
        eprintln!(
            "menubar-demo web trigger={:?} web portal={:?}\n  fret trigger={:?}\n  selected portal={:?}",
            web_trigger.rect, web_portal.rect, fret_trigger, portal.bounds
        );
        eprintln!(
            "menubar-demo fret trigger flags={:?} root_count={} node_count={}",
            trigger.flags,
            snap.roots.len(),
            snap.nodes.len()
        );
    }

    let fret_portal = WebRect {
        x: portal.bounds.origin.x.0,
        y: portal.bounds.origin.y.0,
        w: portal.bounds.size.width.0,
        h: portal.bounds.size.height.0,
    };

    let actual_gap = rect_main_gap(web_side, fret_trigger, fret_portal);
    let actual_cross = rect_cross_delta(web_side, web_align, fret_trigger, fret_portal);

    assert_close("menubar-demo main_gap", actual_gap, expected_gap, 1.0);
    assert_close(
        "menubar-demo cross_delta",
        actual_cross,
        expected_cross,
        1.5,
    );
    assert_close(
        "menubar-demo portal_w",
        fret_portal.w,
        expected_portal_w,
        2.0,
    );
    assert_close(
        "menubar-demo portal_h",
        fret_portal.h,
        expected_portal_h,
        2.0,
    );
}
#[test]
fn web_vs_fret_menubar_demo_view_overlay_placement_matches() {
    assert_menubar_demo_constrained_overlay_placement_matches("menubar-demo.view");
}
#[test]
fn web_vs_fret_menubar_demo_profiles_overlay_placement_matches() {
    assert_menubar_demo_constrained_overlay_placement_matches("menubar-demo.profiles");
}
#[test]
fn web_vs_fret_menubar_demo_view_checkbox_indicator_slot_inset_matches_web() {
    assert_menubar_demo_checkbox_indicator_slot_inset_matches_web_impl("menubar-demo.view");
}
#[test]
fn web_vs_fret_menubar_demo_profiles_radio_indicator_slot_inset_matches_web() {
    assert_menubar_demo_radio_indicator_slot_inset_matches_web_impl("menubar-demo.profiles");
}
#[test]
fn web_vs_fret_menubar_demo_small_viewport_overlay_placement_matches() {
    assert_menubar_demo_constrained_overlay_placement_matches("menubar-demo.vp1440x320");
}
#[test]
fn web_vs_fret_menubar_demo_tiny_viewport_overlay_placement_matches() {
    assert_menubar_demo_constrained_overlay_placement_matches("menubar-demo.vp1440x240");
}
#[test]
fn web_vs_fret_menubar_demo_mobile_tiny_viewport_overlay_placement_matches() {
    assert_menubar_demo_constrained_overlay_placement_matches("menubar-demo.vp375x240");
}
#[test]
fn web_vs_fret_menubar_demo_small_viewport_menu_item_height_matches() {
    assert_menubar_demo_constrained_menu_item_height_matches("menubar-demo.vp1440x320");
}
#[test]
fn web_vs_fret_menubar_demo_tiny_viewport_menu_item_height_matches() {
    assert_menubar_demo_constrained_menu_item_height_matches("menubar-demo.vp1440x240");
}
#[test]
fn web_vs_fret_menubar_demo_mobile_tiny_viewport_menu_item_height_matches() {
    assert_menubar_demo_constrained_menu_item_height_matches("menubar-demo.vp375x240");
}
#[test]
fn web_vs_fret_menubar_demo_menu_item_height_matches() {
    assert_menubar_demo_constrained_menu_item_height_matches("menubar-demo");
}
#[test]
fn web_vs_fret_menubar_demo_view_menu_item_height_matches() {
    assert_menubar_demo_constrained_menu_item_height_matches("menubar-demo.view");
}
#[test]
fn web_vs_fret_menubar_demo_profiles_menu_item_height_matches() {
    assert_menubar_demo_constrained_menu_item_height_matches("menubar-demo.profiles");
}
#[test]
fn web_vs_fret_menubar_demo_item_padding_and_shortcut_match() {
    assert_menubar_demo_item_padding_and_shortcut_match_impl("menubar-demo");
}
#[test]
fn web_vs_fret_menubar_demo_small_viewport_menu_content_insets_match() {
    assert_menubar_demo_constrained_menu_content_insets_match("menubar-demo.vp1440x320");
}
#[test]
fn web_vs_fret_menubar_demo_tiny_viewport_menu_content_insets_match() {
    assert_menubar_demo_constrained_menu_content_insets_match("menubar-demo.vp1440x240");
}
#[test]
fn web_vs_fret_menubar_demo_mobile_tiny_viewport_menu_content_insets_match() {
    assert_menubar_demo_constrained_menu_content_insets_match("menubar-demo.vp375x240");
}
#[test]
fn web_vs_fret_menubar_demo_small_viewport_scroll_state_matches() {
    assert_menubar_demo_constrained_scroll_state_matches("menubar-demo.vp1440x320");
}
#[test]
fn web_vs_fret_menubar_demo_tiny_viewport_scroll_state_matches() {
    assert_menubar_demo_constrained_scroll_state_matches("menubar-demo.vp1440x240");
}
#[test]
fn web_vs_fret_menubar_demo_mobile_tiny_viewport_scroll_state_matches() {
    assert_menubar_demo_constrained_scroll_state_matches("menubar-demo.vp375x240");
}
#[test]
fn web_vs_fret_menubar_demo_mobile_tiny_viewport_wheel_does_not_move_overlay() {
    assert_menubar_demo_wheel_does_not_move_overlay("menubar-demo.vp375x240", -80.0);
}
#[test]
fn web_vs_fret_menubar_demo_menu_content_insets_match() {
    assert_menubar_demo_constrained_menu_content_insets_match("menubar-demo");
}
#[test]
fn web_vs_fret_menubar_demo_view_menu_content_insets_match() {
    assert_menubar_demo_constrained_menu_content_insets_match("menubar-demo.view");
}
#[test]
fn web_vs_fret_menubar_demo_profiles_menu_content_insets_match() {
    assert_menubar_demo_constrained_menu_content_insets_match("menubar-demo.profiles");
}
#[test]
fn web_vs_fret_menubar_demo_submenu_overlay_placement_matches() {
    assert_menubar_demo_submenu_overlay_placement_matches("menubar-demo.submenu-kbd");
}
#[test]
fn web_vs_fret_menubar_demo_submenu_hover_overlay_placement_matches() {
    assert_menubar_demo_submenu_overlay_placement_matches("menubar-demo.submenu");
}
#[test]
fn web_vs_fret_menubar_demo_submenu_small_viewport_overlay_placement_matches() {
    assert_menubar_demo_submenu_overlay_placement_matches("menubar-demo.submenu-kbd-vp1440x320");
}
#[test]
fn web_vs_fret_menubar_demo_submenu_tiny_viewport_overlay_placement_matches() {
    assert_menubar_demo_submenu_overlay_placement_matches("menubar-demo.submenu-kbd-vp1440x240");
}
#[test]
fn web_vs_fret_menubar_demo_submenu_mobile_tiny_viewport_overlay_placement_matches() {
    assert_menubar_demo_submenu_overlay_placement_matches("menubar-demo.submenu-kbd-vp375x240");
}
#[test]
fn web_vs_fret_menubar_demo_submenu_small_viewport_menu_content_insets_match() {
    assert_menubar_demo_submenu_constrained_menu_content_insets_match(
        "menubar-demo.submenu-kbd-vp1440x320",
    );
}
#[test]
fn web_vs_fret_menubar_demo_submenu_tiny_viewport_menu_content_insets_match() {
    assert_menubar_demo_submenu_constrained_menu_content_insets_match(
        "menubar-demo.submenu-kbd-vp1440x240",
    );
}
#[test]
fn web_vs_fret_menubar_demo_submenu_mobile_tiny_viewport_menu_content_insets_match() {
    assert_menubar_demo_submenu_constrained_menu_content_insets_match(
        "menubar-demo.submenu-kbd-vp375x240",
    );
}
#[test]
fn web_vs_fret_menubar_demo_submenu_menu_content_insets_match() {
    assert_menubar_demo_submenu_constrained_menu_content_insets_match("menubar-demo.submenu-kbd");
}
#[test]
fn web_vs_fret_menubar_demo_submenu_hover_menu_content_insets_match() {
    assert_menubar_demo_submenu_constrained_menu_content_insets_match("menubar-demo.submenu");
}
#[test]
fn web_vs_fret_menubar_demo_submenu_first_visible_matches() {
    assert_menubar_demo_submenu_first_visible_matches("menubar-demo.submenu-kbd");
}
#[test]
fn web_vs_fret_menubar_demo_submenu_hover_first_visible_matches() {
    assert_menubar_demo_submenu_first_visible_matches("menubar-demo.submenu");
}
#[test]
fn web_vs_fret_menubar_demo_submenu_small_viewport_first_visible_matches() {
    assert_menubar_demo_submenu_first_visible_matches("menubar-demo.submenu-kbd-vp1440x320");
}
#[test]
fn web_vs_fret_menubar_demo_submenu_tiny_viewport_first_visible_matches() {
    assert_menubar_demo_submenu_first_visible_matches("menubar-demo.submenu-kbd-vp1440x240");
}
#[test]
fn web_vs_fret_menubar_demo_submenu_mobile_tiny_viewport_first_visible_matches() {
    assert_menubar_demo_submenu_first_visible_matches("menubar-demo.submenu-kbd-vp375x240");
}
#[test]
fn web_vs_fret_menubar_demo_submenu_menu_item_height_matches() {
    assert_menubar_demo_submenu_menu_item_height_matches("menubar-demo.submenu-kbd");
}
#[test]
fn web_vs_fret_menubar_demo_submenu_hover_menu_item_height_matches() {
    assert_menubar_demo_submenu_menu_item_height_matches("menubar-demo.submenu");
}
