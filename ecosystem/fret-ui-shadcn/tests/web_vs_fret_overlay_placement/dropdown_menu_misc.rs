use super::*;

#[test]
fn fret_dropdown_menu_tracks_trigger_when_underlay_scrolls() {
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

    let open = app.models_mut().insert(false);
    let scroll_handle = ScrollHandle::default();

    let trigger_test_id = "scroll-underlay-trigger";

    let render = |cx: &mut ElementContext<'_, App>, open: &Model<bool>| {
        use fret_ui_shadcn::dropdown_menu::{DropdownMenu, DropdownMenuEntry, DropdownMenuItem};

        let open = open.clone();
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
                        let dropdown = DropdownMenu::new(open.clone());

                        vec![dropdown.into_element(
                            cx,
                            move |cx| {
                                cx.semantics(
                                    fret_ui::element::SemanticsProps {
                                        layout: {
                                            let mut layout = LayoutStyle::default();
                                            layout.position =
                                                fret_ui::element::PositionStyle::Absolute;
                                            layout.inset.left = Some(Px(16.0));
                                            layout.inset.top = Some(Px(160.0));
                                            layout.size.width = Length::Px(Px(120.0));
                                            layout.size.height = Length::Px(Px(32.0));
                                            layout
                                        },
                                        role: SemanticsRole::Button,
                                        label: Some(Arc::from("ScrollUnderlayTrigger")),
                                        test_id: Some(Arc::from(trigger_test_id)),
                                        ..Default::default()
                                    },
                                    move |cx| {
                                        vec![cx.pressable_with_id(
                                            fret_ui::element::PressableProps {
                                                layout: {
                                                    let mut layout = LayoutStyle::default();
                                                    layout.size.width = Length::Fill;
                                                    layout.size.height = Length::Fill;
                                                    layout
                                                },
                                                enabled: true,
                                                focusable: true,
                                                ..Default::default()
                                            },
                                            |cx, _st, _id| vec![cx.text("Open")],
                                        )]
                                    },
                                )
                            },
                            move |_cx| {
                                vec![
                                    DropdownMenuEntry::Item(DropdownMenuItem::new("Item A")),
                                    DropdownMenuEntry::Item(DropdownMenuItem::new("Item B")),
                                    DropdownMenuEntry::Item(DropdownMenuItem::new("Item C")),
                                ]
                            },
                        )]
                    },
                )]
            },
        )]
    };

    // Frame 1: mount closed so the trigger element id mapping is stable.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| render(cx, &open),
    );

    let _ = app.models_mut().update(&open, |v| *v = true);

    // Frame 2+: open and settle motion to avoid interpreting the open animation as scroll drift.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| render(cx, &open),
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
            |cx| render(cx, &open),
        );
    }

    let snap_before = ui
        .semantics_snapshot()
        .expect("semantics snapshot (before scroll)")
        .clone();
    let trigger_before =
        find_semantics_by_test_id(&snap_before, trigger_test_id).expect("trigger semantics");
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

    // Frame N: apply the scroll. Anchored overlays place using last-frame bounds, so this frame
    // may still use the pre-scroll trigger rect. The runtime should request a follow-up redraw
    // so the next frame can re-anchor using the updated bounds caches.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(3 + settle_frames),
        false,
        |cx| render(cx, &open),
    );

    // Paint once so scroll-induced child render transforms are reflected in last-frame visual
    // bounds caches (used by anchored overlay placement).
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
        |cx| render(cx, &open),
    );

    let snap_after = ui
        .semantics_snapshot()
        .expect("semantics snapshot (after scroll)")
        .clone();
    let trigger_after =
        find_semantics_by_test_id(&snap_after, trigger_test_id).expect("trigger semantics");
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
        "dropdown menu anchor dx stable under scroll",
        dx_after,
        dx_before,
        1.0,
    );
    assert_close(
        "dropdown menu anchor dy stable under scroll",
        dy_after,
        dy_before,
        1.0,
    );
}
#[test]
fn web_vs_fret_item_dropdown_menu_item_height_matches() {
    let web_name = "item-dropdown";
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let expected_hs = web_portal_slot_heights(&theme, &["dropdown-menu-item"]);
    let expected_h = expected_hs
        .iter()
        .copied()
        .next()
        .unwrap_or_else(|| panic!("missing web dropdown-menu-item height for {web_name}"));

    let snap = build_item_dropdown_open_snapshot(theme, expected_h.round());
    let actual_hs = fret_menu_item_heights_in_menus(&snap);
    assert_menu_item_row_height_matches(web_name, expected_h.round(), &actual_hs, 1.0);
}
#[test]
fn web_vs_fret_item_dropdown_menu_item_height_matches_mobile_tiny_viewport() {
    let web_name = "item-dropdown.vp375x240";
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let expected_hs = web_portal_slot_heights(&theme, &["dropdown-menu-item"]);
    let expected_h = expected_hs
        .iter()
        .copied()
        .next()
        .unwrap_or_else(|| panic!("missing web dropdown-menu-item height for {web_name}"));

    let snap = build_item_dropdown_open_snapshot(theme, expected_h.round());
    let actual_hs = fret_menu_item_heights_in_menus(&snap);
    assert_menu_item_row_height_matches(web_name, expected_h.round(), &actual_hs, 1.0);
}
#[test]
fn web_vs_fret_item_dropdown_menu_content_insets_match() {
    let web_name = "item-dropdown";
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let expected = web_menu_content_insets_for_slots(&theme, &["dropdown-menu-content"]);
    let expected_item_hs = web_portal_slot_heights(&theme, &["dropdown-menu-item"]);
    let expected_item_h = expected_item_hs
        .iter()
        .copied()
        .next()
        .unwrap_or_else(|| panic!("missing web dropdown-menu-item height for {web_name}"));
    let expected_menu_h = web_portal_node_by_data_slot(&theme, "dropdown-menu-content")
        .rect
        .h;

    let snap = build_item_dropdown_open_snapshot(theme, expected_item_h.round());
    let actual = fret_menu_content_insets(&snap);
    assert_sorted_insets_match(web_name, &actual, &expected);

    let actual_menu_h = fret_largest_menu_height(&snap)
        .unwrap_or_else(|| panic!("missing fret menu for {web_name}"));
    assert_close(
        &format!("{web_name} menu height"),
        actual_menu_h,
        expected_menu_h,
        2.0,
    );
}
#[test]
fn web_vs_fret_item_dropdown_menu_content_insets_match_mobile_tiny_viewport() {
    let web_name = "item-dropdown.vp375x240";
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let expected = web_menu_content_insets_for_slots(&theme, &["dropdown-menu-content"]);
    let expected_item_hs = web_portal_slot_heights(&theme, &["dropdown-menu-item"]);
    let expected_item_h = expected_item_hs
        .iter()
        .copied()
        .next()
        .unwrap_or_else(|| panic!("missing web dropdown-menu-item height for {web_name}"));
    let expected_menu_h = web_portal_node_by_data_slot(&theme, "dropdown-menu-content")
        .rect
        .h;

    let snap = build_item_dropdown_open_snapshot(theme, expected_item_h.round());
    let actual = fret_menu_content_insets(&snap);
    assert_sorted_insets_match(web_name, &actual, &expected);

    let actual_menu_h = fret_largest_menu_height(&snap)
        .unwrap_or_else(|| panic!("missing fret menu for {web_name}"));
    assert_close(
        &format!("{web_name} menu height"),
        actual_menu_h,
        expected_menu_h,
        2.0,
    );
}
#[test]
fn web_vs_fret_button_group_demo_dropdown_menu_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "button-group-demo",
        Some("menu"),
        |cx, open| render_button_group_demo_dropdown_menu(cx, open.clone()),
        SemanticsRole::Button,
        Some("More Options"),
        SemanticsRole::Menu,
    );
}
#[test]
fn web_vs_fret_button_group_demo_dropdown_menu_overlay_placement_matches_mobile_tiny_viewport() {
    assert_overlay_placement_matches(
        "button-group-demo.vp375x240",
        Some("menu"),
        |cx, open| render_button_group_demo_dropdown_menu(cx, open.clone()),
        SemanticsRole::Button,
        Some("More Options"),
        SemanticsRole::Menu,
    );
}
#[test]
fn web_vs_fret_mode_toggle_dropdown_menu_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "mode-toggle",
        Some("menu"),
        |cx, open| {
            use fret_ui_shadcn::{
                Button, ButtonSize, ButtonVariant, DropdownMenu, DropdownMenuAlign,
                DropdownMenuEntry, DropdownMenuItem,
            };

            fn icon_stub<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
                cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(16.0));
                            layout.size.height = Length::Px(Px(16.0));
                            layout
                        },
                        ..Default::default()
                    },
                    |_cx| Vec::new(),
                )
            }

            DropdownMenu::new(open.clone())
                .align(DropdownMenuAlign::End)
                .into_element(
                    cx,
                    |cx| {
                        Button::new("Toggle theme")
                            .variant(ButtonVariant::Outline)
                            .size(ButtonSize::Icon)
                            .children([icon_stub(cx)])
                            .into_element(cx)
                    },
                    |_cx| {
                        vec![
                            DropdownMenuEntry::Item(DropdownMenuItem::new("Light")),
                            DropdownMenuEntry::Item(DropdownMenuItem::new("Dark")),
                            DropdownMenuEntry::Item(DropdownMenuItem::new("System")),
                        ]
                    },
                )
        },
        SemanticsRole::Button,
        Some("Toggle theme"),
        SemanticsRole::Menu,
    );
}
#[test]
fn web_vs_fret_mode_toggle_dropdown_menu_overlay_placement_matches_mobile_tiny_viewport() {
    assert_overlay_placement_matches(
        "mode-toggle.vp375x240",
        Some("menu"),
        |cx, open| {
            use fret_ui_shadcn::{
                Button, ButtonSize, ButtonVariant, DropdownMenu, DropdownMenuAlign,
                DropdownMenuEntry, DropdownMenuItem,
            };

            fn icon_stub<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
                cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(16.0));
                            layout.size.height = Length::Px(Px(16.0));
                            layout
                        },
                        ..Default::default()
                    },
                    |_cx| Vec::new(),
                )
            }

            DropdownMenu::new(open.clone())
                .align(DropdownMenuAlign::End)
                .into_element(
                    cx,
                    |cx| {
                        Button::new("Toggle theme")
                            .variant(ButtonVariant::Outline)
                            .size(ButtonSize::Icon)
                            .children([icon_stub(cx)])
                            .into_element(cx)
                    },
                    |_cx| {
                        vec![
                            DropdownMenuEntry::Item(DropdownMenuItem::new("Light")),
                            DropdownMenuEntry::Item(DropdownMenuItem::new("Dark")),
                            DropdownMenuEntry::Item(DropdownMenuItem::new("System")),
                        ]
                    },
                )
        },
        SemanticsRole::Button,
        Some("Toggle theme"),
        SemanticsRole::Menu,
    );
}
#[test]
fn web_vs_fret_combobox_dropdown_menu_overlay_placement_matches() {
    assert_combobox_dropdown_menu_overlay_placement_matches("combobox-dropdown-menu");
}
#[test]
fn web_vs_fret_combobox_dropdown_menu_overlay_placement_matches_mobile_tiny_viewport() {
    assert_combobox_dropdown_menu_overlay_placement_matches("combobox-dropdown-menu.vp375x240");
}
#[test]
fn web_vs_fret_combobox_dropdown_menu_menu_item_height_matches() {
    assert_combobox_dropdown_menu_constrained_menu_item_height_matches("combobox-dropdown-menu");
}
#[test]
fn web_vs_fret_combobox_dropdown_menu_menu_item_height_matches_mobile_tiny_viewport() {
    assert_combobox_dropdown_menu_constrained_menu_item_height_matches(
        "combobox-dropdown-menu.vp375x240",
    );
}
#[test]
fn web_vs_fret_combobox_dropdown_menu_menu_content_insets_match() {
    assert_combobox_dropdown_menu_constrained_menu_content_insets_match("combobox-dropdown-menu");
}
#[test]
fn web_vs_fret_combobox_dropdown_menu_menu_content_insets_match_mobile_tiny_viewport() {
    assert_combobox_dropdown_menu_constrained_menu_content_insets_match(
        "combobox-dropdown-menu.vp375x240",
    );
}
#[test]
fn web_vs_fret_breadcrumb_dropdown_menu_item_height_matches() {
    assert_breadcrumb_dropdown_menu_item_height_matches("breadcrumb-dropdown");
}
#[test]
fn web_vs_fret_breadcrumb_dropdown_menu_item_height_matches_mobile_tiny_viewport() {
    assert_breadcrumb_dropdown_menu_item_height_matches("breadcrumb-dropdown.vp375x240");
}
#[test]
fn web_vs_fret_breadcrumb_dropdown_menu_content_insets_match() {
    assert_breadcrumb_dropdown_menu_content_insets_match("breadcrumb-dropdown");
}
#[test]
fn web_vs_fret_breadcrumb_dropdown_menu_content_insets_match_mobile_tiny_viewport() {
    assert_breadcrumb_dropdown_menu_content_insets_match("breadcrumb-dropdown.vp375x240");
}
