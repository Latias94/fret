use super::*;

#[derive(Debug, Clone, Deserialize)]
struct FixtureSuite<T> {
    schema_version: u32,
    cases: Vec<T>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum LayoutScrollRecipe {
    ScrollAreaDemoRootSize,
    ScrollAreaDemoMaxOffsetY,
    ScrollAreaHorizontalDemoMaxOffset,
    ScrollAreaDemoScrollbarBoundsHover,
    ScrollAreaDemoThumbBackgroundHoverLight,
    ScrollAreaDemoThumbBackgroundHoverDark,
    ScrollAreaDemoScrollbarHidesAfterHoverOutDelay,
    ScrollAreaDemoThumbBoundsScrolled,
    ScrollAreaHorizontalDemoScrollbarBoundsHover,
    ScrollAreaHorizontalDemoThumbBackgroundHoverLight,
    ScrollAreaHorizontalDemoThumbBackgroundHoverDark,
    ScrollAreaHorizontalDemoScrollbarHidesAfterHoverOutDelay,
    ScrollAreaHorizontalDemoThumbBoundsScrolled,
}

#[derive(Debug, Clone, Deserialize)]
struct LayoutScrollCase {
    id: String,
    web_name: String,
    #[serde(default)]
    web_name_late: Option<String>,
    recipe: LayoutScrollRecipe,
}

#[test]
fn web_vs_fret_layout_scroll_geometry_matches_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/layout_scroll_cases_v1.json"
    ));
    let suite: FixtureSuite<LayoutScrollCase> =
        serde_json::from_str(raw).expect("layout scroll fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("layout scroll case={}", case.id);
        match case.recipe {
            LayoutScrollRecipe::ScrollAreaDemoRootSize => {
                assert_eq!(case.web_name, "scroll-area-demo");
                assert!(case.web_name_late.is_none());
                web_vs_fret_layout_scroll_area_demo_root_size();
            }
            LayoutScrollRecipe::ScrollAreaDemoMaxOffsetY => {
                assert_eq!(case.web_name, "scroll-area-demo");
                assert!(case.web_name_late.is_none());
                web_vs_fret_layout_scroll_area_demo_max_offset_y_matches_web();
            }
            LayoutScrollRecipe::ScrollAreaHorizontalDemoMaxOffset => {
                assert_eq!(case.web_name, "scroll-area-horizontal-demo");
                assert!(case.web_name_late.is_none());
                web_vs_fret_layout_scroll_area_horizontal_demo_max_offset_matches_web();
            }
            LayoutScrollRecipe::ScrollAreaDemoScrollbarBoundsHover => {
                assert_eq!(case.web_name, "scroll-area-demo.hover");
                assert!(case.web_name_late.is_none());
                web_vs_fret_layout_scroll_area_demo_scrollbar_bounds_match_web_hover();
            }
            LayoutScrollRecipe::ScrollAreaDemoThumbBackgroundHoverLight => {
                assert_eq!(case.web_name, "scroll-area-demo.hover");
                assert!(case.web_name_late.is_none());
                web_vs_fret_layout_scroll_area_demo_thumb_background_matches_web_hover_light();
            }
            LayoutScrollRecipe::ScrollAreaDemoThumbBackgroundHoverDark => {
                assert_eq!(case.web_name, "scroll-area-demo.hover");
                assert!(case.web_name_late.is_none());
                web_vs_fret_layout_scroll_area_demo_thumb_background_matches_web_hover_dark();
            }
            LayoutScrollRecipe::ScrollAreaDemoScrollbarHidesAfterHoverOutDelay => {
                assert_eq!(case.web_name, "scroll-area-demo.hover-out-550ms");
                assert_eq!(
                    case.web_name_late.as_deref(),
                    Some("scroll-area-demo.hover-out-650ms")
                );
                web_vs_fret_layout_scroll_area_demo_scrollbar_hides_after_hover_out_delay();
            }
            LayoutScrollRecipe::ScrollAreaDemoThumbBoundsScrolled => {
                assert_eq!(case.web_name, "scroll-area-demo.scrolled");
                assert!(case.web_name_late.is_none());
                web_vs_fret_layout_scroll_area_demo_thumb_bounds_match_web_scrolled();
            }
            LayoutScrollRecipe::ScrollAreaHorizontalDemoScrollbarBoundsHover => {
                assert_eq!(case.web_name, "scroll-area-horizontal-demo.hover");
                assert!(case.web_name_late.is_none());
                web_vs_fret_layout_scroll_area_horizontal_demo_scrollbar_bounds_match_web_hover();
            }
            LayoutScrollRecipe::ScrollAreaHorizontalDemoThumbBackgroundHoverLight => {
                assert_eq!(case.web_name, "scroll-area-horizontal-demo.hover");
                assert!(case.web_name_late.is_none());
                web_vs_fret_layout_scroll_area_horizontal_demo_thumb_background_matches_web_hover_light(
                );
            }
            LayoutScrollRecipe::ScrollAreaHorizontalDemoThumbBackgroundHoverDark => {
                assert_eq!(case.web_name, "scroll-area-horizontal-demo.hover");
                assert!(case.web_name_late.is_none());
                web_vs_fret_layout_scroll_area_horizontal_demo_thumb_background_matches_web_hover_dark(
                );
            }
            LayoutScrollRecipe::ScrollAreaHorizontalDemoScrollbarHidesAfterHoverOutDelay => {
                assert_eq!(case.web_name, "scroll-area-horizontal-demo.hover-out-550ms");
                assert_eq!(
                    case.web_name_late.as_deref(),
                    Some("scroll-area-horizontal-demo.hover-out-650ms")
                );
                web_vs_fret_layout_scroll_area_horizontal_demo_scrollbar_hides_after_hover_out_delay(
                );
            }
            LayoutScrollRecipe::ScrollAreaHorizontalDemoThumbBoundsScrolled => {
                assert_eq!(case.web_name, "scroll-area-horizontal-demo.scrolled");
                assert!(case.web_name_late.is_none());
                web_vs_fret_layout_scroll_area_horizontal_demo_thumb_bounds_match_web_scrolled();
            }
        }
    }
}

fn web_vs_fret_layout_scroll_area_demo_root_size() {
    let web = read_web_golden("scroll-area-demo");
    let theme = web_theme(&web);
    let web_root = web_find_by_class_tokens(
        &theme.root,
        &["relative", "h-72", "w-48", "rounded-md", "border"],
    )
    .expect("web scroll area root");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let items: Vec<_> = (1..=50).map(|i| cx.text(format!("Item {i}"))).collect();

        let scroll_area = fret_ui_shadcn::ScrollArea::new(items)
            .refine_layout(
                fret_ui_kit::LayoutRefinement::default()
                    .w_px(Px(web_root.rect.w))
                    .h_px(Px(web_root.rect.h)),
            )
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:scroll-area-demo:root")),
                ..Default::default()
            },
            move |_cx| vec![scroll_area],
        )]
    });

    let root = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-demo:root"),
    )
    .expect("fret scroll area root");

    assert_close_px(
        "scroll area root width",
        root.bounds.size.width,
        web_root.rect.w,
        1.0,
    );
    assert_close_px(
        "scroll area root height",
        root.bounds.size.height,
        web_root.rect.h,
        1.0,
    );
}

