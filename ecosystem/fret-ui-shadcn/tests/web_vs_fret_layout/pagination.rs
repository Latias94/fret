use super::*;

#[test]
fn web_vs_fret_layout_pagination_demo_active_link_size_matches_web() {
    let web = read_web_golden("pagination-demo");
    let theme = web_theme(&web);
    let web_active = web_find_by_tag_and_text(&theme.root, "a", "2").expect("web active link");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let link = fret_ui_shadcn::PaginationLink::new(vec![ui::text(cx, "2").into_element(cx)])
            .active(true)
            .into_element(cx);
        let link = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:pagination-demo:active")),
                ..Default::default()
            },
            move |_cx| vec![link],
        );

        vec![link]
    });

    let active = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:pagination-demo:active"),
    )
    .expect("fret active pagination link");

    assert_close_px(
        "pagination-demo active w",
        active.bounds.size.width,
        web_active.rect.w,
        1.0,
    );
    assert_close_px(
        "pagination-demo active h",
        active.bounds.size.height,
        web_active.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_pagination_demo_inactive_link_size_matches_web() {
    let web = read_web_golden("pagination-demo");
    let theme = web_theme(&web);
    let web_inactive = web_find_by_tag_and_text(&theme.root, "a", "1").expect("web inactive link");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let link = fret_ui_shadcn::PaginationLink::new(vec![ui::text(cx, "1").into_element(cx)])
            .active(false)
            .into_element(cx);
        let link = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:pagination-demo:inactive")),
                ..Default::default()
            },
            move |_cx| vec![link],
        );

        vec![link]
    });

    let inactive = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:pagination-demo:inactive"),
    )
    .expect("fret inactive pagination link");

    assert_close_px(
        "pagination-demo inactive w",
        inactive.bounds.size.width,
        web_inactive.rect.w,
        1.0,
    );
    assert_close_px(
        "pagination-demo inactive h",
        inactive.bounds.size.height,
        web_inactive.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_pagination_demo_ellipsis_size_matches_web() {
    let web = read_web_golden("pagination-demo");
    let theme = web_theme(&web);
    let tokens = &["flex", "size-9", "items-center", "justify-center"];
    let web_ellipsis = find_first(&theme.root, &|n| {
        n.tag == "span"
            && n.attrs.get("aria-hidden").is_some_and(|v| v == "true")
            && class_has_all_tokens(n, tokens)
    })
    .expect("web ellipsis");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let ellipsis = fret_ui_shadcn::PaginationEllipsis::new().into_element(cx);
        let ellipsis = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:pagination-demo:ellipsis")),
                ..Default::default()
            },
            move |_cx| vec![ellipsis],
        );

        vec![ellipsis]
    });

    let ellipsis = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:pagination-demo:ellipsis"),
    )
    .expect("fret pagination ellipsis");

    assert_close_px(
        "pagination-demo ellipsis w",
        ellipsis.bounds.size.width,
        web_ellipsis.rect.w,
        1.0,
    );
    assert_close_px(
        "pagination-demo ellipsis h",
        ellipsis.bounds.size.height,
        web_ellipsis.rect.h,
        1.0,
    );
}
