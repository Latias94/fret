use super::*;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum LayoutRadioGroupRecipe {
    RowGeometry,
    IndicatorOffset,
}

#[derive(Debug, Clone, Deserialize)]
struct LayoutRadioGroupCase {
    id: String,
    web_name: String,
    recipe: LayoutRadioGroupRecipe,
}

#[test]
fn web_vs_fret_layout_radio_group_demo_geometry_matches_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/layout_radio_group_cases_v1.json"
    ));
    let suite: FixtureSuite<LayoutRadioGroupCase> =
        serde_json::from_str(raw).expect("layout radio group fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("layout radio group case={}", case.id);
        let web = read_web_golden(&case.web_name);
        let theme = web_theme(&web);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
        );

        match case.recipe {
            LayoutRadioGroupRecipe::RowGeometry => {
                let mut rows: Vec<&WebNode> = Vec::new();
                let mut stack = vec![&theme.root];
                while let Some(node) = stack.pop() {
                    let class_name = node.class_name.as_deref().unwrap_or_default();
                    if node.tag == "div"
                        && class_name.contains("flex")
                        && class_name.contains("items-center")
                        && class_name.contains("gap-3")
                        && node
                            .children
                            .iter()
                            .any(|c| c.attrs.get("role").is_some_and(|role| role == "radio"))
                    {
                        rows.push(node);
                    }

                    for child in node.children.iter().rev() {
                        stack.push(child);
                    }
                }

                rows.sort_by(|a, b| {
                    a.rect
                        .y
                        .partial_cmp(&b.rect.y)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                assert!(
                    rows.len() >= 2,
                    "expected at least two radio-group rows in web golden"
                );

                let web_row0 = rows[0];
                let web_row1 = rows[1];

                let web_row_h = web_row0.rect.h;
                let web_gap_y = web_row1.rect.y - (web_row0.rect.y + web_row0.rect.h);

                let snap = run_fret_root(bounds, |cx| {
                    let items = vec![
                        fret_ui_shadcn::RadioGroupItem::new("default", "Default"),
                        fret_ui_shadcn::RadioGroupItem::new("comfortable", "Comfortable"),
                        fret_ui_shadcn::RadioGroupItem::new("compact", "Compact"),
                    ];

                    let group = items.into_iter().fold(
                        fret_ui_shadcn::RadioGroup::uncontrolled(Some("comfortable"))
                            .a11y_label("Options"),
                        |group, item| group.item(item),
                    );

                    vec![group.into_element(cx)]
                });

                let fret_row0 = find_semantics(&snap, SemanticsRole::RadioButton, Some("Default"))
                    .expect("fret radio row Default");
                let fret_row1 =
                    find_semantics(&snap, SemanticsRole::RadioButton, Some("Comfortable"))
                        .expect("fret radio row Comfortable");

                let fret_row_h = fret_row0.bounds.size.height.0;
                let fret_gap_y = fret_row1.bounds.origin.y.0
                    - (fret_row0.bounds.origin.y.0 + fret_row0.bounds.size.height.0);

                assert!(
                    fret_gap_y.is_finite(),
                    "expected finite fret gap; got={fret_gap_y}"
                );

                assert_close_px("radio-group row height", Px(fret_row_h), web_row_h, 1.0);
                assert_close_px("radio-group row gap", Px(fret_gap_y), web_gap_y, 1.0);
            }
            LayoutRadioGroupRecipe::IndicatorOffset => {
                let web_radio = find_first(&theme.root, &|n| {
                    n.tag == "button"
                        && n.attrs.get("role").is_some_and(|r| r == "radio")
                        && n.attrs.get("aria-checked").is_some_and(|v| v == "true")
                })
                .expect("web checked radio");
                let web_indicator = find_first(web_radio, &|n| {
                    n.tag == "svg" && (n.rect.w - 8.0).abs() <= 0.2
                })
                .expect("web radio indicator svg");

                let expected_dx = web_indicator.rect.x - web_radio.rect.x;
                let expected_dy = web_indicator.rect.y - web_radio.rect.y;

                let (snap, scene) = render_and_paint_in_bounds(bounds, |cx| {
                    let items = vec![
                        fret_ui_shadcn::RadioGroupItem::new("default", "Default"),
                        fret_ui_shadcn::RadioGroupItem::new("comfortable", "Comfortable"),
                        fret_ui_shadcn::RadioGroupItem::new("compact", "Compact"),
                    ];

                    let group = items.into_iter().fold(
                        fret_ui_shadcn::RadioGroup::uncontrolled(Some("comfortable"))
                            .a11y_label("Options"),
                        |group, item| group.item(item),
                    );

                    vec![group.into_element(cx)]
                });

                let row = find_semantics(&snap, SemanticsRole::RadioButton, Some("Comfortable"))
                    .expect("fret comfortable row");
                let row_bounds = row.bounds;

                let mut icon_rect: Option<Rect> = None;
                let mut best_icon_score = 0.0f32;
                let mut dot_rect: Option<Rect> = None;
                let mut best_dot_score = 0.0f32;

                for op in scene.ops() {
                    let SceneOp::Quad { rect, .. } = op else {
                        continue;
                    };

                    let score = overlap_area(*rect, row_bounds);
                    if score <= 0.0 {
                        continue;
                    }

                    let is_icon = (rect.size.width.0 - 16.0).abs() <= 0.2
                        && (rect.size.height.0 - 16.0).abs() <= 0.2;
                    if is_icon && score > best_icon_score {
                        best_icon_score = score;
                        icon_rect = Some(*rect);
                    }
                }

                let icon = icon_rect.expect("missing radio icon quad");

                for op in scene.ops() {
                    let SceneOp::Quad { rect, .. } = op else {
                        continue;
                    };

                    let score_icon = overlap_area(*rect, icon);
                    if score_icon <= 0.0 {
                        continue;
                    }

                    let is_dot = (rect.size.width.0 - 8.0).abs() <= 0.2
                        && (rect.size.height.0 - 8.0).abs() <= 0.2;
                    if is_dot && score_icon > best_dot_score {
                        best_dot_score = score_icon;
                        dot_rect = Some(*rect);
                    }
                }

                let dot = dot_rect.expect("missing radio indicator dot quad");

                assert_close_px(
                    "radio indicator offset x",
                    Px(dot.origin.x.0 - icon.origin.x.0),
                    expected_dx,
                    1.0,
                );
                assert_close_px(
                    "radio indicator offset y",
                    Px(dot.origin.y.0 - icon.origin.y.0),
                    expected_dy,
                    1.0,
                );
            }
        }
    }
}