fn web_vs_fret_layout_scroll_area_demo_max_offset_y_matches_web() {
    let web = read_web_golden("scroll-area-demo");
    let theme = web_theme(&web);
    let web_root = web_find_by_class_tokens(
        &theme.root,
        &["relative", "h-72", "w-48", "rounded-md", "border"],
    )
    .expect("web scroll area root");

    let web_viewport = find_first(web_root, &|n| {
        n.computed_style
            .get("overflowY")
            .is_some_and(|v| v == "scroll")
    })
    .expect("web scroll viewport (overflowY=scroll)");

    let metrics = web_viewport
        .scroll
        .as_ref()
        .expect("web scroll viewport missing scroll metrics (regenerate goldens)");

    let expected_max_offset_y = metrics.scroll_height - metrics.client_height;
    let inset = web_viewport.rect.x - web_root.rect.x;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let handle = ScrollHandle::default();
    let _ = run_fret_root(bounds, |cx| {
        let content = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Px(Px(metrics.scroll_height));
                    layout
                },
                ..Default::default()
            },
            |_cx| vec![],
        );

        let scroll_area = fret_ui_shadcn::ScrollArea::new(vec![content])
            .scroll_handle(handle.clone())
            .refine_layout(LayoutRefinement::default().size_full())
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:scroll-area-demo:max-offset-y")),
                ..Default::default()
            },
            move |cx| {
                vec![cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(web_root.rect.w));
                            layout.size.height = Length::Px(Px(web_root.rect.h));
                            layout
                        },
                        // Match the upstream border inset: the scroll viewport is inset from the
                        // root's border box (fractional due to DPR / layout rounding).
                        padding: Edges::all(Px(inset)),
                        ..Default::default()
                    },
                    move |_cx| vec![scroll_area],
                )]
            },
        )]
    });

    let max = handle.max_offset();
    assert_close_px(
        "scroll area max_offset_y",
        max.y,
        expected_max_offset_y,
        1.0,
    );
    assert!(max.y.0 > 0.0, "expected scroll area to overflow vertically");
}

fn web_vs_fret_layout_scroll_area_horizontal_demo_max_offset_matches_web() {
    let web = read_web_golden("scroll-area-horizontal-demo");
    let theme = web_theme(&web);
    let web_root = web_find_by_class_tokens(
        &theme.root,
        &[
            "relative",
            "w-96",
            "rounded-md",
            "border",
            "whitespace-nowrap",
        ],
    )
    .expect("web horizontal scroll area root");

    let web_viewport = find_first(web_root, &|n| {
        n.computed_style
            .get("overflowX")
            .is_some_and(|v| v == "scroll")
    })
    .expect("web scroll viewport (overflowX=scroll)");

    let metrics = web_viewport
        .scroll
        .as_ref()
        .expect("web scroll viewport missing scroll metrics (regenerate goldens)");

    let expected_max_offset_x = metrics.scroll_width - metrics.client_width;
    let expected_max_offset_y = metrics.scroll_height - metrics.client_height;
    let inset = web_viewport.rect.x - web_root.rect.x;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let handle = ScrollHandle::default();
    let _ = run_fret_root(bounds, |cx| {
        let content = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Px(Px(metrics.scroll_width));
                    layout.size.height = Length::Px(Px(metrics.scroll_height));
                    layout
                },
                ..Default::default()
            },
            |_cx| vec![],
        );

        let scroll_area = fret_ui_shadcn::ScrollArea::new(vec![content])
            .axis(fret_ui::element::ScrollAxis::Both)
            .scroll_handle(handle.clone())
            .refine_layout(LayoutRefinement::default().size_full())
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:scroll-area-horizontal-demo:max-offset")),
                ..Default::default()
            },
            move |cx| {
                vec![cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(web_root.rect.w));
                            layout.size.height = Length::Px(Px(web_root.rect.h));
                            layout
                        },
                        padding: Edges::all(Px(inset)),
                        ..Default::default()
                    },
                    move |_cx| vec![scroll_area],
                )]
            },
        )]
    });

    let max = handle.max_offset();
    assert_close_px(
        "scroll area horizontal max_offset_x",
        max.x,
        expected_max_offset_x,
        1.0,
    );
    assert_close_px(
        "scroll area horizontal max_offset_y",
        max.y,
        expected_max_offset_y,
        1.0,
    );
    assert!(
        max.x.0 > 0.0,
        "expected scroll area to overflow horizontally"
    );
}

fn web_vs_fret_layout_scroll_area_demo_scrollbar_bounds_match_web_hover() {
    let web = read_web_golden("scroll-area-demo.hover");
    let theme = web_theme(&web);
    let web_root = web_find_by_class_tokens(
        &theme.root,
        &["relative", "h-72", "w-48", "rounded-md", "border"],
    )
    .expect("web scroll area root");

    let web_viewport =
        web_find_by_data_slot(&theme.root, "scroll-area-viewport").expect("web scroll viewport");
    let web_scrollbar = web_find_scroll_area_scrollbar(&theme.root, "vertical")
        .expect("web scroll-area-scrollbar (vertical)");

    assert!(
        web_scrollbar
            .attrs
            .get("data-state")
            .is_some_and(|v| v == "visible"),
        "expected web scrollbar to be visible in hover golden"
    );

    let web_content = web_find_best_by(
        web_viewport,
        &|n| n.tag == "div" && n.rect.h > web_viewport.rect.h + 1.0,
        &|n| -n.rect.h,
    )
    .expect("web scroll content (taller than viewport)");

    let expected_rel = WebRect {
        x: web_scrollbar.rect.x - web_root.rect.x,
        y: web_scrollbar.rect.y - web_root.rect.y,
        w: web_scrollbar.rect.w,
        h: web_scrollbar.rect.h,
    };

    // Match the web border inset: the viewport is inset from the root, and the scrollbar is
    // positioned against that inner padding box.
    let inset = web_viewport.rect.x - web_root.rect.x;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let handle = ScrollHandle::default();

    let render = |cx: &mut fret_ui::ElementContext<'_, App>| {
        let content = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Px(Px(web_content.rect.h));
                    layout
                },
                ..Default::default()
            },
            |_cx| vec![],
        );

        let scroll_area =
            fret_ui_shadcn::ScrollAreaRoot::new(fret_ui_shadcn::ScrollAreaViewport::new(vec![
                content,
            ]))
            .scroll_handle(handle.clone())
            .scrollbar(
                fret_ui_shadcn::ScrollAreaScrollbar::new()
                    .orientation(fret_ui_shadcn::ScrollAreaScrollbarOrientation::Vertical),
            )
            .refine_layout(LayoutRefinement::default().size_full())
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:scroll-area-demo:hover")),
                ..Default::default()
            },
            move |cx| {
                vec![cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(web_root.rect.w));
                            layout.size.height = Length::Px(Px(web_root.rect.h));
                            layout
                        },
                        padding: Edges::all(Px(inset)),
                        ..Default::default()
                    },
                    move |_cx| vec![scroll_area],
                )]
            },
        )]
    };

    let root_node = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        render,
    );
    ui.set_root(root_node);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap1 = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot (pre-hover)");

    let panel1 = find_semantics(
        &snap1,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-demo:hover"),
    )
    .expect("fret hover panel (pre-hover)");

    let expected_abs_pre = WebRect {
        x: panel1.bounds.origin.x.0 + expected_rel.x,
        y: panel1.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };

    assert!(
        find_node_with_bounds_close(&ui, panel1.id, expected_abs_pre, 2.0).is_none(),
        "expected scrollbar to be absent before hover"
    );

    let hover_pos = Point::new(
        Px(panel1.bounds.origin.x.0 + (web_viewport.rect.x + web_viewport.rect.w * 0.5)),
        Px(panel1.bounds.origin.y.0 + (web_viewport.rect.y + web_viewport.rect.h * 0.5)),
    );
    let move_event = Event::Pointer(PointerEvent::Move {
        position: hover_pos,
        buttons: MouseButtons::default(),
        modifiers: Modifiers::default(),
        pointer_id: PointerId(0),
        pointer_type: PointerType::Mouse,
    });
    ui.dispatch_event(&mut app, &mut services, &move_event);

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    let root_node = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        render,
    );
    ui.set_root(root_node);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap2 = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot (post-hover)");

    let panel2 = find_semantics(
        &snap2,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-demo:hover"),
    )
    .expect("fret hover panel (post-hover)");

    let expected_abs = WebRect {
        x: panel2.bounds.origin.x.0 + expected_rel.x,
        y: panel2.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };

    let (_, scrollbar_bounds) = find_node_with_bounds_close(&ui, panel2.id, expected_abs, 2.0)
        .expect("fret scrollbar bounds after hover");

    assert_rect_close_px(
        "scroll-area-demo scrollbar",
        scrollbar_bounds,
        expected_abs,
        2.0,
    );
}

