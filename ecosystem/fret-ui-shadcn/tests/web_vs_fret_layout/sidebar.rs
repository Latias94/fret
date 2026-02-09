use super::*;

#[derive(Debug, Clone, Deserialize)]
struct LayoutSidebarMenuButtonHeightCase {
    id: String,
    web_name: String,
}

fn web_find_sidebar_menu_button_by_height<'a>(
    root: &'a WebNode,
    height_token: &str,
) -> Option<&'a WebNode> {
    find_first(root, &|n| {
        (n.tag == "button" || n.tag == "a")
            && class_has_token(n, "peer/menu-button")
            && class_has_token(n, height_token)
    })
}

fn assert_sidebar_menu_button_heights_match_web(web_name: &str) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let web_default = web_find_sidebar_menu_button_by_height(&theme.root, "h-8")
        .unwrap_or_else(|| panic!("missing web sidebar menu button (h-8) in {web_name}"));
    let web_lg = web_find_sidebar_menu_button_by_height(&theme.root, "h-12");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap_default = run_fret_root(bounds, |cx| {
        vec![
            fret_ui_shadcn::SidebarMenuButton::new("Sidebar Menu Button")
                .size(SidebarMenuButtonSize::Default)
                .into_element(cx),
        ]
    });

    let fret_default = find_semantics(
        &snap_default,
        SemanticsRole::Button,
        Some("Sidebar Menu Button"),
    )
    .or_else(|| find_semantics(&snap_default, SemanticsRole::Button, None))
    .expect("fret sidebar menu button (default) semantics node");

    assert_close_px(
        &format!("{web_name} menu button height (h-8)"),
        fret_default.bounds.size.height,
        web_default.rect.h,
        1.0,
    );

    if let Some(web_lg) = web_lg {
        let collapsed = (web_lg.rect.h - 32.0).abs() <= 1.0;
        let snap_lg = run_fret_root(bounds, |cx| {
            vec![
                fret_ui_shadcn::SidebarMenuButton::new("Sidebar Menu Button")
                    .size(SidebarMenuButtonSize::Lg)
                    .collapsed(collapsed)
                    .into_element(cx),
            ]
        });

        let fret_lg = find_semantics(&snap_lg, SemanticsRole::Button, Some("Sidebar Menu Button"))
            .or_else(|| find_semantics(&snap_lg, SemanticsRole::Button, None))
            .expect("fret sidebar menu button (lg) semantics node");

        assert_close_px(
            &format!("{web_name} menu button height (h-12)"),
            fret_lg.bounds.size.height,
            web_lg.rect.h,
            1.0,
        );
    }
}

#[test]
fn web_vs_fret_layout_sidebar_menu_button_heights_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/layout_sidebar_menu_button_height_cases_v1.json"
    ));
    let suite: FixtureSuite<LayoutSidebarMenuButtonHeightCase> =
        serde_json::from_str(raw).expect("layout sidebar menu button height fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!(
            "layout sidebar menu button height case={} web_name={}",
            case.id, case.web_name
        );
        assert_sidebar_menu_button_heights_match_web(&case.web_name);
    }
}
