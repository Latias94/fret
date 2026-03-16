use super::*;
use fret_ui_shadcn::facade as shadcn;

#[test]
fn web_vs_fret_layout_card_with_form_width() {
    let web = read_web_golden("card-with-form");
    let theme = web_theme(&web);
    let web_card = web_find_by_class_tokens(
        &theme.root,
        &[
            "bg-card",
            "text-card-foreground",
            "rounded-xl",
            "border",
            "w-[350px]",
        ],
    )
    .expect("web card root");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let card = shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![
                shadcn::CardTitle::new("Title").into_element(cx),
                shadcn::CardDescription::new("Description").into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new(vec![cx.text("Content")]).into_element(cx),
        ])
        .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(web_card.rect.w)))
        .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:card-with-form:root")),
                ..Default::default()
            },
            move |_cx| vec![card],
        )]
    });

    let card = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:card-with-form:root"),
    )
    .expect("fret card root");

    assert_close_px("card width", card.bounds.size.width, web_card.rect.w, 1.0);
}

#[test]
fn card_header_action_gap_matches_shadcn_gap_2() {
    let expected_gap = {
        let mut app = App::new();
        fret_ui_shadcn::facade::themes::apply_shadcn_new_york(
            &mut app,
            fret_ui_shadcn::facade::themes::ShadcnBaseColor::Neutral,
            fret_ui_shadcn::facade::themes::ShadcnColorScheme::Light,
        );
        let theme = Theme::global(&app);
        MetricRef::space(Space::N2).resolve(theme)
    };

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(420.0), Px(220.0)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let title = cx
            .container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Px(Px(10.0)),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )
            .test_id("test.card.header.title");

        let description = cx
            .container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Px(Px(10.0)),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )
            .test_id("test.card.header.description");

        let action = shadcn::CardAction::new([cx
            .container(ContainerProps::default(), |_cx| Vec::new())
            .test_id("test.card.header.action")])
        .into_element(cx);

        let header = shadcn::CardHeader::new([title, description, action]).into_element(cx);

        let card = shadcn::Card::new([header])
            .refine_layout(LayoutRefinement::default().w_px(Px(320.0)))
            .into_element(cx);

        vec![card]
    });

    let title = find_by_test_id(&snap, "test.card.header.title");
    let description = find_by_test_id(&snap, "test.card.header.description");

    let actual_gap =
        description.bounds.origin.y.0 - (title.bounds.origin.y.0 + title.bounds.size.height.0);
    assert!(
        (actual_gap - expected_gap.0).abs() <= 1.0,
        "expected header gap≈{}px (gap-2) got={}px",
        expected_gap.0,
        actual_gap
    );
}