fn web_vs_fret_layout_scroll_area_demo_thumb_background_matches_web_hover_light() {
    let web = read_web_golden("scroll-area-demo.hover");
    let theme = web
        .themes
        .get("light")
        .expect("missing light theme in scroll-area-demo.hover");

    let web_root = web_find_by_class_tokens(
        &theme.root,
        &["relative", "h-72", "w-48", "rounded-md", "border"],
    )
    .expect("web scroll area root");

    let web_viewport =
        web_find_by_data_slot(&theme.root, "scroll-area-viewport").expect("web scroll viewport");
    let web_scrollbar = web_find_scroll_area_scrollbar(&theme.root, "vertical")
        .expect("web scroll-area-scrollbar (vertical)");
    let web_thumb =
        web_find_scroll_area_thumb_in_scrollbar(web_scrollbar).expect("web scroll-area-thumb");

    assert!(
        web_scrollbar
            .attrs
            .get("data-state")
            .is_some_and(|v| v == "visible"),
        "expected web scrollbar to be visible in hover golden"
    );

    let expected_bg = web_thumb
        .computed_style
        .get("backgroundColor")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web thumb backgroundColor");

    let expected_rel = WebRect {
        x: web_thumb.rect.x - web_root.rect.x,
        y: web_thumb.rect.y - web_root.rect.y,
        w: web_thumb.rect.w,
        h: web_thumb.rect.h,
    };

    let inset = web_viewport.rect.x - web_root.rect.x;

    let web_content = web_find_best_by(
        web_viewport,
        &|n| n.tag == "div" && n.rect.h > web_viewport.rect.h + 1.0,
        &|n| -n.rect.h,
    )
    .expect("web scroll content (taller than viewport)");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let handle = ScrollHandle::default();

    let render = |cx: &mut fret_ui::ElementContext<'_, App>| {
        let content = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Px(Px(web_content.rect.h));
                    layout
                },
                ..Default::default()
            },
            |_cx| vec![],
        );

        let scroll_area =
            fret_ui_shadcn::ScrollAreaRoot::new(fret_ui_shadcn::ScrollAreaViewport::new(vec![
                content,
            ]))
            .scroll_handle(handle.clone())
            .scrollbar(
                fret_ui_shadcn::ScrollAreaScrollbar::new()
                    .orientation(fret_ui_shadcn::ScrollAreaScrollbarOrientation::Vertical),
            )
            .refine_layout(LayoutRefinement::default().size_full())
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:scroll-area-demo:hover-thumb-bg-light")),
                ..Default::default()
            },
            move |cx| {
                vec![cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(web_root.rect.w));
                            layout.size.height = Length::Px(Px(web_root.rect.h));
                            layout
                        },
                        padding: Edges::all(Px(inset)),
                        ..Default::default()
                    },
                    move |_cx| vec![scroll_area],
                )]
            },
        )]
    };

    let root_node = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        render,
    );
    ui.set_root(root_node);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap1 = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot (pre-hover)");
    let panel1 = find_semantics(
        &snap1,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-demo:hover-thumb-bg-light"),
    )
    .expect("fret scroll-area panel (pre-hover)");

    let hover_pos = Point::new(
        Px(panel1.bounds.origin.x.0 + (web_viewport.rect.x + web_viewport.rect.w * 0.5)),
        Px(panel1.bounds.origin.y.0 + (web_viewport.rect.y + web_viewport.rect.h * 0.5)),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: hover_pos,
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    let root_node = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        render,
    );
    ui.set_root(root_node);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap2 = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot (post-hover)");
    let panel2 = find_semantics(
        &snap2,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-demo:hover-thumb-bg-light"),
    )
    .expect("fret scroll-area panel (post-hover)");

    let expected_abs = WebRect {
        x: panel2.bounds.origin.x.0 + expected_rel.x,
        y: panel2.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    let (_rect, bg) =
        find_scene_quad_background_with_rect_close(&scene, expected_abs, 2.0).expect("thumb quad");
    assert_rgba_close(
        "scroll-area-demo.hover thumb background",
        color_to_rgba(bg),
        expected_bg,
        0.02,
    );
}

fn web_vs_fret_layout_scroll_area_demo_thumb_background_matches_web_hover_dark() {
    let web = read_web_golden("scroll-area-demo.hover");
    let theme = web
        .themes
        .get("dark")
        .expect("missing dark theme in scroll-area-demo.hover");

    let web_root = web_find_by_class_tokens(
        &theme.root,
        &["relative", "h-72", "w-48", "rounded-md", "border"],
    )
    .expect("web scroll area root");

    let web_viewport =
        web_find_by_data_slot(&theme.root, "scroll-area-viewport").expect("web scroll viewport");
    let web_scrollbar = web_find_scroll_area_scrollbar(&theme.root, "vertical")
        .expect("web scroll-area-scrollbar (vertical)");
    let web_thumb =
        web_find_scroll_area_thumb_in_scrollbar(web_scrollbar).expect("web scroll-area-thumb");

    assert!(
        web_scrollbar
            .attrs
            .get("data-state")
            .is_some_and(|v| v == "visible"),
        "expected web scrollbar to be visible in hover golden"
    );

    let expected_bg = web_thumb
        .computed_style
        .get("backgroundColor")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web thumb backgroundColor");

    let expected_rel = WebRect {
        x: web_thumb.rect.x - web_root.rect.x,
        y: web_thumb.rect.y - web_root.rect.y,
        w: web_thumb.rect.w,
        h: web_thumb.rect.h,
    };

    let inset = web_viewport.rect.x - web_root.rect.x;

    let web_content = web_find_best_by(
        web_viewport,
        &|n| n.tag == "div" && n.rect.h > web_viewport.rect.h + 1.0,
        &|n| -n.rect.h,
    )
    .expect("web scroll content (taller than viewport)");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let handle = ScrollHandle::default();

    let render = |cx: &mut fret_ui::ElementContext<'_, App>| {
        let content = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Px(Px(web_content.rect.h));
                    layout
                },
                ..Default::default()
            },
            |_cx| vec![],
        );

        let scroll_area =
            fret_ui_shadcn::ScrollAreaRoot::new(fret_ui_shadcn::ScrollAreaViewport::new(vec![
                content,
            ]))
            .scroll_handle(handle.clone())
            .scrollbar(
                fret_ui_shadcn::ScrollAreaScrollbar::new()
                    .orientation(fret_ui_shadcn::ScrollAreaScrollbarOrientation::Vertical),
            )
            .refine_layout(LayoutRefinement::default().size_full())
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:scroll-area-demo:hover-thumb-bg-dark")),
                ..Default::default()
            },
            move |cx| {
                vec![cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(web_root.rect.w));
                            layout.size.height = Length::Px(Px(web_root.rect.h));
                            layout
                        },
                        padding: Edges::all(Px(inset)),
                        ..Default::default()
                    },
                    move |_cx| vec![scroll_area],
                )]
            },
        )]
    };

    let root_node = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        render,
    );
    ui.set_root(root_node);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap1 = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot (pre-hover)");
    let panel1 = find_semantics(
        &snap1,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-demo:hover-thumb-bg-dark"),
    )
    .expect("fret scroll-area panel (pre-hover)");

    let hover_pos = Point::new(
        Px(panel1.bounds.origin.x.0 + (web_viewport.rect.x + web_viewport.rect.w * 0.5)),
        Px(panel1.bounds.origin.y.0 + (web_viewport.rect.y + web_viewport.rect.h * 0.5)),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: hover_pos,
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    let root_node = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        render,
    );
    ui.set_root(root_node);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap2 = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot (post-hover)");
    let panel2 = find_semantics(
        &snap2,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-demo:hover-thumb-bg-dark"),
    )
    .expect("fret scroll-area panel (post-hover)");

    let expected_abs = WebRect {
        x: panel2.bounds.origin.x.0 + expected_rel.x,
        y: panel2.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    let (_rect, bg) =
        find_scene_quad_background_with_rect_close(&scene, expected_abs, 2.0).expect("thumb quad");
    assert_rgba_close(
        "scroll-area-demo.hover dark thumb background",
        color_to_rgba(bg),
        expected_bg,
        0.02,
    );
}

