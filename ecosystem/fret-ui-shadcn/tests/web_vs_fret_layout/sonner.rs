use super::*;

#[test]
fn web_vs_fret_layout_sonner_demo_button_height_matches_web() {
    let web = read_web_golden("sonner-demo");
    let theme = web_theme(&web);
    let web_button =
        web_find_by_tag_and_text(&theme.root, "button", "Show Toast").expect("web button");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        vec![
            fret_ui_shadcn::Button::new("Show Toast")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .into_element(cx),
        ]
    });

    let button = find_semantics(&snap, SemanticsRole::Button, Some("Show Toast"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
        .expect("fret button");

    assert_close_px(
        "sonner-demo button h",
        button.bounds.size.height,
        web_button.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_sonner_types_first_button_height_matches_web() {
    let web = read_web_golden("sonner-types");
    let theme = web_theme(&web);
    let web_button =
        web_find_by_tag_and_text(&theme.root, "button", "Default").expect("web button");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        vec![
            fret_ui_shadcn::Button::new("Default")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .into_element(cx),
        ]
    });

    let button = find_semantics(&snap, SemanticsRole::Button, Some("Default"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
        .expect("fret button");

    assert_close_px(
        "sonner-types button h",
        button.bounds.size.height,
        web_button.rect.h,
        1.0,
    );
}
