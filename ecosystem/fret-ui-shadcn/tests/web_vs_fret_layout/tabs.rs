use super::*;

#[test]
fn web_vs_fret_layout_tabs_demo_tab_list_height() {
    let web = read_web_golden("tabs-demo");
    let theme = web_theme(&web);
    let web_tab_list = web_find_by_class_tokens(
        &theme.root,
        &[
            "bg-muted",
            "text-muted-foreground",
            "inline-flex",
            "h-9",
            "w-fit",
        ],
    )
    .expect("web tab list");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let items = vec![
            fret_ui_shadcn::TabsItem::new("account", "Account", vec![cx.text("Panel")]),
            fret_ui_shadcn::TabsItem::new("password", "Password", vec![cx.text("Panel")]),
        ];

        vec![
            fret_ui_shadcn::Tabs::uncontrolled(Some("account"))
                .items(items)
                .into_element(cx),
        ]
    });

    let tab_list = find_semantics(&snap, SemanticsRole::TabList, None).expect("fret tab list");
    assert_close_px(
        "tab list height",
        tab_list.bounds.size.height,
        web_tab_list.rect.h,
        1.0,
    );
}
#[test]
fn web_vs_fret_layout_tabs_demo_active_tab_height() {
    let web = read_web_golden("tabs-demo");
    let theme = web_theme(&web);
    let web_active_tab = find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|r| r == "tab")
            && n.attrs.get("aria-selected").is_some_and(|v| v == "true")
            && contains_text(n, "Account")
    })
    .expect("web active tab");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let items = vec![
            fret_ui_shadcn::TabsItem::new("account", "Account", vec![cx.text("Panel")]),
            fret_ui_shadcn::TabsItem::new("password", "Password", vec![cx.text("Panel")]),
        ];

        vec![
            fret_ui_shadcn::Tabs::uncontrolled(Some("account"))
                .items(items)
                .into_element(cx),
        ]
    });

    let tab = find_semantics(&snap, SemanticsRole::Tab, Some("Account"))
        .expect("fret active tab semantics node");

    assert_close_px(
        "tab height",
        tab.bounds.size.height,
        web_active_tab.rect.h,
        1.0,
    );
}
#[test]
fn web_vs_fret_layout_tabs_demo_inactive_tab_text_color_matches_web() {
    let web = read_web_golden("tabs-demo");
    let theme = web_theme(&web);
    let web_inactive_tab = find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|r| r == "tab")
            && n.attrs.get("aria-selected").is_some_and(|v| v == "false")
            && contains_text(n, "Password")
    })
    .expect("web inactive tab");
    let expected = web_inactive_tab
        .computed_style
        .get("color")
        .and_then(|s| parse_css_color(s))
        .expect("web inactive tab computedStyle.color");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let (snap, scene) = render_and_paint_in_bounds(bounds, |cx| {
        let items = vec![
            fret_ui_shadcn::TabsItem::new("account", "Account", vec![cx.text("Panel")]),
            fret_ui_shadcn::TabsItem::new("password", "Password", vec![cx.text("Panel")]),
        ];

        vec![
            fret_ui_shadcn::Tabs::uncontrolled(Some("account"))
                .items(items)
                .into_element(cx),
        ]
    });

    let tab = find_semantics(&snap, SemanticsRole::Tab, Some("Password"))
        .expect("fret inactive tab semantics node");

    let mut actual: Option<Rgba> = None;
    for op in scene.ops() {
        if let SceneOp::Text { origin, color, .. } = *op
            && tab.bounds.contains(origin)
        {
            actual = Some(color_to_rgba(color));
            break;
        }
    }
    let actual = actual.expect("fret inactive tab text color");
    assert_rgba_close("inactive tab text color", actual, expected, 0.06);
}
#[test]
fn web_vs_fret_layout_tabs_demo_active_tab_text_color_matches_web() {
    let web = read_web_golden("tabs-demo");
    let theme = web_theme(&web);
    let web_active_tab = find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|r| r == "tab")
            && n.attrs.get("aria-selected").is_some_and(|v| v == "true")
            && contains_text(n, "Account")
    })
    .expect("web active tab");
    let expected = web_active_tab
        .computed_style
        .get("color")
        .and_then(|s| parse_css_color(s))
        .expect("web active tab computedStyle.color");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let (snap, scene) = render_and_paint_in_bounds(bounds, |cx| {
        let items = vec![
            fret_ui_shadcn::TabsItem::new("account", "Account", vec![cx.text("Panel")]),
            fret_ui_shadcn::TabsItem::new("password", "Password", vec![cx.text("Panel")]),
        ];

        vec![
            fret_ui_shadcn::Tabs::uncontrolled(Some("account"))
                .items(items)
                .into_element(cx),
        ]
    });

    let tab = find_semantics(&snap, SemanticsRole::Tab, Some("Account"))
        .expect("fret active tab semantics node");

    let mut actual: Option<Rgba> = None;
    for op in scene.ops() {
        if let SceneOp::Text { origin, color, .. } = *op
            && tab.bounds.contains(origin)
        {
            actual = Some(color_to_rgba(color));
            break;
        }
    }
    let actual = actual.expect("fret active tab text color");
    assert_rgba_close("active tab text color", actual, expected, 0.06);
}
#[test]
fn web_vs_fret_layout_tabs_demo_active_tab_inset_matches_web() {
    let web = read_web_golden("tabs-demo");
    let theme = web_theme(&web);
    let web_tab_list = find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|r| r == "tablist")
    })
    .expect("web tablist role");
    let web_active_tab = find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|r| r == "tab")
            && n.attrs.get("aria-selected").is_some_and(|v| v == "true")
            && contains_text(n, "Account")
    })
    .expect("web active tab");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let items = vec![
            fret_ui_shadcn::TabsItem::new("account", "Account", vec![cx.text("Panel")]),
            fret_ui_shadcn::TabsItem::new("password", "Password", vec![cx.text("Panel")]),
        ];

        vec![
            fret_ui_shadcn::Tabs::uncontrolled(Some("account"))
                .items(items)
                .into_element(cx),
        ]
    });

    let active_tab =
        find_semantics(&snap, SemanticsRole::Tab, Some("Account")).expect("fret active tab");
    let tab_list = {
        let mut parent = active_tab.parent;
        let mut out = None;
        while let Some(pid) = parent {
            let p = snap
                .nodes
                .iter()
                .find(|n| n.id == pid)
                .expect("semantics parent node");
            if p.role == SemanticsRole::TabList {
                out = Some(p);
                break;
            }
            parent = p.parent;
        }
        out.expect("fret tab list ancestor")
    };

    let web_dx = web_active_tab.rect.x - web_tab_list.rect.x;
    let web_dy = web_active_tab.rect.y - web_tab_list.rect.y;
    let fret_dx = active_tab.bounds.origin.x.0 - tab_list.bounds.origin.x.0;
    let fret_dy = active_tab.bounds.origin.y.0 - tab_list.bounds.origin.y.0;

    if std::env::var_os("FRET_TEST_DEBUG_TABS").is_some() {
        eprintln!("web tablist: {:?}", web_tab_list.rect);
        eprintln!("web active tab: {:?}", web_active_tab.rect);
        eprintln!("web inset: ({web_dx:.3}, {web_dy:.3})");
        eprintln!("fret tablist: {:?}", tab_list.bounds);
        eprintln!("fret active tab: {:?}", active_tab.bounds);
        eprintln!("fret inset: ({fret_dx:.3}, {fret_dy:.3})");

        eprintln!("fret tablist ancestors for active tab:");
        let mut parent = active_tab.parent;
        while let Some(pid) = parent {
            let p = snap
                .nodes
                .iter()
                .find(|n| n.id == pid)
                .expect("semantics parent node");
            eprintln!(
                "  - {:?} label={:?} bounds={:?}",
                p.role,
                p.label.as_deref(),
                p.bounds
            );
            parent = p.parent;
        }

        eprintln!("fret tablists:");
        for n in snap
            .nodes
            .iter()
            .filter(|n| n.role == SemanticsRole::TabList)
        {
            eprintln!("  - label={:?} bounds={:?}", n.label.as_deref(), n.bounds);
        }
        eprintln!("fret tabs:");
        for n in snap.nodes.iter().filter(|n| n.role == SemanticsRole::Tab) {
            eprintln!(
                "  - label={:?} selected={} bounds={:?} parent={:?}",
                n.label.as_deref(),
                n.flags.selected,
                n.bounds,
                n.parent
            );
        }
    }

    assert_close_px("active tab inset x", Px(fret_dx), web_dx, 1.0);
    assert_close_px("active tab inset y", Px(fret_dy), web_dy, 1.0);
}
#[test]
fn web_vs_fret_layout_tabs_demo_panel_gap() {
    let web = read_web_golden("tabs-demo");
    let theme = web_theme(&web);
    let web_tab_list = find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|r| r == "tablist")
    })
    .expect("web tablist role");
    let web_panel = find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|r| r == "tabpanel")
    })
    .expect("web tabpanel role");

    let web_gap_y = web_panel.rect.y - (web_tab_list.rect.y + web_tab_list.rect.h);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let items = vec![
            fret_ui_shadcn::TabsItem::new("account", "Account", vec![cx.text("Panel")]),
            fret_ui_shadcn::TabsItem::new("password", "Password", vec![cx.text("Panel")]),
        ];

        vec![
            fret_ui_shadcn::Tabs::uncontrolled(Some("account"))
                .items(items)
                .into_element(cx),
        ]
    });

    let tab_list = find_semantics(&snap, SemanticsRole::TabList, None).expect("fret tab list");
    let panel = find_semantics(&snap, SemanticsRole::TabPanel, None).expect("fret tab panel");

    let fret_gap_y =
        panel.bounds.origin.y.0 - (tab_list.bounds.origin.y.0 + tab_list.bounds.size.height.0);

    assert_close_px("tab panel gap", Px(fret_gap_y), web_gap_y, 1.0);
}