fn web_vs_fret_layout_scroll_area_demo_scrollbar_hides_after_hover_out_delay() {
    let web_early = read_web_golden("scroll-area-demo.hover-out-550ms");
    let theme_early = web_theme(&web_early);
    let web_root = web_find_by_class_tokens(
        &theme_early.root,
        &["relative", "h-72", "w-48", "rounded-md", "border"],
    )
    .expect("web scroll area root");

    let web_viewport = web_find_by_data_slot(&theme_early.root, "scroll-area-viewport")
        .expect("web scroll viewport");
    let web_scrollbar_early = web_find_scroll_area_scrollbar(&theme_early.root, "vertical")
        .expect("web scroll-area-scrollbar (vertical, early)");
    let web_thumb_early =
        web_find_scroll_area_thumb_in_scrollbar(web_scrollbar_early).expect("web thumb (early)");

    assert!(
        web_scrollbar_early
            .attrs
            .get("data-state")
            .is_some_and(|v| v == "visible"),
        "expected early web scrollbar to be visible"
    );

    let web_late = read_web_golden("scroll-area-demo.hover-out-650ms");
    let theme_late = web_theme(&web_late);
    assert!(
        web_find_scroll_area_scrollbar(&theme_late.root, "vertical").is_none(),
        "expected late web scrollbar to be absent"
    );

    let web_content = web_find_best_by(
        web_viewport,
        &|n| n.tag == "div" && n.rect.h > web_viewport.rect.h + 1.0,
        &|n| -n.rect.h,
    )
    .expect("web scroll content (taller than viewport)");

    let expected_rel = WebRect {
        x: web_thumb_early.rect.x - web_root.rect.x,
        y: web_thumb_early.rect.y - web_root.rect.y,
        w: web_thumb_early.rect.w,
        h: web_thumb_early.rect.h,
    };

    let inset = web_viewport.rect.x - web_root.rect.x;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme_early.viewport.w), Px(theme_early.viewport.h)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let handle = ScrollHandle::default();
    let content_h = web_content.rect.h;
    let root_w = web_root.rect.w;
    let root_h = web_root.rect.h;

    let render = |cx: &mut fret_ui::ElementContext<'_, App>| {
        let handle = handle.clone();
        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:scroll-area-demo:hover-out")),
                ..Default::default()
            },
            move |cx| {
                vec![cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(root_w));
                            layout.size.height = Length::Px(Px(root_h));
                            layout
                        },
                        padding: Edges::all(Px(inset)),
                        ..Default::default()
                    },
                    move |cx| {
                        let content = cx.container(
                            ContainerProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Fill;
                                    layout.size.height = Length::Px(Px(content_h));
                                    layout
                                },
                                ..Default::default()
                            },
                            |_cx| vec![],
                        );

                        let scroll_area =
                            fret_ui_shadcn::ScrollAreaRoot::new(
                                fret_ui_shadcn::ScrollAreaViewport::new(vec![content]),
                            )
                            .scroll_handle(handle.clone())
                            .scrollbar(fret_ui_shadcn::ScrollAreaScrollbar::new().orientation(
                                fret_ui_shadcn::ScrollAreaScrollbarOrientation::Vertical,
                            ))
                            .refine_layout(LayoutRefinement::default().size_full())
                            .into_element(cx);

                        vec![scroll_area]
                    },
                )]
            },
        )]
    };

    macro_rules! render_at {
        ($frame:expr) => {{
            app.set_frame_id(FrameId($frame));
            let root_node = fret_ui::declarative::render_root(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                "web-vs-fret-layout",
                &render,
            );
            ui.set_root(root_node);
            ui.request_semantics_snapshot();
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
            ui.semantics_snapshot()
                .cloned()
                .expect("expected semantics snapshot")
        }};
    }

    let snap0 = render_at!(0);
    let panel0 = find_semantics(
        &snap0,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-demo:hover-out"),
    )
    .expect("fret hover-out panel (initial)");
    let expected_abs0 = WebRect {
        x: panel0.bounds.origin.x.0 + expected_rel.x,
        y: panel0.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };
    let mut scene0 = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene0, 1.0);
    assert!(
        find_scene_quad_with_rect_close(&scene0, expected_abs0, 2.0).is_none(),
        "expected thumb quad to be absent before hover"
    );

    let hover_pos = Point::new(
        Px(panel0.bounds.origin.x.0 + (web_viewport.rect.x + web_viewport.rect.w * 0.5)),
        Px(panel0.bounds.origin.y.0 + (web_viewport.rect.y + web_viewport.rect.h * 0.5)),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: hover_pos,
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let snap1 = render_at!(1);
    let panel1 = find_semantics(
        &snap1,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-demo:hover-out"),
    )
    .expect("fret hover-out panel (hovered)");
    let expected_abs1 = WebRect {
        x: panel1.bounds.origin.x.0 + expected_rel.x,
        y: panel1.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };
    let mut scene1 = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene1, 1.0);
    assert!(
        find_scene_quad_with_rect_close(&scene1, expected_abs1, 2.0).is_some(),
        "expected thumb quad to be present after hover"
    );

    // Move outside the ScrollArea hover region (Radix uses pointer leave on the root).
    // Using the outer panel bounds is more robust than aiming for a "gap" near the viewport.
    // Move inside the window but outside the ScrollArea bounds so hover state clears.
    let leave_pos = Point::new(Px(root_w + 100.0), Px(root_h + 100.0));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: leave_pos,
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    // Render once at the "leave tick" so the hover timer is armed.
    let snap_leave = render_at!(2);
    let panel_leave = find_semantics(
        &snap_leave,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-demo:hover-out"),
    )
    .expect("fret hover-out panel (leave)");
    let expected_abs_leave = WebRect {
        x: panel_leave.bounds.origin.x.0 + expected_rel.x,
        y: panel_leave.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };
    let mut scene_leave = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene_leave, 1.0);
    assert!(
        find_scene_quad_with_rect_close(&scene_leave, expected_abs_leave, 2.0).is_some(),
        "expected thumb quad to remain visible immediately after leave"
    );

    // The scrollHideDelay timer advances via per-frame ticks in the ScrollAreaVisibility driver.
    // To match the web goldens, step through frames rather than jumping the FrameId.
    let mut snap_early: Option<fret_core::SemanticsSnapshot> = None;
    let mut snap_late: Option<fret_core::SemanticsSnapshot> = None;
    let mut scene_early: Option<Scene> = None;
    let mut scene_late: Option<Scene> = None;
    for frame in 3..=(2 + 39) {
        let snap = render_at!(frame);
        if frame == 2 + 33 {
            snap_early = Some(snap);
            let mut scene = Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
            scene_early = Some(scene);
        } else if frame == 2 + 39 {
            snap_late = Some(snap);
            let mut scene = Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
            scene_late = Some(scene);
        }
    }

    // ~550ms after leaving (33 ticks at ~60fps): still visible.
    let snap_early = snap_early.expect("missing snap_early");
    let panel_early = find_semantics(
        &snap_early,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-demo:hover-out"),
    )
    .expect("fret hover-out panel (early)");
    let expected_abs_early = WebRect {
        x: panel_early.bounds.origin.x.0 + expected_rel.x,
        y: panel_early.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };
    let scene_early = scene_early.expect("missing scene_early");
    assert!(
        find_scene_quad_with_rect_close(&scene_early, expected_abs_early, 2.0).is_some(),
        "expected thumb quad to remain visible before scrollHideDelay"
    );

    // ~650ms after leaving (39 ticks at ~60fps): hidden.
    let snap_late = snap_late.expect("missing snap_late");
    let panel_late = find_semantics(
        &snap_late,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-demo:hover-out"),
    )
    .expect("fret hover-out panel (late)");
    let expected_abs_late = WebRect {
        x: panel_late.bounds.origin.x.0 + expected_rel.x,
        y: panel_late.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };
    let scene_late = scene_late.expect("missing scene_late");
    assert!(
        find_scene_quad_with_rect_close(&scene_late, expected_abs_late, 2.0).is_none(),
        "expected thumb quad to be hidden after scrollHideDelay"
    );
}

