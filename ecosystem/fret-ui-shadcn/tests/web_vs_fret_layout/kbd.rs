use super::*;

#[derive(Debug, Clone, Deserialize)]
struct LayoutKbdHeightCase {
    id: String,
    web_name: String,
    text: String,
}

fn assert_kbd_first_height_matches_web(web_name: &str, text: &str) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let web_kbd = find_first(&theme.root, &|n| n.tag == "kbd").expect("web kbd");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let label = format!("Golden:{web_name}:kbd");
    let snap = run_fret_root(bounds, |cx| {
        let kbd = fret_ui_shadcn::Kbd::new(text).into_element(cx);
        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from(label.clone())),
                ..Default::default()
            },
            move |_cx| vec![kbd],
        )]
    });

    let kbd = find_semantics(&snap, SemanticsRole::Panel, Some(&label)).expect("fret kbd");

    assert_close_px("kbd height", kbd.bounds.size.height, web_kbd.rect.h, 1.0);
}

#[test]
fn web_vs_fret_layout_kbd_heights_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/layout_kbd_height_cases_v1.json"
    ));
    let suite: FixtureSuite<LayoutKbdHeightCase> =
        serde_json::from_str(raw).expect("layout kbd height fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!(
            "layout kbd height case={} web_name={}",
            case.id, case.web_name
        );
        assert_kbd_first_height_matches_web(&case.web_name, &case.text);
    }
}

#[test]
fn web_vs_fret_layout_kbd_tooltip_kbd_height_matches_web() {
    let web = read_web_golden("kbd-tooltip");
    let theme = web_theme(&web);
    let web_button = web_find_by_tag_and_text(&theme.root, "button", "Save").expect("web button");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        vec![
            fret_ui_shadcn::Button::new("Save")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .size(fret_ui_shadcn::ButtonSize::Sm)
                .into_element(cx),
        ]
    });

    let button = find_semantics(&snap, SemanticsRole::Button, Some("Save"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
        .expect("fret button");

    assert_close_px(
        "kbd-tooltip button h",
        button.bounds.size.height,
        web_button.rect.h,
        1.0,
    );
}
