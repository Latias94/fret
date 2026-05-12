use super::*;

#[test]
fn preedit_rich_text_inserts_and_underlines() {
    let preedit = PreeditState {
        text: "世界".to_string(),
        cursor: Some((0, "世".len())),
    };
    let fg = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    let selection_bg = Color {
        r: 0.2,
        g: 0.2,
        b: 0.2,
        a: 1.0,
    };

    let shaping = fret_core::TextShapingStyle::default();
    let rich = paint::materialize_preedit_rich_text(
        "hello".into(),
        2,
        &shaping,
        &preedit,
        fg,
        selection_bg,
    );
    assert_eq!(rich.text.as_ref(), "he世界llo");
    assert!(rich.is_valid());
    assert!(
        rich.spans.iter().any(|s| s.paint.underline.is_some()),
        "expected preedit spans to be underlined"
    );
    assert!(
        rich.spans.iter().any(|s| s.paint.bg.is_some()),
        "expected cursor range to be highlighted"
    );
}

#[test]
fn preedit_rich_text_applies_code_shaping_to_all_spans() {
    let preedit = PreeditState {
        text: "ab".to_string(),
        cursor: None,
    };
    let fg = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    let selection_bg = Color {
        r: 0.2,
        g: 0.2,
        b: 0.2,
        a: 1.0,
    };

    let shaping = fret_core::TextShapingStyle::default()
        .with_feature("liga", 0)
        .with_feature("calt", 0);
    let rich = paint::materialize_preedit_rich_text(
        "hello".into(),
        2,
        &shaping,
        &preedit,
        fg,
        selection_bg,
    );
    assert!(rich.is_valid());
    assert!(
        rich.spans.iter().all(|s| s
            .shaping
            .features
            .iter()
            .any(|f| f.tag == "liga" && f.value == 0)),
        "expected `liga=0` to be applied to every span"
    );
    assert!(
        rich.spans.iter().all(|s| s
            .shaping
            .features
            .iter()
            .any(|f| f.tag == "calt" && f.value == 0)),
        "expected `calt=0` to be applied to every span"
    );
}