fn web_vs_fret_layout_scroll_area_demo_thumb_bounds_match_web_scrolled() {
    let web = read_web_golden("scroll-area-demo.scrolled");
    let theme = web_theme(&web);
    let web_root = web_find_by_class_tokens(
        &theme.root,
        &["relative", "h-72", "w-48", "rounded-md", "border"],
    )
    .expect("web scroll area root");

    let web_viewport =
        web_find_by_data_slot(&theme.root, "scroll-area-viewport").expect("web scroll viewport");
    let web_scrollbar = web_find_scroll_area_scrollbar(&theme.root, "vertical")
        .expect("web scroll-area-scrollbar (vertical)");
    let web_thumb =
        web_find_scroll_area_thumb_in_scrollbar(web_scrollbar).expect("web scroll-area-thumb");
    let web_scroll = web_viewport
        .scroll
        .as_ref()
        .expect("web scroll viewport scroll metrics");

    assert!(
        (web_scroll.scroll_top - 80.0).abs() < 0.01,
        "expected scrollTop=80 in golden"
    );

    assert!(
        web_scrollbar
            .attrs
            .get("data-state")
            .is_some_and(|v| v == "visible"),
        "expected web scrollbar to be visible in scrolled golden"
    );

    let web_content = web_find_best_by(
        web_viewport,
        &|n| n.tag == "div" && n.rect.h > web_viewport.rect.h + 1.0,
        &|n| -n.rect.h,
    )
    .expect("web scroll content (taller than viewport)");

    let expected_rel = WebRect {
        x: web_thumb.rect.x - web_root.rect.x,
        y: web_thumb.rect.y - web_root.rect.y,
        w: web_thumb.rect.w,
        h: web_thumb.rect.h,
    };

    let inset = web_viewport.rect.x - web_root.rect.x;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let handle = ScrollHandle::default();

    let render = |cx: &mut fret_ui::ElementContext<'_, App>| {
        let content = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Px(Px(web_content.rect.h));
                    layout
                },
                ..Default::default()
            },
            |_cx| vec![],
        );

        let scroll_area =
            fret_ui_shadcn::ScrollAreaRoot::new(fret_ui_shadcn::ScrollAreaViewport::new(vec![
                content,
            ]))
            .scroll_handle(handle.clone())
            .scrollbar(
                fret_ui_shadcn::ScrollAreaScrollbar::new()
                    .orientation(fret_ui_shadcn::ScrollAreaScrollbarOrientation::Vertical),
            )
            .refine_layout(LayoutRefinement::default().size_full())
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:scroll-area-demo:scrolled")),
                ..Default::default()
            },
            move |cx| {
                vec![cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(web_root.rect.w));
                            layout.size.height = Length::Px(Px(web_root.rect.h));
                            layout
                        },
                        padding: Edges::all(Px(inset)),
                        ..Default::default()
                    },
                    move |_cx| vec![scroll_area],
                )]
            },
        )]
    };

    let root_node = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        render,
    );
    ui.set_root(root_node);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap1 = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot (pre-hover)");

    let panel1 = find_semantics(
        &snap1,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-demo:scrolled"),
    )
    .expect("fret scrolled panel (pre-hover)");

    let expected_abs_pre = WebRect {
        x: panel1.bounds.origin.x.0 + expected_rel.x,
        y: panel1.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };

    let mut scene1 = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene1, 1.0);
    assert!(
        find_scene_quad_with_rect_close(&scene1, expected_abs_pre, 2.0).is_none(),
        "expected thumb quad to be absent before hover"
    );

    let hover_pos = Point::new(
        Px(panel1.bounds.origin.x.0 + (web_viewport.rect.x + web_viewport.rect.w * 0.5)),
        Px(panel1.bounds.origin.y.0 + (web_viewport.rect.y + web_viewport.rect.h * 0.5)),
    );
    let move_event = Event::Pointer(PointerEvent::Move {
        position: hover_pos,
        buttons: MouseButtons::default(),
        modifiers: Modifiers::default(),
        pointer_id: PointerId(0),
        pointer_type: PointerType::Mouse,
    });
    ui.dispatch_event(&mut app, &mut services, &move_event);

    handle.set_offset(Point::new(Px(0.0), Px(web_scroll.scroll_top)));

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    let root_node = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        render,
    );
    ui.set_root(root_node);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap2 = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot (post-hover)");

    let panel2 = find_semantics(
        &snap2,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-demo:scrolled"),
    )
    .expect("fret scrolled panel (post-hover)");

    let expected_abs = WebRect {
        x: panel2.bounds.origin.x.0 + expected_rel.x,
        y: panel2.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };

    let mut scene2 = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene2, 1.0);
    let thumb_bounds =
        find_scene_quad_with_rect_close(&scene2, expected_abs, 2.0).expect("fret thumb quad");
    assert_rect_close_px("scroll-area-demo thumb", thumb_bounds, expected_abs, 2.0);
}

fn web_vs_fret_layout_scroll_area_horizontal_demo_scrollbar_bounds_match_web_hover() {
    let web = read_web_golden("scroll-area-horizontal-demo.hover");
    let theme = web_theme(&web);
    let web_root =
        web_find_by_class_tokens(&theme.root, &["relative", "w-96", "rounded-md", "border"])
            .expect("web scroll area root");

    let web_viewport =
        web_find_by_data_slot(&theme.root, "scroll-area-viewport").expect("web scroll viewport");
    let web_scrollbar = web_find_scroll_area_scrollbar(&theme.root, "horizontal")
        .expect("web scroll-area-scrollbar (horizontal)");
    let _web_thumb = web_find_scroll_area_thumb_in_scrollbar(web_scrollbar)
        .expect("web scroll-area-thumb (horizontal)");

    assert!(
        web_scrollbar
            .attrs
            .get("data-state")
            .is_some_and(|v| v == "visible"),
        "expected web scrollbar to be visible in hover golden"
    );

    let web_content = web_find_best_by(
        web_viewport,
        &|n| n.tag == "div" && n.rect.w > web_viewport.rect.w + 1.0,
        &|n| -n.rect.w,
    )
    .expect("web scroll content (wider than viewport)");

    let expected_rel = WebRect {
        x: web_scrollbar.rect.x - web_root.rect.x,
        y: web_scrollbar.rect.y - web_root.rect.y,
        w: web_scrollbar.rect.w,
        h: web_scrollbar.rect.h,
    };

    let inset = web_viewport.rect.x - web_root.rect.x;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let handle = ScrollHandle::default();

    let render = |cx: &mut fret_ui::ElementContext<'_, App>| {
        let content = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Px(Px(web_content.rect.w));
                    layout.size.height = Length::Fill;
                    layout
                },
                ..Default::default()
            },
            |_cx| vec![],
        );

        let scroll_area =
            fret_ui_shadcn::ScrollAreaRoot::new(fret_ui_shadcn::ScrollAreaViewport::new(vec![
                content,
            ]))
            .scroll_handle(handle.clone())
            .scrollbar(
                fret_ui_shadcn::ScrollAreaScrollbar::new()
                    .orientation(fret_ui_shadcn::ScrollAreaScrollbarOrientation::Vertical),
            )
            .scrollbar(
                fret_ui_shadcn::ScrollAreaScrollbar::new()
                    .orientation(fret_ui_shadcn::ScrollAreaScrollbarOrientation::Horizontal),
            )
            .corner(true)
            .refine_layout(LayoutRefinement::default().size_full())
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:scroll-area-horizontal-demo:hover")),
                ..Default::default()
            },
            move |cx| {
                vec![cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(web_root.rect.w));
                            layout.size.height = Length::Px(Px(web_root.rect.h));
                            layout
                        },
                        padding: Edges::all(Px(inset)),
                        ..Default::default()
                    },
                    move |_cx| vec![scroll_area],
                )]
            },
        )]
    };

    let root_node = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        render,
    );
    ui.set_root(root_node);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap1 = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot (pre-hover)");

    let panel1 = find_semantics(
        &snap1,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-horizontal-demo:hover"),
    )
    .expect("fret hover panel (pre-hover)");

    let expected_abs_pre = WebRect {
        x: panel1.bounds.origin.x.0 + expected_rel.x,
        y: panel1.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };

    assert!(
        find_node_with_bounds_close(&ui, panel1.id, expected_abs_pre, 2.0).is_none(),
        "expected scrollbar to be absent before hover"
    );

    let hover_pos = Point::new(
        Px(panel1.bounds.origin.x.0 + (web_viewport.rect.x + web_viewport.rect.w * 0.5)),
        Px(panel1.bounds.origin.y.0 + (web_viewport.rect.y + web_viewport.rect.h * 0.5)),
    );
    let move_event = Event::Pointer(PointerEvent::Move {
        position: hover_pos,
        buttons: MouseButtons::default(),
        modifiers: Modifiers::default(),
        pointer_id: PointerId(0),
        pointer_type: PointerType::Mouse,
    });
    ui.dispatch_event(&mut app, &mut services, &move_event);

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    let root_node = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        render,
    );
    ui.set_root(root_node);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap2 = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot (post-hover)");

    let panel2 = find_semantics(
        &snap2,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-horizontal-demo:hover"),
    )
    .expect("fret hover panel (post-hover)");

    let expected_abs = WebRect {
        x: panel2.bounds.origin.x.0 + expected_rel.x,
        y: panel2.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };

    let (_, scrollbar_bounds) = find_node_with_bounds_close(&ui, panel2.id, expected_abs, 2.0)
        .expect("fret scrollbar bounds after hover");

    assert_rect_close_px(
        "scroll-area-horizontal-demo scrollbar",
        scrollbar_bounds,
        expected_abs,
        2.0,
    );
}

