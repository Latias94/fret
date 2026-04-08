use super::*;
use fret_ui_shadcn::facade as shadcn;

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
        vec![shadcn::Button::new("Login").into_element(cx)]
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

#[test]
fn web_vs_fret_layout_button_link_geometry_matches_web() {
    let web = read_web_golden("button-link");
    let theme = web_theme(&web);
    let web_button = web_find_by_tag_and_text(&theme.root, "button", "Link").expect("web button");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        vec![
            shadcn::Button::new("Link")
                .variant(shadcn::ButtonVariant::Link)
                .into_element(cx),
        ]
    });

    let button = find_semantics(&snap, SemanticsRole::Button, Some("Link"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
        .expect("fret button");

    assert_close_px(
        "button-link w",
        button.bounds.size.width,
        web_button.rect.w,
        4.0,
    );
    assert_close_px(
        "button-link h",
        button.bounds.size.height,
        web_button.rect.h,
        1.0,
    );
}

#[test]
fn button_link_in_grid_auto_track_preserves_intrinsic_width() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(320.0), Px(100.0)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        vec![cx.grid(
            GridProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                cols: 1,
                rows: Some(2),
                template_columns: Some(vec![
                    fret_ui::element::GridTrackSizing::Fr(1.0),
                    fret_ui::element::GridTrackSizing::Auto,
                ]),
                template_rows: Some(vec![
                    fret_ui::element::GridTrackSizing::Auto,
                    fret_ui::element::GridTrackSizing::Auto,
                ]),
                align: CrossAlign::Start,
                padding: Edges::all(Px(24.0)).into(),
                row_gap: Some(Px(8.0).into()),
                ..Default::default()
            },
            |cx| {
                let mut title_props = ContainerProps::default();
                title_props.layout.size.width = Length::Fill;
                title_props.layout.size.height = Length::Px(Px(14.0));
                title_props.layout.grid.column.start = Some(1);
                title_props.layout.grid.row.start = Some(1);
                let title = cx.container(title_props, |_cx| Vec::new());

                let mut description_props = ContainerProps::default();
                description_props.layout.size.width = Length::Fill;
                description_props.layout.size.height = Length::Px(Px(20.0));
                description_props.layout.grid.column.start = Some(1);
                description_props.layout.grid.row.start = Some(2);
                let description = cx.container(description_props, |_cx| Vec::new());

                let mut slot_props = ContainerProps::default();
                slot_props.layout.grid.column.start = Some(2);
                slot_props.layout.grid.row.start = Some(1);
                slot_props.layout.grid.row.span = Some(2);
                slot_props.layout.grid.align_self = Some(CrossAlign::Start);
                slot_props.layout.grid.justify_self = Some(CrossAlign::End);
                let slot = cx
                    .container(slot_props, |cx| {
                        vec![
                            shadcn::Button::new("Sign Up")
                                .variant(shadcn::ButtonVariant::Link)
                                .into_element(cx)
                                .test_id("grid-auto-link-button"),
                        ]
                    })
                    .test_id("grid-auto-link-slot");

                vec![title, description, slot]
            },
        )]
    });

    let slot = find_by_test_id(&snap, "grid-auto-link-slot");
    let button = find_by_test_id(&snap, "grid-auto-link-button");
    let button_semantics = find_semantics(&snap, SemanticsRole::Button, Some("Sign Up"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
        .expect("grid auto-track link button semantics");

    assert!(
        slot.bounds.size.width.0 > 0.0,
        "expected grid auto track slot to keep non-zero intrinsic width for link button, got slot={:?}; prepared={:#?}",
        slot.bounds,
        services.prepared
    );
    assert!(
        button.bounds.size.width.0 > 0.0,
        "expected link button root to keep non-zero intrinsic width in grid auto track, got button={:?}; slot={:?}; prepared={:#?}",
        button.bounds,
        slot.bounds,
        services.prepared
    );
    assert!(
        button_semantics.bounds.size.width.0 > 0.0,
        "expected link button semantics root to keep non-zero intrinsic width in grid auto track, got semantics={:?}; button={:?}; slot={:?}; prepared={:#?}",
        button_semantics.bounds,
        button.bounds,
        slot.bounds,
        services.prepared
    );
}
