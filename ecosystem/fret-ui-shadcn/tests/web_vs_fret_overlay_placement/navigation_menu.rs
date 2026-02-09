use super::*;

#[test]
fn fret_navigation_menu_tracks_trigger_when_underlay_scrolls() {
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

    let open_value = "components";
    let model: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let root_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
    let scroll_handle = ScrollHandle::default();

    let render = |cx: &mut ElementContext<'_, App>| {
        let scroll_handle = scroll_handle.clone();
        let root_id_out = root_id_out.clone();
        let model = model.clone();

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
                        let nav = cx.container(
                            ContainerProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.position = fret_ui::element::PositionStyle::Absolute;
                                    layout.inset.left = Some(Px(16.0));
                                    layout.inset.top = Some(Px(160.0));
                                    layout.size.width = Length::Px(Px(360.0));
                                    layout.size.height = Length::Px(Px(32.0));
                                    layout
                                },
                                ..Default::default()
                            },
                            move |cx| {
                                let items = vec![
                                    fret_ui_shadcn::NavigationMenuItem::new(
                                        "home",
                                        "Home",
                                        vec![cx.text("Home")],
                                    ),
                                    fret_ui_shadcn::NavigationMenuItem::new(
                                        "components",
                                        "Components",
                                        vec![cx.text("Components Panel")],
                                    ),
                                ];
                                let el = fret_ui_shadcn::NavigationMenu::new(model.clone())
                                    .viewport(false)
                                    .indicator(false)
                                    .items(items)
                                    .into_element(cx);
                                root_id_out.set(Some(el.id));
                                vec![el]
                            },
                        );

                        vec![nav]
                    },
                )]
            },
        )]
    };

    // Frame 1: mount closed.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        render,
    );

    let _ = app
        .models_mut()
        .update(&model, |v| *v = Some(Arc::from(open_value)));

    // Frame 2+: open and settle motion.
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            false,
            render,
        );
    }

    // Paint once so last-frame visual bounds caches are populated (used by anchored overlay placement).
    let mut scene = fret_core::Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    ui.ingest_paint_cache_source(&mut scene);
    scene.clear();
    let _ = app.flush_effects();

    let root_id = root_id_out.get().expect("navigation menu root id");
    let trigger_id = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "nav-menu-underlay-scroll-trigger-query",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_trigger_id(
                cx, root_id, open_value,
            )
        },
    )
    .unwrap_or_else(|| panic!("fret navigation-menu trigger id for {open_value}"));
    let content_id = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "nav-menu-underlay-scroll-content-query",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_viewport_content_id(
                cx, root_id, open_value,
            )
        },
    )
    .unwrap_or_else(|| panic!("fret navigation-menu content id for {open_value}"));

    let trigger_node = fret_ui::elements::node_for_element(&mut app, window, trigger_id)
        .expect("nav menu trigger node");
    let content_node = fret_ui::elements::node_for_element(&mut app, window, content_id)
        .expect("nav menu content node");

    let trigger_before = ui
        .debug_node_visual_bounds(trigger_node)
        .expect("nav menu trigger visual bounds");
    let content_before = ui
        .debug_node_visual_bounds(content_node)
        .expect("nav menu content visual bounds");

    let dx_before = content_before.origin.x.0 - trigger_before.origin.x.0;
    let dy_before = content_before.origin.y.0 - trigger_before.origin.y.0;

    // Scroll the underlay (wheel over the scroll viewport, not over the navigation menu viewport).
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

    // Frame N: apply the scroll and paint so visual bounds caches update.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2 + settle_frames),
        false,
        render,
    );
    let mut scene = fret_core::Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    ui.ingest_paint_cache_source(&mut scene);
    scene.clear();

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

    // Frame N+1: expected to re-anchor the open content to the scrolled trigger.
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
    ui.ingest_paint_cache_source(&mut scene);
    scene.clear();
    let _ = app.flush_effects();

    let content_id_after = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "nav-menu-underlay-scroll-content-query-after",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_viewport_content_id(
                cx, root_id, open_value,
            )
        },
    )
    .unwrap_or_else(|| panic!("fret navigation-menu content id for {open_value} (after scroll)"));

    let trigger_after_node = fret_ui::elements::node_for_element(&mut app, window, trigger_id)
        .expect("nav menu trigger node (after scroll)");
    let content_after_node =
        fret_ui::elements::node_for_element(&mut app, window, content_id_after)
            .expect("nav menu content node (after scroll)");

    let trigger_after = ui
        .debug_node_visual_bounds(trigger_after_node)
        .expect("nav menu trigger visual bounds (after scroll)");
    let content_after = ui
        .debug_node_visual_bounds(content_after_node)
        .expect("nav menu content visual bounds (after scroll)");

    assert!(
        (trigger_after.origin.y.0 - trigger_before.origin.y.0).abs() > 1.0,
        "expected trigger to move under scroll (before_y={} after_y={})",
        trigger_before.origin.y.0,
        trigger_after.origin.y.0
    );

    let dx_after = content_after.origin.x.0 - trigger_after.origin.x.0;
    let dy_after = content_after.origin.y.0 - trigger_after.origin.y.0;

    assert_close(
        "navigation menu anchor dx stable under scroll",
        dx_after,
        dx_before,
        1.0,
    );
    assert_close(
        "navigation menu anchor dy stable under scroll",
        dy_after,
        dy_before,
        1.0,
    );
}
#[test]
fn fret_navigation_menu_tracks_trigger_when_underlay_scrolls_via_wheel_over_overlay() {
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

    let open_value = "components";
    let model: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let root_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
    let scroll_handle = ScrollHandle::default();

    let render = |cx: &mut ElementContext<'_, App>| {
        let scroll_handle = scroll_handle.clone();
        let root_id_out = root_id_out.clone();
        let model = model.clone();

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
                        let nav = cx.container(
                            ContainerProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.position = fret_ui::element::PositionStyle::Absolute;
                                    layout.inset.left = Some(Px(16.0));
                                    layout.inset.top = Some(Px(160.0));
                                    layout.size.width = Length::Px(Px(360.0));
                                    layout.size.height = Length::Px(Px(32.0));
                                    layout
                                },
                                ..Default::default()
                            },
                            move |cx| {
                                let items = vec![
                                    fret_ui_shadcn::NavigationMenuItem::new(
                                        "home",
                                        "Home",
                                        vec![cx.text("Home")],
                                    ),
                                    fret_ui_shadcn::NavigationMenuItem::new(
                                        "components",
                                        "Components",
                                        vec![cx.text("Components Panel")],
                                    ),
                                ];
                                let el = fret_ui_shadcn::NavigationMenu::new(model.clone())
                                    .viewport(false)
                                    .indicator(false)
                                    .items(items)
                                    .into_element(cx);
                                root_id_out.set(Some(el.id));
                                vec![el]
                            },
                        );

                        vec![nav]
                    },
                )]
            },
        )]
    };

    // Frame 1: mount closed.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        render,
    );

    let _ = app
        .models_mut()
        .update(&model, |v| *v = Some(Arc::from(open_value)));

    // Frame 2+: open and settle motion.
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            false,
            render,
        );
    }

    // Paint once so last-frame visual bounds caches are populated (used by anchored overlay placement).
    let mut scene = fret_core::Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    ui.ingest_paint_cache_source(&mut scene);
    scene.clear();
    let _ = app.flush_effects();

    let root_id = root_id_out.get().expect("navigation menu root id");
    let trigger_id = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "nav-menu-underlay-scroll-trigger-query",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_trigger_id(
                cx, root_id, open_value,
            )
        },
    )
    .unwrap_or_else(|| panic!("fret navigation-menu trigger id for {open_value}"));
    let content_id_before = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "nav-menu-underlay-scroll-content-query-before",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_viewport_content_id(
                cx, root_id, open_value,
            )
        },
    )
    .unwrap_or_else(|| panic!("fret navigation-menu content id for {open_value}"));

    let trigger_before_node = fret_ui::elements::node_for_element(&mut app, window, trigger_id)
        .expect("nav menu trigger node (before scroll)");
    let content_before_node =
        fret_ui::elements::node_for_element(&mut app, window, content_id_before)
            .expect("nav menu content node (before scroll)");

    let trigger_before = ui
        .debug_node_visual_bounds(trigger_before_node)
        .expect("nav menu trigger visual bounds (before scroll)");
    let content_before = ui
        .debug_node_visual_bounds(content_before_node)
        .expect("nav menu content visual bounds (before scroll)");

    let dx_before = content_before.origin.x.0 - trigger_before.origin.x.0;
    let dy_before = content_before.origin.y.0 - trigger_before.origin.y.0;

    // Wheel over the overlay content. NavigationMenu content has no internal scroll range, so the
    // wheel should not cause the overlay to re-anchor/jitter.
    let overlay_wheel_point = Point::new(
        Px(content_before.origin.x.0 + content_before.size.width.0 * 0.5),
        Px(content_before.origin.y.0 + content_before.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Wheel {
            pointer_id: fret_core::PointerId::default(),
            position: overlay_wheel_point,
            delta: Point::new(Px(0.0), Px(-80.0)),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
    assert!(
        scroll_handle.offset().y.0 <= 0.01,
        "expected wheel over non-scrollable overlay to not scroll the underlay; y={}",
        scroll_handle.offset().y.0
    );

    // Frame N: settle any hover/wheel side-effects.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2 + settle_frames),
        false,
        render,
    );
    let mut scene = fret_core::Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    ui.ingest_paint_cache_source(&mut scene);
    scene.clear();
    let _ = app.flush_effects();

    let trigger_after_node = fret_ui::elements::node_for_element(&mut app, window, trigger_id)
        .expect("nav menu trigger node (after wheel)");
    let content_after_node =
        fret_ui::elements::node_for_element(&mut app, window, content_id_before)
            .expect("nav menu content node (after wheel)");

    let trigger_after = ui
        .debug_node_visual_bounds(trigger_after_node)
        .expect("nav menu trigger visual bounds (after wheel)");
    let content_after = ui
        .debug_node_visual_bounds(content_after_node)
        .expect("nav menu content visual bounds (after wheel)");

    assert_close(
        "navigation menu trigger x stable under wheel over overlay",
        trigger_after.origin.x.0,
        trigger_before.origin.x.0,
        1.0,
    );
    assert_close(
        "navigation menu trigger y stable under wheel over overlay",
        trigger_after.origin.y.0,
        trigger_before.origin.y.0,
        1.0,
    );
    assert_close(
        "navigation menu content x stable under wheel over overlay",
        content_after.origin.x.0,
        content_before.origin.x.0,
        1.0,
    );
    assert_close(
        "navigation menu content y stable under wheel over overlay",
        content_after.origin.y.0,
        content_before.origin.y.0,
        1.0,
    );

    let dx_after = content_after.origin.x.0 - trigger_after.origin.x.0;
    let dy_after = content_after.origin.y.0 - trigger_after.origin.y.0;

    assert_close(
        "navigation menu anchor dx stable under wheel over overlay",
        dx_after,
        dx_before,
        1.0,
    );
    assert_close(
        "navigation menu anchor dy stable under wheel over overlay",
        dy_after,
        dy_before,
        1.0,
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_overlay_placement_matches() {
    let web = read_web_golden_open("navigation-menu-demo");
    let theme = web_theme(&web);

    let web_trigger =
        web_find_by_data_slot_and_state(&theme.root, "navigation-menu-trigger", "open")
            .unwrap_or_else(|| {
                find_first(&theme.root, &|n| {
                    n.attrs
                        .get("data-slot")
                        .is_some_and(|v| v.as_str() == "navigation-menu-trigger")
                })
                .expect("web trigger slot=navigation-menu-trigger")
            });
    let web_content =
        web_find_by_data_slot_and_state(&theme.root, "navigation-menu-content", "open")
            .expect("web content slot=navigation-menu-content state=open");

    let web_side = infer_side(web_trigger.rect, web_content.rect);
    let web_align = infer_align(web_side, web_trigger.rect, web_content.rect);
    let expected_gap = rect_main_gap(web_side, web_trigger.rect, web_content.rect);
    let expected_cross = rect_cross_delta(web_side, web_align, web_trigger.rect, web_content.rect);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let model: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let root_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let el = fret_ui_shadcn::NavigationMenu::new(model.clone())
                .viewport(false)
                .indicator(false)
                .items(vec![fret_ui_shadcn::NavigationMenuItem::new(
                    "home",
                    "Home",
                    vec![shadcn_nav_menu_demo_home_desktop_panel(cx, model.clone())],
                )])
                .into_element(cx);
            root_id_out.set(Some(el.id));
            vec![pad_root(cx, Px(0.0), el)]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Home"))
        .expect("fret trigger semantics (Home)");
    let click_point = Point::new(
        Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
        Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
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
                let el = fret_ui_shadcn::NavigationMenu::new(model.clone())
                    .viewport(false)
                    .indicator(false)
                    .items(vec![fret_ui_shadcn::NavigationMenuItem::new(
                        "home",
                        "Home",
                        vec![shadcn_nav_menu_demo_home_desktop_panel(cx, model.clone())],
                    )])
                    .into_element(cx);
                root_id_out.set(Some(el.id));
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let root_id = root_id_out.get().expect("navigation menu root id");
    let content_id = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-query",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_viewport_content_id(
                cx, root_id, "home",
            )
        },
    )
    .expect("fret nav menu content id");
    let content_bounds =
        bounds_for_element(&mut app, window, content_id).expect("fret nav menu content bounds");

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Home"))
        .expect("fret trigger semantics (Home)");

    let fret_trigger = WebRect {
        x: trigger.bounds.origin.x.0,
        y: trigger.bounds.origin.y.0,
        w: trigger.bounds.size.width.0,
        h: trigger.bounds.size.height.0,
    };
    let fret_content = WebRect {
        x: content_bounds.origin.x.0,
        y: content_bounds.origin.y.0,
        w: content_bounds.size.width.0,
        h: content_bounds.size.height.0,
    };

    let actual_gap = rect_main_gap(web_side, fret_trigger, fret_content);
    let actual_cross = rect_cross_delta(web_side, web_align, fret_trigger, fret_content);

    assert_close(
        "navigation-menu-demo main_gap",
        actual_gap,
        expected_gap,
        1.0,
    );
    assert_close(
        "navigation-menu-demo cross_delta",
        actual_cross,
        expected_cross,
        1.5,
    );
    assert_close(
        "navigation-menu-demo content_width",
        fret_content.w,
        web_content.rect.w,
        2.0,
    );
    assert_close(
        "navigation-menu-demo content_height",
        fret_content.h,
        web_content.rect.h,
        2.0,
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_components_overlay_placement_matches() {
    web_vs_fret_navigation_menu_demo_variant_overlay_placement_matches(
        "navigation-menu-demo.components",
        "components",
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_components_small_viewport_overlay_placement_matches() {
    web_vs_fret_navigation_menu_demo_variant_overlay_placement_matches(
        "navigation-menu-demo.components-vp1440x320",
        "components",
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_list_overlay_placement_matches() {
    web_vs_fret_navigation_menu_demo_variant_overlay_placement_matches(
        "navigation-menu-demo.list",
        "list",
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_list_small_viewport_overlay_placement_matches() {
    web_vs_fret_navigation_menu_demo_variant_overlay_placement_matches(
        "navigation-menu-demo.list-vp1440x320",
        "list",
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_simple_small_viewport_overlay_placement_matches() {
    web_vs_fret_navigation_menu_demo_variant_overlay_placement_matches(
        "navigation-menu-demo.simple-vp1440x320",
        "simple",
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_simple_overlay_placement_matches() {
    web_vs_fret_navigation_menu_demo_variant_overlay_placement_matches(
        "navigation-menu-demo.simple",
        "simple",
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_with_icon_small_viewport_overlay_placement_matches() {
    web_vs_fret_navigation_menu_demo_variant_overlay_placement_matches(
        "navigation-menu-demo.with-icon-vp1440x320",
        "with-icon",
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_with_icon_overlay_placement_matches() {
    web_vs_fret_navigation_menu_demo_variant_overlay_placement_matches(
        "navigation-menu-demo.with-icon",
        "with-icon",
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_hover_home_to_components_overlay_placement_matches() {
    web_vs_fret_navigation_menu_demo_hover_switch_overlay_placement_matches(
        "navigation-menu-demo.home-then-hover-components",
        "home",
        "components",
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_indicator_geometry_matches_web() {
    let web = read_web_golden_open("navigation-menu-demo-indicator");
    let theme = web_theme(&web);

    let web_trigger =
        web_find_by_data_slot_and_state(&theme.root, "navigation-menu-trigger", "open")
            .expect("web trigger slot=navigation-menu-trigger state=open");
    let web_indicator =
        web_find_by_data_slot_and_state(&theme.root, "navigation-menu-indicator", "visible")
            .expect("web indicator slot=navigation-menu-indicator state=visible");
    let web_viewport =
        web_find_by_data_slot_and_state(&theme.root, "navigation-menu-viewport", "open")
            .expect("web viewport slot=navigation-menu-viewport state=open");

    let web_diamond = web_indicator
        .children
        .iter()
        .find(|n| web_css_px(n, "width").is_some_and(|v| (v - 8.0).abs() <= 0.01))
        .unwrap_or_else(|| panic!("missing web navigation-menu indicator diamond node"));
    let web_diamond_unrotated = web_unrotated_rect_for_rotated_square(web_diamond);

    let expected_track = web_indicator.rect;

    let expected_diamond = WebRect {
        x: web_diamond_unrotated.x,
        y: web_diamond_unrotated.y,
        w: web_diamond_unrotated.w,
        h: web_diamond_unrotated.h,
    };

    assert_close(
        "navigation-menu-demo-indicator web trigger_x == indicator_x",
        web_trigger.rect.x,
        expected_track.x,
        0.5,
    );
    assert_close(
        "navigation-menu-demo-indicator web trigger_w ~= indicator_w",
        web_trigger.rect.w,
        expected_track.w,
        1.0,
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::from_web_theme(&theme);

    let bounds = bounds_for_web_theme(&theme);

    let model: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("home")));
    let root_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for frame in 1..=(1 + settle_frames) {
        let request_semantics = frame == 1 + settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame as u64),
            request_semantics,
            |cx| {
                let items = vec![fret_ui_shadcn::NavigationMenuItem::new(
                    "home",
                    "Home",
                    vec![shadcn_nav_menu_demo_indicator_panel(cx)],
                )];

                let el = fret_ui_shadcn::NavigationMenu::new(model.clone())
                    .viewport(true)
                    .indicator(true)
                    .items(items)
                    .into_element(cx);
                root_id_out.set(Some(el.id));
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let root_id = root_id_out.get().expect("navigation menu root id");
    let viewport_id = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-indicator-viewport-id",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_viewport_panel_id(cx, root_id)
        },
    )
    .expect("fret nav menu viewport panel id");
    let viewport_bounds =
        bounds_for_element(&mut app, window, viewport_id).expect("fret nav menu viewport bounds");

    let indicator_track_id = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-indicator-track-id",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_indicator_track_id(
                cx, root_id,
            )
        },
    )
    .expect("fret nav menu indicator track id");
    let indicator_track_bounds = bounds_for_element(&mut app, window, indicator_track_id)
        .expect("fret nav menu indicator track bounds");

    let indicator_diamond_id = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-indicator-diamond-id",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_indicator_diamond_id(
                cx, root_id,
            )
        },
    )
    .expect("fret nav menu indicator diamond id");
    let indicator_diamond_bounds = bounds_for_element(&mut app, window, indicator_diamond_id)
        .expect("fret nav menu indicator diamond bounds");

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Home"))
        .expect("fret trigger semantics (Home)");

    assert_close(
        "navigation-menu-demo-indicator track_x",
        indicator_track_bounds.origin.x.0,
        expected_track.x,
        1.0,
    );
    assert_close(
        "navigation-menu-demo-indicator track_y",
        indicator_track_bounds.origin.y.0,
        expected_track.y,
        1.0,
    );
    assert_close(
        "navigation-menu-demo-indicator track_w",
        indicator_track_bounds.size.width.0,
        expected_track.w,
        1.5,
    );
    assert_close(
        "navigation-menu-demo-indicator track_h",
        indicator_track_bounds.size.height.0,
        expected_track.h,
        0.5,
    );

    assert_close(
        "navigation-menu-demo-indicator trigger_x == track_x",
        trigger.bounds.origin.x.0,
        indicator_track_bounds.origin.x.0,
        1.0,
    );
    assert_close(
        "navigation-menu-demo-indicator trigger_w == track_w",
        trigger.bounds.size.width.0,
        indicator_track_bounds.size.width.0,
        1.5,
    );

    let web_gap_to_viewport = web_viewport.rect.y - (expected_track.y + expected_track.h);
    let fret_gap_to_viewport = viewport_bounds.origin.y.0
        - (indicator_track_bounds.origin.y.0 + indicator_track_bounds.size.height.0);
    assert_close(
        "navigation-menu-demo-indicator gap_to_viewport",
        fret_gap_to_viewport,
        web_gap_to_viewport,
        1.0,
    );
    assert_close(
        "navigation-menu-demo-indicator viewport_w",
        viewport_bounds.size.width.0,
        web_viewport.rect.w,
        2.0,
    );
    assert_close(
        "navigation-menu-demo-indicator viewport_h",
        viewport_bounds.size.height.0,
        web_viewport.rect.h,
        2.0,
    );

    let actual_diamond_left =
        indicator_diamond_bounds.origin.x.0 - indicator_track_bounds.origin.x.0;
    let actual_diamond_top =
        indicator_diamond_bounds.origin.y.0 - indicator_track_bounds.origin.y.0;
    assert_close(
        "navigation-menu-demo-indicator diamond_left",
        actual_diamond_left,
        expected_diamond.x - expected_track.x,
        1.5,
    );
    assert_close(
        "navigation-menu-demo-indicator diamond_top",
        actual_diamond_top,
        expected_diamond.y - expected_track.y,
        1.5,
    );
    assert_close(
        "navigation-menu-demo-indicator diamond_w",
        indicator_diamond_bounds.size.width.0,
        expected_diamond.w,
        0.5,
    );
    assert_close(
        "navigation-menu-demo-indicator diamond_h",
        indicator_diamond_bounds.size.height.0,
        expected_diamond.h,
        0.5,
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_home_mobile_viewport_height_matches() {
    let web = read_web_golden_open("navigation-menu-demo.home-mobile");
    let theme = web_theme(&web);

    let web_trigger =
        web_find_by_data_slot_and_state(&theme.root, "navigation-menu-trigger", "open")
            .expect("web trigger slot=navigation-menu-trigger state=open");
    let web_viewport =
        web_find_by_data_slot_and_state(&theme.root, "navigation-menu-viewport", "open")
            .expect("web viewport slot=navigation-menu-viewport state=open");

    let web_side = infer_side(web_trigger.rect, web_viewport.rect);
    let web_align = infer_align(web_side, web_trigger.rect, web_viewport.rect);
    let expected_gap = rect_main_gap(web_side, web_trigger.rect, web_viewport.rect);
    let expected_cross = rect_cross_delta(web_side, web_align, web_trigger.rect, web_viewport.rect);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::from_web_theme(&theme);

    let bounds = bounds_for_web_theme(&theme);

    let model: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let root_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let items = vec![
                fret_ui_shadcn::NavigationMenuItem::new(
                    "home",
                    "Home",
                    vec![shadcn_nav_menu_demo_home_panel(cx, model.clone())],
                ),
                fret_ui_shadcn::NavigationMenuItem::new(
                    "components",
                    "Components",
                    vec![shadcn_nav_menu_demo_components_mobile_panel(
                        cx,
                        model.clone(),
                    )],
                ),
                fret_ui_shadcn::NavigationMenuItem::new("docs", "Docs", Vec::new()),
            ];

            let el = fret_ui_shadcn::NavigationMenu::new(model.clone())
                .viewport(true)
                .indicator(false)
                .items(items)
                .into_element(cx);
            root_id_out.set(Some(el.id));
            vec![pad_root(cx, Px(0.0), el)]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Home"))
        .expect("fret trigger semantics (Home)");
    let click_point = Point::new(
        Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
        Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
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
                let items = vec![
                    fret_ui_shadcn::NavigationMenuItem::new(
                        "home",
                        "Home",
                        vec![shadcn_nav_menu_demo_home_panel(cx, model.clone())],
                    ),
                    fret_ui_shadcn::NavigationMenuItem::new(
                        "components",
                        "Components",
                        vec![shadcn_nav_menu_demo_components_mobile_panel(
                            cx,
                            model.clone(),
                        )],
                    ),
                    fret_ui_shadcn::NavigationMenuItem::new("docs", "Docs", Vec::new()),
                ];

                let el = fret_ui_shadcn::NavigationMenu::new(model.clone())
                    .viewport(true)
                    .indicator(false)
                    .items(items)
                    .into_element(cx);
                root_id_out.set(Some(el.id));
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let root_id = root_id_out.get().expect("navigation menu root id");
    let viewport_id = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-viewport-query",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_viewport_panel_id(cx, root_id)
        },
    )
    .expect("fret nav menu viewport panel id");
    let viewport_bounds =
        bounds_for_element(&mut app, window, viewport_id).expect("fret nav menu viewport bounds");

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Home"))
        .expect("fret trigger semantics (Home)");

    let fret_trigger = WebRect {
        x: trigger.bounds.origin.x.0,
        y: trigger.bounds.origin.y.0,
        w: trigger.bounds.size.width.0,
        h: trigger.bounds.size.height.0,
    };
    let fret_viewport = WebRect {
        x: viewport_bounds.origin.x.0,
        y: viewport_bounds.origin.y.0,
        w: viewport_bounds.size.width.0,
        h: viewport_bounds.size.height.0,
    };

    let debug = std::env::var("FRET_DEBUG_NAV_MENU_MOBILE")
        .ok()
        .is_some_and(|v| v == "1");
    if debug {
        eprintln!(
            "nav-menu home-mobile web viewport={:?} web trigger={:?} fret viewport={:?} fret trigger={:?}",
            web_viewport.rect, web_trigger.rect, fret_viewport, fret_trigger
        );
    }

    let actual_gap = rect_main_gap(web_side, fret_trigger, fret_viewport);
    let actual_cross = rect_cross_delta(web_side, web_align, fret_trigger, fret_viewport);

    assert_close(
        "navigation-menu-demo.home-mobile main_gap",
        actual_gap,
        expected_gap,
        1.0,
    );
    assert_close(
        "navigation-menu-demo.home-mobile cross_delta",
        actual_cross,
        expected_cross,
        1.5,
    );
    assert_close(
        "navigation-menu-demo.home-mobile viewport_height",
        fret_viewport.h,
        web_viewport.rect.h,
        1.5,
    );
    assert_close(
        "navigation-menu-demo.home-mobile viewport_width",
        fret_viewport.w,
        web_viewport.rect.w,
        1.5,
    );
    assert_close(
        "navigation-menu-demo.home-mobile trigger_height",
        fret_trigger.h,
        web_trigger.rect.h,
        1.0,
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_home_mobile_viewport_insets_match() {
    assert_navigation_menu_demo_mobile_viewport_insets_match(
        "navigation-menu-demo.home-mobile",
        "home",
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_home_mobile_small_viewport_height_matches() {
    assert_navigation_menu_demo_mobile_viewport_geometry_matches(
        "navigation-menu-demo.home-mobile-vp375x320",
        "home",
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_home_mobile_small_viewport_insets_match() {
    assert_navigation_menu_demo_mobile_viewport_insets_match(
        "navigation-menu-demo.home-mobile-vp375x320",
        "home",
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_home_mobile_tiny_viewport_height_matches() {
    assert_navigation_menu_demo_mobile_viewport_geometry_matches(
        "navigation-menu-demo.home-mobile-vp375x240",
        "home",
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_home_mobile_tiny_viewport_insets_match() {
    assert_navigation_menu_demo_mobile_viewport_insets_match(
        "navigation-menu-demo.home-mobile-vp375x240",
        "home",
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_components_mobile_viewport_height_matches() {
    assert_navigation_menu_demo_mobile_viewport_geometry_matches(
        "navigation-menu-demo.components-mobile",
        "components",
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_components_mobile_small_viewport_height_matches() {
    assert_navigation_menu_demo_mobile_viewport_geometry_matches(
        "navigation-menu-demo.components-mobile-vp375x320",
        "components",
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_components_mobile_tiny_viewport_height_matches() {
    assert_navigation_menu_demo_mobile_viewport_geometry_matches(
        "navigation-menu-demo.components-mobile-vp375x240",
        "components",
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_components_mobile_viewport_insets_match() {
    assert_navigation_menu_demo_mobile_viewport_insets_match(
        "navigation-menu-demo.components-mobile",
        "components",
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_components_mobile_small_viewport_insets_match() {
    assert_navigation_menu_demo_mobile_viewport_insets_match(
        "navigation-menu-demo.components-mobile-vp375x320",
        "components",
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_components_mobile_tiny_viewport_insets_match() {
    assert_navigation_menu_demo_mobile_viewport_insets_match(
        "navigation-menu-demo.components-mobile-vp375x240",
        "components",
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_home_mobile_hover_to_components_viewport_geometry_matches() {
    assert_navigation_menu_demo_mobile_viewport_geometry_after_hover_matches(
        "navigation-menu-demo.home-mobile-then-hover-components",
        "home",
        "components",
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_home_mobile_small_viewport_hover_to_components_viewport_geometry_matches()
 {
    assert_navigation_menu_demo_mobile_viewport_geometry_after_hover_matches(
        "navigation-menu-demo.home-mobile-vp375x320-then-hover-components",
        "home",
        "components",
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_home_mobile_tiny_viewport_hover_to_components_viewport_geometry_matches()
 {
    assert_navigation_menu_demo_mobile_viewport_geometry_after_hover_matches(
        "navigation-menu-demo.home-mobile-vp375x240-then-hover-components",
        "home",
        "components",
    );
}