fn web_vs_fret_layout_scroll_area_horizontal_demo_thumb_background_matches_web_hover_light() {
    let web = read_web_golden("scroll-area-horizontal-demo.hover");
    let theme = web
        .themes
        .get("light")
        .expect("missing light theme in scroll-area-horizontal-demo.hover");

    let web_root =
        web_find_by_class_tokens(&theme.root, &["relative", "w-96", "rounded-md", "border"])
            .expect("web scroll area root");

    let web_viewport =
        web_find_by_data_slot(&theme.root, "scroll-area-viewport").expect("web scroll viewport");
    let web_scrollbar = web_find_scroll_area_scrollbar(&theme.root, "horizontal")
        .expect("web scroll-area-scrollbar (horizontal)");
    let web_thumb = web_find_scroll_area_thumb_in_scrollbar(web_scrollbar)
        .expect("web scroll-area-thumb (horizontal)");

    assert!(
        web_scrollbar
            .attrs
            .get("data-state")
            .is_some_and(|v| v == "visible"),
        "expected web scrollbar to be visible in hover golden"
    );

    let expected_bg = web_thumb
        .computed_style
        .get("backgroundColor")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web thumb backgroundColor");

    let expected_rel = WebRect {
        x: web_thumb.rect.x - web_root.rect.x,
        y: web_thumb.rect.y - web_root.rect.y,
        w: web_thumb.rect.w,
        h: web_thumb.rect.h,
    };

    let inset = web_viewport.rect.x - web_root.rect.x;

    let web_content = web_find_best_by(
        web_viewport,
        &|n| n.tag == "div" && n.rect.w > web_viewport.rect.w + 1.0,
        &|n| -n.rect.w,
    )
    .expect("web scroll content (wider than viewport)");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let handle = ScrollHandle::default();

    let render = |cx: &mut fret_ui::ElementContext<'_, App>| {
        let content = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Px(Px(web_content.rect.w));
                    layout.size.height = Length::Fill;
                    layout
                },
                ..Default::default()
            },
            |_cx| vec![],
        );

        let scroll_area =
            fret_ui_shadcn::ScrollAreaRoot::new(fret_ui_shadcn::ScrollAreaViewport::new(vec![
                content,
            ]))
            .scroll_handle(handle.clone())
            .scrollbar(
                fret_ui_shadcn::ScrollAreaScrollbar::new()
                    .orientation(fret_ui_shadcn::ScrollAreaScrollbarOrientation::Vertical),
            )
            .scrollbar(
                fret_ui_shadcn::ScrollAreaScrollbar::new()
                    .orientation(fret_ui_shadcn::ScrollAreaScrollbarOrientation::Horizontal),
            )
            .corner(true)
            .refine_layout(LayoutRefinement::default().size_full())
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from(
                    "Golden:scroll-area-horizontal-demo:hover-thumb-bg-light",
                )),
                ..Default::default()
            },
            move |cx| {
                vec![cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(web_root.rect.w));
                            layout.size.height = Length::Px(Px(web_root.rect.h));
                            layout
                        },
                        padding: Edges::all(Px(inset)),
                        ..Default::default()
                    },
                    move |_cx| vec![scroll_area],
                )]
            },
        )]
    };

    let root_node = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        render,
    );
    ui.set_root(root_node);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap1 = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot (pre-hover)");
    let panel1 = find_semantics(
        &snap1,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-horizontal-demo:hover-thumb-bg-light"),
    )
    .expect("fret scroll-area panel (pre-hover)");

    let hover_pos = Point::new(
        Px(panel1.bounds.origin.x.0 + (web_viewport.rect.x + web_viewport.rect.w * 0.5)),
        Px(panel1.bounds.origin.y.0 + (web_viewport.rect.y + web_viewport.rect.h * 0.5)),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: hover_pos,
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    let root_node = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        render,
    );
    ui.set_root(root_node);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap2 = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot (post-hover)");
    let panel2 = find_semantics(
        &snap2,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-horizontal-demo:hover-thumb-bg-light"),
    )
    .expect("fret scroll-area panel (post-hover)");

    let expected_abs = WebRect {
        x: panel2.bounds.origin.x.0 + expected_rel.x,
        y: panel2.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    let (_rect, bg) =
        find_scene_quad_background_with_rect_close(&scene, expected_abs, 2.0).expect("thumb quad");
    assert_rgba_close(
        "scroll-area-horizontal-demo.hover thumb background",
        color_to_rgba(bg),
        expected_bg,
        0.02,
    );
}

