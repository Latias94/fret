use super::*;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum LayoutSwitchRecipe {
    TrackSize,
    ThumbOffsetUnchecked,
}

#[derive(Debug, Clone, Deserialize)]
struct LayoutSwitchCase {
    id: String,
    web_name: String,
    recipe: LayoutSwitchRecipe,
}

#[test]
fn web_vs_fret_layout_switch_demo_geometry_matches_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/layout_switch_cases_v1.json"
    ));
    let suite: FixtureSuite<LayoutSwitchCase> =
        serde_json::from_str(raw).expect("layout switch fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("layout switch case={}", case.id);
        let web = read_web_golden(&case.web_name);
        let theme = web_theme(&web);
        let web_switch = find_first(&theme.root, &|n| {
            n.tag == "button"
                && n.attrs.get("role").is_some_and(|r| r == "switch")
                && n.attrs.get("aria-checked").is_some_and(|v| v == "false")
        })
        .expect("web switch");

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
        );

        match case.recipe {
            LayoutSwitchRecipe::TrackSize => {
                let snap = run_fret_root(bounds, |cx| {
                    let model: Model<bool> = cx.app.models_mut().insert(false);
                    vec![
                        fret_ui_shadcn::Switch::new(model)
                            .a11y_label("Switch")
                            .into_element(cx),
                    ]
                });

                let switch = find_semantics(&snap, SemanticsRole::Switch, Some("Switch"))
                    .or_else(|| find_semantics(&snap, SemanticsRole::Switch, None))
                    .expect("fret switch semantics node");

                assert_close_px(
                    "switch width",
                    switch.bounds.size.width,
                    web_switch.rect.w,
                    1.0,
                );
                assert_close_px(
                    "switch height",
                    switch.bounds.size.height,
                    web_switch.rect.h,
                    1.0,
                );
            }
            LayoutSwitchRecipe::ThumbOffsetUnchecked => {
                let web_thumb = find_first(web_switch, &|n| {
                    n.tag == "span"
                        && n.attrs
                            .get("data-state")
                            .is_some_and(|state| state == "unchecked")
                        && (n.rect.w - 16.0).abs() <= 0.2
                        && (n.rect.h - 16.0).abs() <= 0.2
                })
                .expect("web switch thumb");

                let (snap, scene) = render_and_paint_in_bounds(bounds, |cx| {
                    let model: Model<bool> = cx.app.models_mut().insert(false);
                    vec![
                        fret_ui_shadcn::Switch::new(model)
                            .a11y_label("Switch")
                            .into_element(cx),
                    ]
                });

                let switch = find_semantics(&snap, SemanticsRole::Switch, Some("Switch"))
                    .or_else(|| find_semantics(&snap, SemanticsRole::Switch, None))
                    .expect("fret switch semantics");
                let switch_bounds = switch.bounds;

                let mut track_rect: Option<Rect> = None;
                let mut best_track_score = 0.0f32;
                let mut thumb_rect: Option<Rect> = None;
                let mut best_thumb_score = 0.0f32;

                for op in scene.ops() {
                    let SceneOp::Quad {
                        rect, background, ..
                    } = op
                    else {
                        continue;
                    };

                    // Ignore low-alpha shadow quads. The switch thumb/track are fully opaque in shadcn-web.
                    if background.a < 0.5 {
                        continue;
                    }

                    let score = overlap_area(*rect, switch_bounds);
                    if score <= 0.0 {
                        continue;
                    }

                    let is_track = (rect.size.width.0 - switch_bounds.size.width.0).abs() <= 1.0
                        && (rect.size.height.0 - switch_bounds.size.height.0).abs() <= 1.0;
                    if is_track && score > best_track_score {
                        best_track_score = score;
                        track_rect = Some(*rect);
                    }

                    let is_thumb = (rect.size.width.0 - 16.0).abs() <= 0.2
                        && (rect.size.height.0 - 16.0).abs() <= 0.2;
                    if is_thumb && score > best_thumb_score {
                        best_thumb_score = score;
                        thumb_rect = Some(*rect);
                    }
                }

                let track = track_rect.expect("missing switch track quad");
                let thumb = thumb_rect.expect("missing switch thumb quad");

                let expected_dx = web_thumb.rect.x - web_switch.rect.x;
                let expected_dy = web_thumb.rect.y - web_switch.rect.y;

                assert_close_px(
                    "switch thumb offset x",
                    Px(thumb.origin.x.0 - track.origin.x.0),
                    expected_dx,
                    1.0,
                );
                assert_close_px(
                    "switch thumb offset y",
                    Px(thumb.origin.y.0 - track.origin.y.0),
                    expected_dy,
                    1.0,
                );
            }
        }
    }
}
