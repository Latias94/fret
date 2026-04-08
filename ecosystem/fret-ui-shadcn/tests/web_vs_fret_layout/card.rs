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

#[test]
fn card_action_with_link_button_in_grid_auto_track_preserves_intrinsic_width() {
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

                let action = shadcn::CardAction::new([{
                    shadcn::Button::new("Sign Up")
                        .variant(shadcn::ButtonVariant::Link)
                        .into_element(cx)
                        .test_id("grid-auto-card-action-button")
                }])
                .into_element(cx)
                .test_id("grid-auto-card-action-slot");

                vec![title, description, action]
            },
        )]
    });

    let slot = find_by_test_id(&snap, "grid-auto-card-action-slot");
    let button = find_by_test_id(&snap, "grid-auto-card-action-button");

    assert!(
        slot.bounds.size.width.0 > 0.0,
        "expected CardAction slot to keep non-zero width in a grid auto track, got slot={:?}; prepared={:#?}",
        slot.bounds,
        services.prepared
    );
    assert!(
        button.bounds.size.width.0 > 0.0,
        "expected Button(Link) root inside CardAction to keep non-zero width, got button={:?}; slot={:?}; prepared={:#?}",
        button.bounds,
        slot.bounds,
        services.prepared
    );
}

#[test]
fn card_title_description_and_action_raw_grid_keep_action_intrinsic_width() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(384.0), Px(120.0)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        vec![cx.grid(
            GridProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Auto;
                    layout.size.min_width = Some(Length::Px(Px(0.0)));
                    layout
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
                gap: Px(8.0).into(),
                padding: Edges::all(Px(24.0)).into(),
                align: CrossAlign::Start,
                ..Default::default()
            },
            |cx| {
                vec![
                    shadcn::CardTitle::new("Login to your account")
                        .into_element(cx)
                        .test_id("raw-grid-card-title"),
                    shadcn::CardDescription::new("Enter your email below to login to your account")
                        .into_element(cx)
                        .test_id("raw-grid-card-description"),
                    shadcn::CardAction::new([{
                        shadcn::Button::new("Sign Up")
                            .variant(shadcn::ButtonVariant::Link)
                            .into_element(cx)
                            .test_id("raw-grid-card-action-button")
                    }])
                    .into_element(cx)
                    .test_id("raw-grid-card-action-slot"),
                ]
            },
        )]
    });

    let title = find_by_test_id(&snap, "raw-grid-card-title");
    let description = find_by_test_id(&snap, "raw-grid-card-description");
    let slot = find_by_test_id(&snap, "raw-grid-card-action-slot");
    let button = find_by_test_id(&snap, "raw-grid-card-action-button");
    let button_semantics = find_semantics(&snap, SemanticsRole::Button, Some("Sign Up"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
        .expect("raw grid card action button semantics");

    assert!(
        slot.bounds.size.width.0 > 0.0,
        "expected CardAction slot to keep non-zero width alongside CardTitle/CardDescription, got slot={:?}; title={:?}; description={:?}; button={:?}; prepared={:#?}",
        slot.bounds,
        title.bounds,
        description.bounds,
        button.bounds,
        services.prepared
    );
    assert!(
        button.bounds.size.width.0 > 0.0,
        "expected action button root to keep non-zero width alongside CardTitle/CardDescription, got slot={:?}; title={:?}; description={:?}; button={:?}; prepared={:#?}",
        slot.bounds,
        title.bounds,
        description.bounds,
        button.bounds,
        services.prepared
    );
    assert!(
        button_semantics.bounds.size.width.0 > 0.0,
        "expected action button semantics root to keep non-zero width alongside CardTitle/CardDescription, got semantics={:?}; slot={:?}; title={:?}; description={:?}; button={:?}; prepared={:#?}",
        button_semantics.bounds,
        slot.bounds,
        title.bounds,
        description.bounds,
        button.bounds,
        services.prepared
    );
}

#[test]
fn card_header_as_flex_child_keeps_action_intrinsic_width() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(384.0), Px(160.0)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        vec![
            ui::v_flex(|cx| {
                vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("Login to your account")
                            .into_element(cx)
                            .test_id("flex-child-card-title"),
                        shadcn::CardDescription::new(
                            "Enter your email below to login to your account",
                        )
                        .into_element(cx)
                        .test_id("flex-child-card-description"),
                        shadcn::CardAction::new([{
                            shadcn::Button::new("Sign Up")
                                .variant(shadcn::ButtonVariant::Link)
                                .into_element(cx)
                                .test_id("flex-child-card-action-button")
                        }])
                        .into_element(cx)
                        .test_id("flex-child-card-action-slot"),
                    ])
                    .into_element(cx),
                ]
            })
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .into_element(cx),
        ]
    });

    let title = find_by_test_id(&snap, "flex-child-card-title");
    let description = find_by_test_id(&snap, "flex-child-card-description");
    let slot = find_by_test_id(&snap, "flex-child-card-action-slot");
    let button = find_by_test_id(&snap, "flex-child-card-action-button");
    let button_semantics = find_semantics(&snap, SemanticsRole::Button, Some("Sign Up"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
        .expect("flex child card action button semantics");

    assert!(
        slot.bounds.size.width.0 > 0.0,
        "expected CardHeader action slot to keep non-zero width as a flex child, got slot={:?}; title={:?}; description={:?}; button={:?}; prepared={:#?}",
        slot.bounds,
        title.bounds,
        description.bounds,
        button.bounds,
        services.prepared
    );
    assert!(
        button.bounds.size.width.0 > 0.0,
        "expected CardHeader action button root to keep non-zero width as a flex child, got slot={:?}; title={:?}; description={:?}; button={:?}; prepared={:#?}",
        slot.bounds,
        title.bounds,
        description.bounds,
        button.bounds,
        services.prepared
    );
    assert!(
        button_semantics.bounds.size.width.0 > 0.0,
        "expected CardHeader action button semantics root to keep non-zero width as a flex child, got semantics={:?}; slot={:?}; title={:?}; description={:?}; button={:?}; prepared={:#?}",
        button_semantics.bounds,
        slot.bounds,
        title.bounds,
        description.bounds,
        button.bounds,
        services.prepared
    );
}

#[test]
fn card_demo_action_does_not_require_a_second_frame_to_get_width() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(384.0), Px(220.0)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_frames_with_services(bounds, &mut services, 2, |cx| {
        vec![
            shadcn::Card::new(vec![{
                shadcn::CardHeader::new(vec![
                    shadcn::CardTitle::new("Login to your account").into_element(cx),
                    shadcn::CardDescription::new("Enter your email below to login to your account")
                        .into_element(cx),
                    shadcn::CardAction::new([{
                        shadcn::Button::new("Sign Up")
                            .variant(shadcn::ButtonVariant::Link)
                            .into_element(cx)
                    }])
                    .into_element(cx),
                ])
                .into_element(cx)
            }])
            .refine_layout(LayoutRefinement::default().w_px(Px(384.0)))
            .into_element(cx),
        ]
    });

    let sign_up = find_semantics(&snap, SemanticsRole::Button, Some("Sign Up"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
        .expect("sign up button");

    assert!(
        sign_up.bounds.size.width.0 > 0.0,
        "expected card demo Sign Up button to have non-zero width by the second frame, got {:?}; prepared={:#?}",
        sign_up.bounds,
        services.prepared
    );
}

#[test]
fn card_header_inside_bordered_padded_container_keeps_action_intrinsic_width() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(384.0), Px(220.0)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let outer = cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Px(Px(384.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                border: Edges::all(Px(1.0)),
                padding: Edges {
                    top: Px(24.0).into(),
                    bottom: Px(24.0).into(),
                    left: Px(0.0).into(),
                    right: Px(0.0).into(),
                }
                .into(),
                ..Default::default()
            },
            |cx| {
                vec![
                    ui::v_flex(|cx| {
                        vec![
                            shadcn::CardHeader::new(vec![
                                shadcn::CardTitle::new("Login to your account")
                                    .into_element(cx)
                                    .test_id("bordered-container-card-title"),
                                shadcn::CardDescription::new(
                                    "Enter your email below to login to your account",
                                )
                                .into_element(cx)
                                .test_id("bordered-container-card-description"),
                                shadcn::CardAction::new([{
                                    shadcn::Button::new("Sign Up")
                                        .variant(shadcn::ButtonVariant::Link)
                                        .into_element(cx)
                                        .test_id("bordered-container-card-action-button")
                                }])
                                .into_element(cx)
                                .test_id("bordered-container-card-action-slot"),
                            ])
                            .into_element(cx),
                        ]
                    })
                    .gap(Space::N6)
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .into_element(cx),
                ]
            },
        );

        vec![outer]
    });

    let title = find_by_test_id(&snap, "bordered-container-card-title");
    let description = find_by_test_id(&snap, "bordered-container-card-description");
    let slot = find_by_test_id(&snap, "bordered-container-card-action-slot");
    let button = find_by_test_id(&snap, "bordered-container-card-action-button");
    let button_semantics = find_semantics(&snap, SemanticsRole::Button, Some("Sign Up"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
        .expect("bordered container card action button semantics");

    assert!(
        slot.bounds.size.width.0 > 0.0,
        "expected CardHeader action slot to keep non-zero width inside bordered+padded container, got slot={:?}; title={:?}; description={:?}; button={:?}; prepared={:#?}",
        slot.bounds,
        title.bounds,
        description.bounds,
        button.bounds,
        services.prepared
    );
    assert!(
        button.bounds.size.width.0 > 0.0,
        "expected CardHeader action button root to keep non-zero width inside bordered+padded container, got slot={:?}; title={:?}; description={:?}; button={:?}; prepared={:#?}",
        slot.bounds,
        title.bounds,
        description.bounds,
        button.bounds,
        services.prepared
    );
    assert!(
        button_semantics.bounds.size.width.0 > 0.0,
        "expected CardHeader action button semantics root to keep non-zero width inside bordered+padded container, got semantics={:?}; slot={:?}; title={:?}; description={:?}; button={:?}; prepared={:#?}",
        button_semantics.bounds,
        slot.bounds,
        title.bounds,
        description.bounds,
        button.bounds,
        services.prepared
    );
}

#[test]
fn card_footer_short_text_keeps_same_wrap_budget_as_card_content() {
    fn prepared_text<'a>(
        services: &'a StyleAwareServices,
        expected_text: &str,
    ) -> &'a RecordedTextPrepare {
        services
            .prepared
            .iter()
            .rev()
            .find(|record| record.text == expected_text)
            .unwrap_or_else(|| {
                panic!(
                    "missing prepared text record for {expected_text:?}; prepared={:#?}",
                    services.prepared
                )
            })
    }

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(384.0), Px(320.0)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        vec![
            ui::v_flex(|cx| {
                vec![
                    shadcn::Card::new([{
                        shadcn::CardContent::new([{
                            ui::text("Wrap budget text A")
                                .text_sm()
                                .test_id("card-content-wrap-budget")
                                .into_element(cx)
                        }])
                        .into_element(cx)
                    }])
                    .refine_layout(LayoutRefinement::default().w_px(Px(260.0)))
                    .into_element(cx),
                    shadcn::Card::new([{
                        shadcn::CardFooter::new([{
                            ui::text("Wrap budget text B")
                                .text_sm()
                                .test_id("card-footer-wrap-budget")
                                .into_element(cx)
                        }])
                        .into_element(cx)
                    }])
                    .refine_layout(LayoutRefinement::default().w_px(Px(260.0)))
                    .into_element(cx),
                ]
            })
            .gap(Space::N6)
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .into_element(cx),
        ]
    });

    let content_record = prepared_text(&services, "Wrap budget text A");
    let footer_record = prepared_text(&services, "Wrap budget text B");
    let content_max_width = content_record
        .constraints
        .max_width
        .expect("CardContent wrap-budget text should have a definite max width");
    let footer_max_width = footer_record
        .constraints
        .max_width
        .expect("CardFooter wrap-budget text should have a definite max width");

    let content_text = find_by_test_id(&snap, "card-content-wrap-budget");
    let footer_text = find_by_test_id(&snap, "card-footer-wrap-budget");

    assert_close_px(
        "card footer wrap budget matches card content",
        footer_max_width,
        content_max_width.0,
        1.0,
    );
    assert!(
        footer_max_width.0 > 0.0,
        "expected CardFooter short text to receive a non-zero final wrap budget, got footer={footer_max_width:?} content={content_max_width:?}; prepared={:#?}",
        services.prepared
    );
    assert!(
        (footer_text.bounds.size.height.0 - content_text.bounds.size.height.0).abs() <= 1.0,
        "expected CardFooter short text to stay single-line like CardContent text: footer={:?} content={:?}; prepared={:#?}",
        footer_text.bounds,
        content_text.bounds,
        services.prepared
    );
}