fn web_vs_fret_layout_scroll_area_horizontal_demo_thumb_background_matches_web_hover_dark() {
    let web = read_web_golden("scroll-area-horizontal-demo.hover");
    let theme = web
        .themes
        .get("dark")
        .expect("missing dark theme in scroll-area-horizontal-demo.hover");

    let web_root =
        web_find_by_class_tokens(&theme.root, &["relative", "w-96", "rounded-md", "border"])
            .expect("web scroll area root");

    let web_viewport =
        web_find_by_data_slot(&theme.root, "scroll-area-viewport").expect("web scroll viewport");
    let web_scrollbar = web_find_scroll_area_scrollbar(&theme.root, "horizontal")
        .expect("web scroll-area-scrollbar (horizontal)");
    let web_thumb = web_find_scroll_area_thumb_in_scrollbar(web_scrollbar)
        .expect("web scroll-area-thumb (horizontal)");

    assert!(
        web_scrollbar
            .attrs
            .get("data-state")
            .is_some_and(|v| v == "visible"),
        "expected web scrollbar to be visible in hover golden"
    );

    let expected_bg = web_thumb
        .computed_style
        .get("backgroundColor")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web thumb backgroundColor");

    let expected_rel = WebRect {
        x: web_thumb.rect.x - web_root.rect.x,
        y: web_thumb.rect.y - web_root.rect.y,
        w: web_thumb.rect.w,
        h: web_thumb.rect.h,
    };

    let inset = web_viewport.rect.x - web_root.rect.x;

    let web_content = web_find_best_by(
        web_viewport,
        &|n| n.tag == "div" && n.rect.w > web_viewport.rect.w + 1.0,
        &|n| -n.rect.w,
    )
    .expect("web scroll content (wider than viewport)");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let handle = ScrollHandle::default();

    let render = |cx: &mut fret_ui::ElementContext<'_, App>| {
        let content = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Px(Px(web_content.rect.w));
                    layout.size.height = Length::Fill;
                    layout
                },
                ..Default::default()
            },
            |_cx| vec![],
        );

        let scroll_area =
            fret_ui_shadcn::ScrollAreaRoot::new(fret_ui_shadcn::ScrollAreaViewport::new(vec![
                content,
            ]))
            .scroll_handle(handle.clone())
            .scrollbar(
                fret_ui_shadcn::ScrollAreaScrollbar::new()
                    .orientation(fret_ui_shadcn::ScrollAreaScrollbarOrientation::Vertical),
            )
            .scrollbar(
                fret_ui_shadcn::ScrollAreaScrollbar::new()
                    .orientation(fret_ui_shadcn::ScrollAreaScrollbarOrientation::Horizontal),
            )
            .corner(true)
            .refine_layout(LayoutRefinement::default().size_full())
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from(
                    "Golden:scroll-area-horizontal-demo:hover-thumb-bg-dark",
                )),
                ..Default::default()
            },
            move |cx| {
                vec![cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(web_root.rect.w));
                            layout.size.height = Length::Px(Px(web_root.rect.h));
                            layout
                        },
                        padding: Edges::all(Px(inset)),
                        ..Default::default()
                    },
                    move |_cx| vec![scroll_area],
                )]
            },
        )]
    };

    let root_node = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        render,
    );
    ui.set_root(root_node);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap1 = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot (pre-hover)");
    let panel1 = find_semantics(
        &snap1,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-horizontal-demo:hover-thumb-bg-dark"),
    )
    .expect("fret scroll-area panel (pre-hover)");

    let hover_pos = Point::new(
        Px(panel1.bounds.origin.x.0 + (web_viewport.rect.x + web_viewport.rect.w * 0.5)),
        Px(panel1.bounds.origin.y.0 + (web_viewport.rect.y + web_viewport.rect.h * 0.5)),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: hover_pos,
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    let root_node = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        render,
    );
    ui.set_root(root_node);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap2 = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot (post-hover)");
    let panel2 = find_semantics(
        &snap2,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-horizontal-demo:hover-thumb-bg-dark"),
    )
    .expect("fret scroll-area panel (post-hover)");

    let expected_abs = WebRect {
        x: panel2.bounds.origin.x.0 + expected_rel.x,
        y: panel2.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    let (_rect, bg) =
        find_scene_quad_background_with_rect_close(&scene, expected_abs, 2.0).expect("thumb quad");
    assert_rgba_close(
        "scroll-area-horizontal-demo.hover dark thumb background",
        color_to_rgba(bg),
        expected_bg,
        0.02,
    );
}

fn web_vs_fret_layout_scroll_area_horizontal_demo_scrollbar_hides_after_hover_out_delay() {
    let web_early = read_web_golden("scroll-area-horizontal-demo.hover-out-550ms");
    let theme_early = web_theme(&web_early);
    let web_root = web_find_by_class_tokens(
        &theme_early.root,
        &["relative", "w-96", "rounded-md", "border"],
    )
    .expect("web scroll area root");

    let web_viewport = web_find_by_data_slot(&theme_early.root, "scroll-area-viewport")
        .expect("web scroll viewport");
    let web_scrollbar_early = web_find_scroll_area_scrollbar(&theme_early.root, "horizontal")
        .expect("web scroll-area-scrollbar (horizontal, early)");
    let web_thumb_early =
        web_find_scroll_area_thumb_in_scrollbar(web_scrollbar_early).expect("web thumb (early)");

    assert!(
        web_scrollbar_early
            .attrs
            .get("data-state")
            .is_some_and(|v| v == "visible"),
        "expected early web scrollbar to be visible"
    );

    let web_late = read_web_golden("scroll-area-horizontal-demo.hover-out-650ms");
    let theme_late = web_theme(&web_late);
    assert!(
        web_find_scroll_area_scrollbar(&theme_late.root, "horizontal").is_none(),
        "expected late web scrollbar to be absent"
    );

    let web_content = web_find_best_by(
        web_viewport,
        &|n| n.tag == "div" && n.rect.w > web_viewport.rect.w + 1.0,
        &|n| -n.rect.w,
    )
    .expect("web scroll content (wider than viewport)");

    let expected_rel = WebRect {
        x: web_thumb_early.rect.x - web_root.rect.x,
        y: web_thumb_early.rect.y - web_root.rect.y,
        w: web_thumb_early.rect.w,
        h: web_thumb_early.rect.h,
    };

    let inset = web_viewport.rect.x - web_root.rect.x;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme_early.viewport.w), Px(theme_early.viewport.h)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let handle = ScrollHandle::default();
    let content_w = web_content.rect.w;
    let root_w = web_root.rect.w;
    let root_h = web_root.rect.h;

    let render = |cx: &mut fret_ui::ElementContext<'_, App>| {
        let handle = handle.clone();
        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:scroll-area-horizontal-demo:hover-out")),
                ..Default::default()
            },
            move |cx| {
                vec![cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(root_w));
                            layout.size.height = Length::Px(Px(root_h));
                            layout
                        },
                        padding: Edges::all(Px(inset)),
                        ..Default::default()
                    },
                    move |cx| {
                        let content = cx.container(
                            ContainerProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Px(Px(content_w));
                                    layout.size.height = Length::Fill;
                                    layout
                                },
                                ..Default::default()
                            },
                            |_cx| vec![],
                        );

                        let scroll_area =
                            fret_ui_shadcn::ScrollAreaRoot::new(
                                fret_ui_shadcn::ScrollAreaViewport::new(vec![content]),
                            )
                            .scroll_handle(handle.clone())
                            .scrollbar(fret_ui_shadcn::ScrollAreaScrollbar::new().orientation(
                                fret_ui_shadcn::ScrollAreaScrollbarOrientation::Vertical,
                            ))
                            .scrollbar(fret_ui_shadcn::ScrollAreaScrollbar::new().orientation(
                                fret_ui_shadcn::ScrollAreaScrollbarOrientation::Horizontal,
                            ))
                            .corner(true)
                            .refine_layout(LayoutRefinement::default().size_full())
                            .into_element(cx);

                        vec![scroll_area]
                    },
                )]
            },
        )]
    };

    macro_rules! render_at {
        ($frame:expr) => {{
            app.set_frame_id(FrameId($frame));
            let root_node = fret_ui::declarative::render_root(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                "web-vs-fret-layout",
                &render,
            );
            ui.set_root(root_node);
            ui.request_semantics_snapshot();
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
            ui.semantics_snapshot()
                .cloned()
                .expect("expected semantics snapshot")
        }};
    }

    let snap0 = render_at!(0);
    let panel0 = find_semantics(
        &snap0,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-horizontal-demo:hover-out"),
    )
    .expect("fret hover-out panel (initial)");
    let expected_abs0 = WebRect {
        x: panel0.bounds.origin.x.0 + expected_rel.x,
        y: panel0.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };
    let mut scene0 = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene0, 1.0);
    assert!(
        find_scene_quad_with_rect_close(&scene0, expected_abs0, 2.0).is_none(),
        "expected thumb quad to be absent before hover"
    );

    let hover_pos = Point::new(
        Px(panel0.bounds.origin.x.0 + (web_viewport.rect.x + web_viewport.rect.w * 0.5)),
        Px(panel0.bounds.origin.y.0 + (web_viewport.rect.y + web_viewport.rect.h * 0.5)),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: hover_pos,
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let snap1 = render_at!(1);
    let panel1 = find_semantics(
        &snap1,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-horizontal-demo:hover-out"),
    )
    .expect("fret hover-out panel (hovered)");
    let expected_abs1 = WebRect {
        x: panel1.bounds.origin.x.0 + expected_rel.x,
        y: panel1.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };
    let mut scene1 = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene1, 1.0);
    assert!(
        find_scene_quad_with_rect_close(&scene1, expected_abs1, 2.0).is_some(),
        "expected thumb quad to be present after hover"
    );

    let leave_pos = Point::new(Px(root_w + 100.0), Px(root_h + 100.0));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: leave_pos,
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let snap_leave = render_at!(2);
    let panel_leave = find_semantics(
        &snap_leave,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-horizontal-demo:hover-out"),
    )
    .expect("fret hover-out panel (leave)");
    let expected_abs_leave = WebRect {
        x: panel_leave.bounds.origin.x.0 + expected_rel.x,
        y: panel_leave.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };
    let mut scene_leave = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene_leave, 1.0);
    assert!(
        find_scene_quad_with_rect_close(&scene_leave, expected_abs_leave, 2.0).is_some(),
        "expected thumb quad to remain visible immediately after leave"
    );

    let mut snap_early: Option<fret_core::SemanticsSnapshot> = None;
    let mut snap_late: Option<fret_core::SemanticsSnapshot> = None;
    let mut scene_early: Option<Scene> = None;
    let mut scene_late: Option<Scene> = None;
    for frame in 3..=(2 + 39) {
        let snap = render_at!(frame);
        if frame == 2 + 33 {
            snap_early = Some(snap);
            let mut scene = Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
            scene_early = Some(scene);
        } else if frame == 2 + 39 {
            snap_late = Some(snap);
            let mut scene = Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
            scene_late = Some(scene);
        }
    }

    let snap_early = snap_early.expect("missing snap_early");
    let panel_early = find_semantics(
        &snap_early,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-horizontal-demo:hover-out"),
    )
    .expect("fret hover-out panel (early)");
    let expected_abs_early = WebRect {
        x: panel_early.bounds.origin.x.0 + expected_rel.x,
        y: panel_early.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };
    let scene_early = scene_early.expect("missing scene_early");
    assert!(
        find_scene_quad_with_rect_close(&scene_early, expected_abs_early, 2.0).is_some(),
        "expected thumb quad to remain visible before scrollHideDelay"
    );

    let snap_late = snap_late.expect("missing snap_late");
    let panel_late = find_semantics(
        &snap_late,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-horizontal-demo:hover-out"),
    )
    .expect("fret hover-out panel (late)");
    let expected_abs_late = WebRect {
        x: panel_late.bounds.origin.x.0 + expected_rel.x,
        y: panel_late.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };
    let scene_late = scene_late.expect("missing scene_late");
    assert!(
        find_scene_quad_with_rect_close(&scene_late, expected_abs_late, 2.0).is_none(),
        "expected thumb quad to be hidden after scrollHideDelay"
    );
}

