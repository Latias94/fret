use super::*;

#[derive(Debug, Clone, Deserialize)]
struct LayoutShellContainerCenteredCase {
    id: String,
    web_name: String,
    container_class_tokens: Vec<String>,
}

fn assert_shell_container_centered_x_w_matches(web_name: &str, tokens: &[&str]) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);
    let web_container = web_find_by_class_tokens(&theme.root, tokens).expect("web shell container");
    let max_w = web_container.rect.w;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let label: Arc<str> = Arc::from(format!("Golden:{web_name}:container"));
    let label_str: &str = &label;
    let snap = run_fret_root(bounds, |cx| {
        vec![cx.flex(
            FlexProps {
                layout: decl_style::layout_style(
                    &Theme::global(&*cx.app),
                    LayoutRefinement::default().size_full().min_w_0(),
                ),
                direction: fret_core::Axis::Horizontal,
                gap: Px(0.0),
                padding: Edges::all(Px(40.0)),
                justify: MainAlign::Center,
                align: CrossAlign::Center,
                wrap: false,
            },
            {
                let label = label.clone();
                move |cx| {
                    vec![cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(label.clone()),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![
                                cx.container(
                                    ContainerProps {
                                        layout: decl_style::layout_style(
                                            &Theme::global(&*cx.app),
                                            LayoutRefinement::default()
                                                .w_px(MetricRef::Px(Px(max_w)))
                                                .min_w_0(),
                                        ),
                                        ..Default::default()
                                    },
                                    |_cx| Vec::new(),
                                ),
                            ]
                        },
                    )]
                }
            },
        )]
    });

    let fret_container =
        find_semantics(&snap, SemanticsRole::Panel, Some(label_str)).expect("fret container");
    assert_panel_x_w_match(
        web_name,
        "container",
        &fret_container.bounds,
        web_container.rect,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_shell_container_centered_x_w_matches_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/layout_shell_container_centered_cases_v1.json"
    ));
    let suite: FixtureSuite<LayoutShellContainerCenteredCase> =
        serde_json::from_str(raw).expect("layout shell container centered fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!(
            "layout shell container centered case={} web_name={}",
            case.id, case.web_name
        );
        let tokens: Vec<&str> = case
            .container_class_tokens
            .iter()
            .map(|s| s.as_str())
            .collect();
        assert_shell_container_centered_x_w_matches(&case.web_name, &tokens);
    }
}

fn assert_two_col_shell_container_x_w_matches(web_name: &str, container_tokens: &[&str]) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);
    let web_container =
        web_find_by_class_tokens(&theme.root, container_tokens).expect("web container");
    let max_w = web_container.rect.w;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let label: Arc<str> = Arc::from(format!("Golden:{web_name}:container"));
    let label_str: &str = &label;
    let col_w = theme.viewport.w / 2.0;
    let snap = run_fret_root(bounds, |cx| {
        let center = cx.flex(
            FlexProps {
                layout: decl_style::layout_style(
                    &Theme::global(&*cx.app),
                    LayoutRefinement::default().flex_1().min_w_0().min_h_0(),
                ),
                direction: fret_core::Axis::Horizontal,
                gap: Px(0.0),
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::Center,
                align: CrossAlign::Center,
                wrap: false,
            },
            {
                let label = label.clone();
                move |cx| {
                    vec![cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(label.clone()),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![
                                cx.container(
                                    ContainerProps {
                                        layout: decl_style::layout_style(
                                            &Theme::global(&*cx.app),
                                            LayoutRefinement::default()
                                                .w_px(MetricRef::Px(Px(max_w)))
                                                .min_w_0(),
                                        ),
                                        ..Default::default()
                                    },
                                    |_cx| Vec::new(),
                                ),
                            ]
                        },
                    )]
                }
            },
        );

        let left = cx.flex(
            FlexProps {
                layout: decl_style::layout_style(
                    &Theme::global(&*cx.app),
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(col_w)))
                        .h_full()
                        .min_w_0()
                        .min_h_0(),
                ),
                direction: fret_core::Axis::Vertical,
                gap: Px(16.0),
                padding: Edges::all(Px(40.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Stretch,
                wrap: false,
            },
            move |_cx| vec![center],
        );

        let right = cx.container(
            ContainerProps {
                layout: decl_style::layout_style(
                    &Theme::global(&*cx.app),
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(col_w)))
                        .h_full()
                        .min_w_0()
                        .min_h_0(),
                ),
                ..Default::default()
            },
            |_cx| Vec::new(),
        );

        vec![cx.flex(
            FlexProps {
                layout: decl_style::layout_style(
                    &Theme::global(&*cx.app),
                    LayoutRefinement::default().size_full().min_w_0().min_h_0(),
                ),
                direction: fret_core::Axis::Horizontal,
                gap: Px(0.0),
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Stretch,
                wrap: false,
            },
            move |_cx| vec![left, right],
        )]
    });

    let fret_container =
        find_semantics(&snap, SemanticsRole::Panel, Some(label_str)).expect("fret container");
    assert_panel_x_w_match(
        web_name,
        "container",
        &fret_container.bounds,
        web_container.rect,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_login_01_shell_container_matches() {
    assert_shell_container_centered_x_w_matches("login-01", &["w-full", "max-w-sm"]);
}

#[test]
fn web_vs_fret_layout_login_02_shell_container_matches() {
    assert_two_col_shell_container_x_w_matches("login-02", &["w-full", "max-w-xs"]);
}

#[test]
fn web_vs_fret_layout_signup_02_shell_container_matches() {
    assert_two_col_shell_container_x_w_matches("signup-02", &["w-full", "max-w-xs"]);
}

#[test]
fn web_vs_fret_layout_otp_02_shell_container_matches() {
    assert_two_col_shell_container_x_w_matches("otp-02", &["w-full", "max-w-xs"]);
}
