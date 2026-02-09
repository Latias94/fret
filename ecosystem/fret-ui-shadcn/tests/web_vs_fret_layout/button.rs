use super::*;

#[test]
fn web_vs_fret_layout_button_as_child_geometry_matches_web() {
    let web = read_web_golden("button-as-child");
    let theme = web_theme(&web);
    let web_link = web_find_by_tag_and_text(&theme.root, "a", "Login").expect("web link");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        vec![fret_ui_shadcn::Button::new("Login").into_element(cx)]
    });

    let button = find_semantics(&snap, SemanticsRole::Button, Some("Login"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
        .expect("fret button");

    assert_close_px(
        "button-as-child w",
        button.bounds.size.width,
        web_link.rect.w,
        4.0,
    );
    assert_close_px(
        "button-as-child h",
        button.bounds.size.height,
        web_link.rect.h,
        1.0,
    );
}
