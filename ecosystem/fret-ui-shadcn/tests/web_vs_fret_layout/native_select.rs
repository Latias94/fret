use super::*;
use fret_ui_shadcn::facade as shadcn;

#[derive(Debug, Clone, Deserialize)]
struct LayoutNativeSelectCase {
    id: String,
    web_name: String,
    label_text: String,
    #[serde(default)]
    disabled: bool,
    #[serde(default)]
    aria_invalid: bool,
}

#[test]
fn web_vs_fret_layout_native_select_heights_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/layout_native_select_cases_v1.json"
    ));
    let suite: FixtureSuite<LayoutNativeSelectCase> =
        serde_json::from_str(raw).expect("layout native select fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("layout native select case={}", case.id);
        let web = read_web_golden(&case.web_name);
        let theme = web_theme(&web);
        let web_select = find_first(&theme.root, &|n| n.tag == "select").expect("web select");

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
        );

        let snap = run_fret_root(bounds, |cx| {
            let mut select = shadcn::NativeSelect::new_controllable(cx, None, None, None, false)
                .a11y_label("NativeSelect")
                .placeholder(case.label_text.clone())
                .refine_layout(
                    LayoutRefinement::default().w_px(MetricRef::Px(Px(web_select.rect.w))),
                );
            if case.disabled {
                select = select.disabled(true);
            }
            if case.aria_invalid {
                select = select.aria_invalid(true);
            }

            vec![select.into_element(cx)]
        });

        let select = find_semantics(&snap, SemanticsRole::ComboBox, Some("NativeSelect"))
            .or_else(|| find_semantics(&snap, SemanticsRole::ComboBox, None))
            .expect("fret native select");

        assert_close_px(
            &format!("{} h", case.web_name),
            select.bounds.size.height,
            web_select.rect.h,
            1.0,
        );
    }
}

#[test]
fn web_vs_fret_layout_native_select_chevron_matches_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/layout_native_select_cases_v1.json"
    ));
    let suite: FixtureSuite<LayoutNativeSelectCase> =
        serde_json::from_str(raw).expect("layout native select fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("layout native select chevron case={}", case.id);
        let web = read_web_golden(&case.web_name);
        let theme = web_theme(&web);

        let wrapper = find_first(&theme.root, &|n| class_has_token(n, "group/native-select"))
            .expect("web native select wrapper");
        let web_select = find_first(wrapper, &|n| n.tag == "select").expect("web select");
        let web_icon = find_first(wrapper, &|n| {
            n.tag == "svg" && class_has_token(n, "lucide-chevron-down")
        })
        .expect("web chevron icon");

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
        );

        let snap = run_fret_root(bounds, |cx| {
            let mut select = shadcn::NativeSelect::new_controllable(cx, None, None, None, false)
                .a11y_label("NativeSelect")
                .placeholder(case.label_text.clone())
                .test_id_prefix("test-native-select")
                .options([shadcn::NativeSelectOption::new("alpha", "Alpha")])
                .refine_layout(
                    LayoutRefinement::default().w_px(MetricRef::Px(Px(web_select.rect.w))),
                );
            if case.disabled {
                select = select.disabled(true);
            }
            if case.aria_invalid {
                select = select.aria_invalid(true);
            }

            vec![select.into_element(cx)]
        });

        let icon = find_by_test_id(&snap, "test-native-select-icon");
        assert_rect_close_px(
            &format!("{} chevron", case.web_name),
            icon.bounds,
            web_icon.rect,
            1.0,
        );
    }
}