#[test]
fn card_header_inside_exact_card_chrome_container_keeps_action_intrinsic_width() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(384.0), Px(220.0)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let theme = Theme::global(&*cx.app);
        let base_radius = theme.metric_token("metric.radius.lg");
        let rounded_xl = Px(base_radius.0 + 4.0);
        let props = decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .radius(MetricRef::Px(rounded_xl))
                .border_width(MetricRef::Px(Px(1.0)))
                .shadow_sm()
                .py(Space::N6)
                .bg(ColorRef::Color(theme.color_token("card")))
                .border_color(ColorRef::Color(theme.color_token("border"))),
            LayoutRefinement::default().w_px(Px(384.0)),
        );
        let fg = theme.color_token("card-foreground");

        vec![
            cx.container(props, |cx| {
                vec![
                    ui::v_flex(|cx| {
                        vec![
                            shadcn::CardHeader::new(vec![
                                shadcn::CardTitle::new("Login to your account")
                                    .into_element(cx)
                                    .test_id("exact-card-chrome-title"),
                                shadcn::CardDescription::new(
                                    "Enter your email below to login to your account",
                                )
                                .into_element(cx)
                                .test_id("exact-card-chrome-description"),
                                shadcn::CardAction::new([{
                                    shadcn::Button::new("Sign Up")
                                        .variant(shadcn::ButtonVariant::Link)
                                        .into_element(cx)
                                        .test_id("exact-card-chrome-button")
                                }])
                                .into_element(cx),
                            ])
                            .into_element(cx),
                        ]
                    })
                    .gap(Space::N6)
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .into_element(cx),
                ]
            })
            .inherit_foreground(fg),
        ]
    });

    let sign_up = find_semantics(&snap, SemanticsRole::Button, Some("Sign Up"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
        .expect("exact card chrome sign up button");

    assert!(
        sign_up.bounds.size.width.0 > 0.0,
        "expected manual exact-card-chrome container to keep non-zero Sign Up width, got {:?}; prepared={:#?}",
        sign_up.bounds,
        services.prepared
    );
    assert!(
        sign_up.bounds.origin.x.0 + sign_up.bounds.size.width.0 <= 384.0,
        "expected manual exact-card-chrome container to keep Sign Up inside the card width, got {:?}; prepared={:#?}",
        sign_up.bounds,
        services.prepared
    );
}

#[test]
fn card_header_build_inside_exact_card_chrome_container_keeps_action_intrinsic_width() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(384.0), Px(220.0)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let theme = Theme::global(&*cx.app);
        let base_radius = theme.metric_token("metric.radius.lg");
        let rounded_xl = Px(base_radius.0 + 4.0);
        let props = decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .radius(MetricRef::Px(rounded_xl))
                .border_width(MetricRef::Px(Px(1.0)))
                .shadow_sm()
                .py(Space::N6)
                .bg(ColorRef::Color(theme.color_token("card")))
                .border_color(ColorRef::Color(theme.color_token("border"))),
            LayoutRefinement::default().w_px(Px(384.0)),
        );
        let fg = theme.color_token("card-foreground");

        vec![
            cx.container(props, |cx| {
                vec![
                    ui::v_flex(|cx| {
                        vec![
                            shadcn::card_header(|cx| {
                                ui::children![
                                    cx;
                                    shadcn::card_title("Login to your account"),
                                    shadcn::card_description("Enter your email below to login to your account"),
                                    shadcn::card_action(|cx| {
                                        ui::children![
                                            cx;
                                            shadcn::Button::new("Sign Up")
                                                .variant(shadcn::ButtonVariant::Link),
                                        ]
                                    }),
                                ]
                            })
                            .into_element(cx),
                        ]
                    })
                    .gap(Space::N6)
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .into_element(cx),
                ]
            })
            .inherit_foreground(fg),
        ]
    });

    let sign_up = find_semantics(&snap, SemanticsRole::Button, Some("Sign Up"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
        .expect("exact card chrome sign up button via CardHeaderBuild");

    assert!(
        sign_up.bounds.size.width.0 > 0.0,
        "expected exact-card-chrome container + CardHeaderBuild to keep non-zero Sign Up width, got {:?}; prepared={:#?}",
        sign_up.bounds,
        services.prepared
    );
}

#[test]
fn card_header_new_with_card_action_build_keeps_action_intrinsic_width() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(384.0), Px(220.0)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let theme = Theme::global(&*cx.app);
        let base_radius = theme.metric_token("metric.radius.lg");
        let rounded_xl = Px(base_radius.0 + 4.0);
        let props = decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .radius(MetricRef::Px(rounded_xl))
                .border_width(MetricRef::Px(Px(1.0)))
                .shadow_sm()
                .py(Space::N6)
                .bg(ColorRef::Color(theme.color_token("card")))
                .border_color(ColorRef::Color(theme.color_token("border"))),
            LayoutRefinement::default().w_px(Px(384.0)),
        );
        let fg = theme.color_token("card-foreground");

        vec![
            cx.container(props, |cx| {
                vec![
                    ui::v_flex(|cx| {
                        vec![
                            shadcn::CardHeader::new(vec![
                                shadcn::CardTitle::new("Login to your account").into_element(cx),
                                shadcn::CardDescription::new(
                                    "Enter your email below to login to your account",
                                )
                                .into_element(cx),
                                shadcn::card_action(|cx| {
                                    ui::children![
                                        cx;
                                        shadcn::Button::new("Sign Up")
                                            .variant(shadcn::ButtonVariant::Link),
                                    ]
                                })
                                .into_element(cx),
                            ])
                            .into_element(cx),
                        ]
                    })
                    .gap(Space::N6)
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .into_element(cx),
                ]
            })
            .inherit_foreground(fg),
        ]
    });

    let sign_up = find_semantics(&snap, SemanticsRole::Button, Some("Sign Up"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
        .expect("exact card chrome sign up button via CardActionBuild");

    assert!(
        sign_up.bounds.size.width.0 > 0.0,
        "expected CardHeader::new + CardActionBuild to keep non-zero Sign Up width, got {:?}; prepared={:#?}",
        sign_up.bounds,
        services.prepared
    );
}

#[test]
fn builder_first_card_keeps_action_intrinsic_width() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(384.0), Px(220.0)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        vec![
            shadcn::card(|cx| {
                ui::children![
                    cx;
                    shadcn::card_header(|cx| {
                        ui::children![
                            cx;
                            shadcn::card_title("Login to your account"),
                            shadcn::card_description("Enter your email below to login to your account"),
                            shadcn::card_action(|cx| {
                                ui::children![
                                    cx;
                                    shadcn::Button::new("Sign Up")
                                        .variant(shadcn::ButtonVariant::Link),
                                ]
                            }),
                        ]
                    }),
                ]
            })
            .refine_layout(LayoutRefinement::default().w_px(Px(384.0)))
            .into_element(cx),
        ]
    });

    let sign_up = find_semantics(&snap, SemanticsRole::Button, Some("Sign Up"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
        .expect("builder-first sign up button");

    assert!(
        sign_up.bounds.size.width.0 > 0.0,
        "expected builder-first card Sign Up button to keep non-zero width, got {:?}; prepared={:#?}",
        sign_up.bounds,
        services.prepared
    );
}

#[test]
fn web_vs_fret_layout_card_demo_header_action_geometry_matches_web() {
    let web = read_web_golden("card-demo");
    let theme = web_theme(&web);
    let web_card = web_find_by_class_tokens(
        &theme.root,
        &[
            "bg-card",
            "text-card-foreground",
            "rounded-xl",
            "border",
            "max-w-sm",
        ],
    )
    .expect("web card root");
    let web_title = find_first(&theme.root, &|node| {
        node.tag == "div" && node.text.as_deref() == Some("Login to your account")
    })
    .expect("web card title");
    let web_description = find_first(&theme.root, &|node| {
        node.tag == "div"
            && node.text.as_deref() == Some("Enter your email below to login to your account")
    })
    .expect("web card description");
    let web_sign_up =
        web_find_by_tag_and_text(&theme.root, "button", "Sign Up").expect("web sign up button");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let card = shadcn::Card::new(vec![{
            shadcn::CardHeader::new(vec![
                shadcn::CardTitle::new("Login to your account")
                    .into_element(cx)
                    .test_id("card-demo-title"),
                shadcn::CardDescription::new("Enter your email below to login to your account")
                    .into_element(cx)
                    .test_id("card-demo-description"),
                shadcn::CardAction::new([{
                    shadcn::Button::new("Sign Up")
                        .variant(shadcn::ButtonVariant::Link)
                        .test_id("card-demo-sign-up")
                        .into_element(cx)
                }])
                .into_element(cx),
            ])
            .into_element(cx)
        }])
        .refine_layout(LayoutRefinement::default().w_px(Px(web_card.rect.w)))
        .into_element(cx)
        .test_id("card-demo-card");

        vec![card]
    });

    let card = find_by_test_id(&snap, "card-demo-card");
    let title = find_by_test_id(&snap, "card-demo-title");
    let description = find_by_test_id(&snap, "card-demo-description");
    let sign_up = find_semantics(&snap, SemanticsRole::Button, Some("Sign Up"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
        .expect("sign up button");
    let sign_up_button = find_by_test_id(&snap, "card-demo-sign-up");
    let sign_up_chrome = find_by_test_id(&snap, "card-demo-sign-up.chrome");

    let epsilon = 4.0;
    let title_x = title.bounds.origin.x.0 - card.bounds.origin.x.0;
    let title_y = title.bounds.origin.y.0 - card.bounds.origin.y.0;
    let description_x = description.bounds.origin.x.0 - card.bounds.origin.x.0;
    let description_y = description.bounds.origin.y.0 - card.bounds.origin.y.0;
    let sign_up_x = sign_up.bounds.origin.x.0 - card.bounds.origin.x.0;
    let sign_up_y = sign_up.bounds.origin.y.0 - card.bounds.origin.y.0;

    assert_close_px(
        "card-demo card width",
        card.bounds.size.width,
        web_card.rect.w,
        1.0,
    );
    assert_close_px(
        "card-demo title x",
        Px(title_x),
        web_title.rect.x - web_card.rect.x,
        epsilon,
    );
    assert_close_px(
        "card-demo title y",
        Px(title_y),
        web_title.rect.y - web_card.rect.y,
        epsilon,
    );
    assert_close_px(
        "card-demo description x",
        Px(description_x),
        web_description.rect.x - web_card.rect.x,
        epsilon,
    );
    assert_close_px(
        "card-demo description y",
        Px(description_y),
        web_description.rect.y - web_card.rect.y,
        epsilon,
    );
    assert!(
        (sign_up_x - (web_sign_up.rect.x - web_card.rect.x)).abs() <= epsilon,
        "card-demo sign-up x: expected≈{} (±{epsilon}) got={sign_up_x}; button={:?}; chrome={:?}",
        web_sign_up.rect.x - web_card.rect.x,
        sign_up_button.bounds,
        sign_up_chrome.bounds,
    );
    assert!(
        (sign_up_y - (web_sign_up.rect.y - web_card.rect.y)).abs() <= epsilon,
        "card-demo sign-up y: expected≈{} (±{epsilon}) got={sign_up_y}; button={:?}; chrome={:?}",
        web_sign_up.rect.y - web_card.rect.y,
        sign_up_button.bounds,
        sign_up_chrome.bounds,
    );
    assert!(
        (sign_up.bounds.size.width.0 - web_sign_up.rect.w).abs() <= epsilon,
        "card-demo sign-up w: expected≈{} (±{epsilon}) got={}; button={:?}; chrome={:?}",
        web_sign_up.rect.w,
        sign_up.bounds.size.width.0,
        sign_up_button.bounds,
        sign_up_chrome.bounds,
    );
    assert!(
        (sign_up.bounds.size.height.0 - web_sign_up.rect.h).abs() <= epsilon,
        "card-demo sign-up h: expected≈{} (±{epsilon}) got={}; button={:?}; chrome={:?}",
        web_sign_up.rect.h,
        sign_up.bounds.size.height.0,
        sign_up_button.bounds,
        sign_up_chrome.bounds,
    );
}