fn web_vs_fret_layout_scroll_area_horizontal_demo_thumb_bounds_match_web_scrolled() {
    let web = read_web_golden("scroll-area-horizontal-demo.scrolled");
    let theme = web_theme(&web);
    let web_root =
        web_find_by_class_tokens(&theme.root, &["relative", "w-96", "rounded-md", "border"])
            .expect("web scroll area root");

    let web_viewport =
        web_find_by_data_slot(&theme.root, "scroll-area-viewport").expect("web scroll viewport");
    let web_scrollbar = web_find_scroll_area_scrollbar(&theme.root, "horizontal")
        .expect("web scroll-area-scrollbar (horizontal)");
    let web_thumb =
        web_find_scroll_area_thumb_in_scrollbar(web_scrollbar).expect("web scroll-area-thumb");
    let web_scroll = web_viewport
        .scroll
        .as_ref()
        .expect("web scroll viewport scroll metrics");

    assert!(
        (web_scroll.scroll_left - 80.0).abs() < 0.01,
        "expected scrollLeft=80 in golden"
    );

    assert!(
        web_scrollbar
            .attrs
            .get("data-state")
            .is_some_and(|v| v == "visible"),
        "expected web scrollbar to be visible in scrolled golden"
    );

    let web_content = web_find_best_by(
        web_viewport,
        &|n| n.tag == "div" && n.rect.w > web_viewport.rect.w + 1.0,
        &|n| -n.rect.w,
    )
    .expect("web scroll content (wider than viewport)");

    let expected_rel = WebRect {
        x: web_thumb.rect.x - web_root.rect.x,
        y: web_thumb.rect.y - web_root.rect.y,
        w: web_thumb.rect.w,
        h: web_thumb.rect.h,
    };

    let inset = web_viewport.rect.x - web_root.rect.x;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let handle = ScrollHandle::default();

    let render = |cx: &mut fret_ui::ElementContext<'_, App>| {
        let content = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Px(Px(web_content.rect.w));
                    layout.size.height = Length::Fill;
                    layout
                },
                ..Default::default()
            },
            |_cx| vec![],
        );

        let scroll_area =
            fret_ui_shadcn::ScrollAreaRoot::new(fret_ui_shadcn::ScrollAreaViewport::new(vec![
                content,
            ]))
            .scroll_handle(handle.clone())
            .scrollbar(
                fret_ui_shadcn::ScrollAreaScrollbar::new()
                    .orientation(fret_ui_shadcn::ScrollAreaScrollbarOrientation::Vertical),
            )
            .scrollbar(
                fret_ui_shadcn::ScrollAreaScrollbar::new()
                    .orientation(fret_ui_shadcn::ScrollAreaScrollbarOrientation::Horizontal),
            )
            .corner(true)
            .refine_layout(LayoutRefinement::default().size_full())
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:scroll-area-horizontal-demo:scrolled")),
                ..Default::default()
            },
            move |cx| {
                vec![cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(web_root.rect.w));
                            layout.size.height = Length::Px(Px(web_root.rect.h));
                            layout
                        },
                        padding: Edges::all(Px(inset)),
                        ..Default::default()
                    },
                    move |_cx| vec![scroll_area],
                )]
            },
        )]
    };

    let root_node = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        render,
    );
    ui.set_root(root_node);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap1 = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot (pre-hover)");

    let panel1 = find_semantics(
        &snap1,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-horizontal-demo:scrolled"),
    )
    .expect("fret scrolled panel (pre-hover)");

    let expected_abs_pre = WebRect {
        x: panel1.bounds.origin.x.0 + expected_rel.x,
        y: panel1.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };

    let mut scene1 = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene1, 1.0);
    assert!(
        find_scene_quad_with_rect_close(&scene1, expected_abs_pre, 2.0).is_none(),
        "expected thumb quad to be absent before hover"
    );

    let hover_pos = Point::new(
        Px(panel1.bounds.origin.x.0 + (web_viewport.rect.x + web_viewport.rect.w * 0.5)),
        Px(panel1.bounds.origin.y.0 + (web_viewport.rect.y + web_viewport.rect.h * 0.5)),
    );
    let move_event = Event::Pointer(PointerEvent::Move {
        position: hover_pos,
        buttons: MouseButtons::default(),
        modifiers: Modifiers::default(),
        pointer_id: PointerId(0),
        pointer_type: PointerType::Mouse,
    });
    ui.dispatch_event(&mut app, &mut services, &move_event);

    handle.set_offset(Point::new(Px(web_scroll.scroll_left), Px(0.0)));

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    let root_node = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        render,
    );
    ui.set_root(root_node);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap2 = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot (post-hover)");

    let panel2 = find_semantics(
        &snap2,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-horizontal-demo:scrolled"),
    )
    .expect("fret scrolled panel (post-hover)");

    let expected_abs = WebRect {
        x: panel2.bounds.origin.x.0 + expected_rel.x,
        y: panel2.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };

    let mut scene2 = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene2, 1.0);
    let thumb_bounds =
        find_scene_quad_with_rect_close(&scene2, expected_abs, 2.0).expect("fret thumb quad");
    assert_rect_close_px(
        "scroll-area-horizontal-demo thumb",
        thumb_bounds,
        expected_abs,
        2.0,
    );
}
