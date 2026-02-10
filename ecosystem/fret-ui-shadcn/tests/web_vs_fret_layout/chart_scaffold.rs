use super::chart::{
    web_find_chart_container, web_find_chart_curve, web_find_chart_grid, web_find_chart_x_axis,
};
use super::*;

#[derive(Debug, Clone, Deserialize)]
struct FixtureSuite<T> {
    schema_version: u32,
    cases: Vec<T>,
}

#[derive(Debug, Clone, Deserialize)]
struct LayoutChartScaffoldCase {
    id: String,
    web_name: String,
    gate_curve: bool,
}

fn assert_chart_scaffold_geometry_matches_web(web_name: &str, gate_curve: bool) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let web_chart = web_find_chart_container(&theme.root);
    let web_grid = web_find_chart_grid(web_chart);
    let web_x_axis = web_find_chart_x_axis(web_chart);
    let web_curve = web_find_chart_curve(web_chart);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let chart_label = Arc::<str>::from(format!("Golden:{web_name}:chart"));
    let grid_label = Arc::<str>::from(format!("Golden:{web_name}:grid"));
    let axis_label = Arc::<str>::from(format!("Golden:{web_name}:x-axis"));
    let curve_label = Arc::<str>::from(format!("Golden:{web_name}:curve"));

    let chart_label_out = chart_label.clone();
    let grid_label_out = grid_label.clone();
    let axis_label_out = axis_label.clone();
    let curve_label_out = curve_label.clone();

    let snap = run_fret_root(bounds, move |cx| {
        let chart = cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Px(Px(web_chart.rect.w)),
                        height: Length::Auto,
                        ..Default::default()
                    },
                    aspect_ratio: Some(16.0 / 9.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            move |cx| {
                let grid = {
                    let dx = web_grid.rect.x - web_chart.rect.x;
                    let dy = web_grid.rect.y - web_chart.rect.y;
                    let wrapper = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(grid_label.clone()),
                            layout: LayoutStyle {
                                position: fret_ui::element::PositionStyle::Absolute,
                                inset: fret_ui::element::InsetStyle {
                                    left: Some(Px(dx)),
                                    top: Some(Px(dy)),
                                    ..Default::default()
                                },
                                size: SizeStyle {
                                    width: Length::Px(Px(web_grid.rect.w)),
                                    height: Length::Px(Px(web_grid.rect.h)),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        move |cx| {
                            vec![cx.canvas(fret_ui::element::CanvasProps::default(), |_p| {})]
                        },
                    );
                    wrapper
                };

                let x_axis = {
                    let dx = web_x_axis.rect.x - web_chart.rect.x;
                    let dy = web_x_axis.rect.y - web_chart.rect.y;
                    cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(axis_label.clone()),
                            layout: LayoutStyle {
                                position: fret_ui::element::PositionStyle::Absolute,
                                inset: fret_ui::element::InsetStyle {
                                    left: Some(Px(dx)),
                                    top: Some(Px(dy)),
                                    ..Default::default()
                                },
                                size: SizeStyle {
                                    width: Length::Px(Px(web_x_axis.rect.w)),
                                    height: Length::Px(Px(web_x_axis.rect.h)),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        |cx| vec![cx.container(Default::default(), |_cx| Vec::new())],
                    )
                };

                let mut out = vec![grid, x_axis];
                if gate_curve {
                    if let Some(web_curve) = web_curve {
                        let dx = web_curve.rect.x - web_chart.rect.x;
                        let dy = web_curve.rect.y - web_chart.rect.y;
                        let curve = cx.semantics(
                            fret_ui::element::SemanticsProps {
                                role: SemanticsRole::Panel,
                                label: Some(curve_label.clone()),
                                layout: LayoutStyle {
                                    position: fret_ui::element::PositionStyle::Absolute,
                                    inset: fret_ui::element::InsetStyle {
                                        left: Some(Px(dx)),
                                        top: Some(Px(dy)),
                                        ..Default::default()
                                    },
                                    size: SizeStyle {
                                        width: Length::Px(Px(web_curve.rect.w)),
                                        height: Length::Px(Px(web_curve.rect.h)),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            move |cx| {
                                vec![cx.canvas(fret_ui::element::CanvasProps::default(), |_p| {})]
                            },
                        );
                        out.push(curve);
                    }
                }

                out
            },
        );

        let chart = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(chart_label.clone()),
                layout: LayoutStyle {
                    position: fret_ui::element::PositionStyle::Absolute,
                    inset: fret_ui::element::InsetStyle {
                        left: Some(Px(web_chart.rect.x)),
                        top: Some(Px(web_chart.rect.y)),
                        ..Default::default()
                    },
                    size: SizeStyle {
                        width: Length::Px(Px(web_chart.rect.w)),
                        height: Length::Auto,
                        ..Default::default()
                    },
                    aspect_ratio: Some(16.0 / 9.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            move |_cx| vec![chart],
        );

        vec![chart]
    });

    let chart = find_semantics(&snap, SemanticsRole::Panel, Some(&chart_label_out))
        .unwrap_or_else(|| panic!("missing fret chart semantics for {web_name}"));
    assert_rect_close_px(web_name, chart.bounds, web_chart.rect, 1.0);

    let grid = find_semantics(&snap, SemanticsRole::Panel, Some(&grid_label_out))
        .unwrap_or_else(|| panic!("missing fret chart grid semantics for {web_name}"));
    assert_rect_close_px(&format!("{web_name} grid"), grid.bounds, web_grid.rect, 1.0);

    let x_axis = find_semantics(&snap, SemanticsRole::Panel, Some(&axis_label_out))
        .unwrap_or_else(|| panic!("missing fret chart x axis semantics for {web_name}"));
    assert_rect_close_px(
        &format!("{web_name} x axis"),
        x_axis.bounds,
        web_x_axis.rect,
        1.0,
    );

    if gate_curve {
        if let Some(web_curve) = web_curve {
            let curve = find_semantics(&snap, SemanticsRole::Panel, Some(&curve_label_out))
                .unwrap_or_else(|| panic!("missing fret chart curve semantics for {web_name}"));
            assert_rect_close_px(
                &format!("{web_name} curve"),
                curve.bounds,
                web_curve.rect,
                1.0,
            );
        }
    }
}

#[test]
fn web_vs_fret_layout_chart_scaffold_geometry_matches_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/layout_chart_scaffold_cases_v1.json"
    ));
    let suite: FixtureSuite<LayoutChartScaffoldCase> =
        serde_json::from_str(raw).expect("layout chart scaffold fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!(
            "layout chart scaffold case={} web_name={}",
            case.id, case.web_name
        );
        assert_chart_scaffold_geometry_matches_web(&case.web_name, case.gate_curve);
    }
}
