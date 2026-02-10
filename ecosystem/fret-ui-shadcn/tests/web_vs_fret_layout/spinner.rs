use super::*;

#[test]
fn web_vs_fret_layout_spinner_input_group_geometry_matches() {
    let web = read_web_golden("spinner-input-group");
    let theme = web_theme(&web);

    let mut web_groups: Vec<&WebNode> = Vec::new();
    fn walk_collect<'a>(n: &'a WebNode, out: &mut Vec<&'a WebNode>) {
        if n.tag == "div"
            && n.class_name.as_deref().is_some_and(|c| {
                let mut has_group = false;
                let mut has_border = false;
                for t in c.split_whitespace() {
                    has_group |= t == "group/input-group";
                    has_border |= t == "border-input";
                }
                has_group && has_border
            })
        {
            out.push(n);
        }
        for c in &n.children {
            walk_collect(c, out);
        }
    }
    walk_collect(&theme.root, &mut web_groups);
    web_groups.sort_by(|a, b| {
        a.rect
            .y
            .partial_cmp(&b.rect.y)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let web_group0 = *web_groups.get(0).expect("web group 0");
    let web_group1 = *web_groups.get(1).expect("web group 1");

    let expected_gap_y = web_group1.rect.y - (web_group0.rect.y + web_group0.rect.h);

    let web_input0 = web_group0
        .children
        .iter()
        .find(|n| n.tag == "input")
        .expect("web input0");
    let web_svg0 = find_first(web_group0, &|n| n.tag == "svg").expect("web svg0");

    let web_textarea1 = web_group1
        .children
        .iter()
        .find(|n| n.tag == "textarea")
        .expect("web textarea1");
    let web_svg1a = find_first(web_group1, &|n| {
        n.tag == "svg" && (n.rect.w - 16.0).abs() <= 0.1
    })
    .expect("web svg1a (spinner)");
    let web_svg1b = find_first(web_group1, &|n| {
        n.tag == "svg" && (n.rect.w - 14.0).abs() <= 0.1
    })
    .expect("web svg1b (arrow)");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let model0: Model<String> = app.models_mut().insert(String::new());
    let model1: Model<String> = app.models_mut().insert(String::new());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices::default();

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout-spinner-input-group",
        |cx| {
            let container_layout =
                fret_ui_kit::LayoutRefinement::default().w_px(Px(web_group0.rect.w));
            let container = cx.container(
                fret_ui::element::ContainerProps {
                    layout: fret_ui_kit::declarative::style::layout_style(
                        &fret_ui::Theme::global(&*cx.app),
                        container_layout,
                    ),
                    ..Default::default()
                },
                move |cx| {
                    let spinner0 = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:spinner-input-group:0:spinner")),
                            ..Default::default()
                        },
                        move |cx| vec![fret_ui_shadcn::Spinner::new().speed(0.0).into_element(cx)],
                    );

                    let group0 = fret_ui_shadcn::InputGroup::new(model0.clone())
                        .a11y_label("Golden:spinner-input-group:0:input")
                        .trailing(vec![spinner0])
                        .into_element(cx);
                    let group0 = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:spinner-input-group:0:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![group0],
                    );

                    let spinner1 = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:spinner-input-group:1:spinner")),
                            ..Default::default()
                        },
                        move |cx| vec![fret_ui_shadcn::Spinner::new().speed(0.0).into_element(cx)],
                    );
                    let arrow = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:spinner-input-group:1:arrow")),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![decl_icon::icon_with(
                                cx,
                                fret_icons::ids::ui::CHEVRON_UP,
                                Some(Px(14.0)),
                                None,
                            )]
                        },
                    );
                    let send_button = cx.container(
                        fret_ui::element::ContainerProps {
                            layout: fret_ui_kit::declarative::style::layout_style(
                                &fret_ui::Theme::global(&*cx.app),
                                fret_ui_kit::LayoutRefinement::default()
                                    .ml_auto()
                                    .w_px(Px(30.0))
                                    .h_px(Px(24.0)),
                            ),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![cx.flex(
                                FlexProps {
                                    layout: LayoutStyle::default(),
                                    direction: fret_core::Axis::Horizontal,
                                    gap: Px(0.0),
                                    padding: Edges::symmetric(Px(8.0), Px(0.0)),
                                    justify: MainAlign::Start,
                                    align: CrossAlign::Center,
                                    wrap: false,
                                },
                                move |_cx| vec![arrow],
                            )]
                        },
                    );

                    let group1_addon = vec![spinner1, cx.text("Validating..."), send_button];
                    let group1 = fret_ui_shadcn::InputGroup::new(model1.clone())
                        .textarea()
                        .a11y_label("Golden:spinner-input-group:1:textarea")
                        .block_end(group1_addon)
                        .into_element(cx);
                    let group1 = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:spinner-input-group:1:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![group1],
                    );

                    vec![cx.column(
                        ColumnProps {
                            gap: Px(expected_gap_y),
                            ..Default::default()
                        },
                        move |_cx| vec![group0, group1],
                    )]
                },
            );

            vec![container]
        },
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let group0 = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:spinner-input-group:0:root"),
    )
    .expect("fret group0");
    let input0 = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:spinner-input-group:0:input"),
    )
    .expect("fret input0");
    let spinner0 = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:spinner-input-group:0:spinner"),
    )
    .expect("fret spinner0");

    assert_close_px(
        "spinner-input-group group0 y",
        group0.bounds.origin.y,
        web_group0.rect.y,
        1.0,
    );
    assert_close_px(
        "spinner-input-group group0 w",
        group0.bounds.size.width,
        web_group0.rect.w,
        1.0,
    );
    assert_close_px(
        "spinner-input-group group0 h",
        group0.bounds.size.height,
        web_group0.rect.h,
        1.0,
    );
    assert_close_px(
        "spinner-input-group input0 x",
        input0.bounds.origin.x,
        web_input0.rect.x,
        1.0,
    );
    assert_close_px(
        "spinner-input-group input0 w",
        input0.bounds.size.width,
        web_input0.rect.w,
        1.0,
    );
    assert_close_px(
        "spinner-input-group spinner0 x",
        spinner0.bounds.origin.x,
        web_svg0.rect.x,
        1.0,
    );
    assert_close_px(
        "spinner-input-group spinner0 y",
        spinner0.bounds.origin.y,
        web_svg0.rect.y,
        1.0,
    );
    assert_close_px(
        "spinner-input-group spinner0 w",
        spinner0.bounds.size.width,
        web_svg0.rect.w,
        1.0,
    );
    assert_close_px(
        "spinner-input-group spinner0 h",
        spinner0.bounds.size.height,
        web_svg0.rect.h,
        1.0,
    );

    let group1 = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:spinner-input-group:1:root"),
    )
    .expect("fret group1");
    let textarea1 = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:spinner-input-group:1:textarea"),
    )
    .expect("fret textarea1");
    let spinner1 = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:spinner-input-group:1:spinner"),
    )
    .expect("fret spinner1");
    let arrow = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:spinner-input-group:1:arrow"),
    )
    .expect("fret arrow");

    assert_close_px(
        "spinner-input-group group1 y",
        group1.bounds.origin.y,
        web_group1.rect.y,
        1.0,
    );
    assert_close_px(
        "spinner-input-group group1 w",
        group1.bounds.size.width,
        web_group1.rect.w,
        1.0,
    );
    assert_close_px(
        "spinner-input-group group1 h",
        group1.bounds.size.height,
        web_group1.rect.h,
        1.0,
    );
    assert_close_px(
        "spinner-input-group textarea1 x",
        textarea1.bounds.origin.x,
        web_textarea1.rect.x,
        1.0,
    );
    assert_close_px(
        "spinner-input-group textarea1 y",
        textarea1.bounds.origin.y,
        web_textarea1.rect.y,
        1.0,
    );
    assert_close_px(
        "spinner-input-group textarea1 w",
        textarea1.bounds.size.width,
        web_textarea1.rect.w,
        1.0,
    );
    assert_close_px(
        "spinner-input-group textarea1 h",
        textarea1.bounds.size.height,
        web_textarea1.rect.h,
        1.0,
    );
    assert_close_px(
        "spinner-input-group spinner1 x",
        spinner1.bounds.origin.x,
        web_svg1a.rect.x,
        1.0,
    );
    assert_close_px(
        "spinner-input-group spinner1 y",
        spinner1.bounds.origin.y,
        web_svg1a.rect.y,
        1.0,
    );
    assert_close_px(
        "spinner-input-group arrow x",
        arrow.bounds.origin.x,
        web_svg1b.rect.x,
        1.0,
    );
    assert_close_px(
        "spinner-input-group arrow y",
        arrow.bounds.origin.y,
        web_svg1b.rect.y,
        1.0,
    );
    assert_close_px(
        "spinner-input-group arrow w",
        arrow.bounds.size.width,
        web_svg1b.rect.w,
        1.0,
    );
    assert_close_px(
        "spinner-input-group arrow h",
        arrow.bounds.size.height,
        web_svg1b.rect.h,
        1.0,
    );
}
#[test]
fn web_vs_fret_layout_spinner_basic_geometry_matches_web() {
    let web = read_web_golden("spinner-basic");
    let theme = web_theme(&web);
    let web_spinner = find_first(&theme.root, &|n| {
        n.tag == "svg" && class_has_token(n, "animate-spin")
    })
    .expect("web spinner svg");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let spinner = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("Golden:spinner-basic:spinner")),
                ..Default::default()
            },
            move |cx| vec![fret_ui_shadcn::Spinner::new().speed(0.0).into_element(cx)],
        );
        vec![spinner]
    });

    let spinner = find_by_test_id(&snap, "Golden:spinner-basic:spinner");
    assert_close_px(
        "spinner-basic width",
        spinner.bounds.size.width,
        web_spinner.rect.w,
        1.0,
    );
    assert_close_px(
        "spinner-basic height",
        spinner.bounds.size.height,
        web_spinner.rect.h,
        1.0,
    );
}
#[test]
fn web_vs_fret_layout_spinner_custom_geometry_matches_web() {
    let web = read_web_golden("spinner-custom");
    let theme = web_theme(&web);
    let web_spinner = find_first(&theme.root, &|n| {
        n.tag == "svg" && class_has_token(n, "animate-spin")
    })
    .expect("web spinner svg");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let spinner = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("Golden:spinner-custom:spinner")),
                ..Default::default()
            },
            move |cx| vec![fret_ui_shadcn::Spinner::new().speed(0.0).into_element(cx)],
        );
        vec![spinner]
    });

    let spinner = find_by_test_id(&snap, "Golden:spinner-custom:spinner");
    assert_close_px(
        "spinner-custom width",
        spinner.bounds.size.width,
        web_spinner.rect.w,
        1.0,
    );
    assert_close_px(
        "spinner-custom height",
        spinner.bounds.size.height,
        web_spinner.rect.h,
        1.0,
    );
}
#[test]
fn web_vs_fret_layout_spinner_size_variants_match_web() {
    let web = read_web_golden("spinner-size");
    let theme = web_theme(&web);
    let mut web_spinners = find_all(&theme.root, &|n| {
        n.tag == "svg" && class_has_token(n, "animate-spin")
    });
    web_spinners.sort_by(|a, b| a.rect.w.total_cmp(&b.rect.w));
    assert_eq!(web_spinners.len(), 4, "expected 4 web spinners");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let sizes = [Px(12.0), Px(16.0), Px(24.0), Px(32.0)];
        let mut out = Vec::new();
        for (i, size) in sizes.into_iter().enumerate() {
            let id = Arc::from(format!("Golden:spinner-size:{i}"));
            let layout = LayoutRefinement::default()
                .w_px(MetricRef::Px(size))
                .h_px(MetricRef::Px(size));
            out.push(cx.semantics(
                fret_ui::element::SemanticsProps {
                    role: SemanticsRole::Panel,
                    test_id: Some(id),
                    ..Default::default()
                },
                move |cx| {
                    vec![
                        fret_ui_shadcn::Spinner::new()
                            .refine_layout(layout)
                            .speed(0.0)
                            .into_element(cx),
                    ]
                },
            ));
        }
        out
    });

    for (i, web_spinner) in web_spinners.iter().enumerate() {
        let id = format!("Golden:spinner-size:{i}");
        let spinner = find_by_test_id(&snap, &id);
        assert_close_px(
            &format!("spinner-size[{i}] width"),
            spinner.bounds.size.width,
            web_spinner.rect.w,
            1.0,
        );
        assert_close_px(
            &format!("spinner-size[{i}] height"),
            spinner.bounds.size.height,
            web_spinner.rect.h,
            1.0,
        );
    }
}
#[test]
fn web_vs_fret_layout_spinner_color_sizes_match_web() {
    let web = read_web_golden("spinner-color");
    let theme = web_theme(&web);
    let web_spinners = find_all(&theme.root, &|n| {
        n.tag == "svg" && class_has_token(n, "animate-spin")
    });
    assert_eq!(web_spinners.len(), 5, "expected 5 web spinners");
    for (i, s) in web_spinners.iter().enumerate() {
        assert_close_px(
            &format!("spinner-color[{i}] width"),
            Px(s.rect.w),
            24.0,
            0.5,
        );
        assert_close_px(
            &format!("spinner-color[{i}] height"),
            Px(s.rect.h),
            24.0,
            0.5,
        );
    }
}
#[test]
fn web_vs_fret_layout_spinner_button_disabled_sm_heights_match_web() {
    let web = read_web_golden("spinner-button");
    let theme = web_theme(&web);

    let mut web_buttons = find_all(&theme.root, &|n| n.tag == "button");
    web_buttons.sort_by(|a, b| a.rect.y.total_cmp(&b.rect.y));
    assert_eq!(web_buttons.len(), 3, "expected 3 web buttons");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let buttons = vec![
            fret_ui_shadcn::Button::new("Loading...")
                .size(fret_ui_shadcn::ButtonSize::Sm)
                .disabled(true)
                .test_id("Golden:spinner-button:btn-0")
                .children(vec![
                    fret_ui_shadcn::Spinner::new().speed(0.0).into_element(cx),
                ])
                .into_element(cx),
            fret_ui_shadcn::Button::new("Please wait")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .size(fret_ui_shadcn::ButtonSize::Sm)
                .disabled(true)
                .test_id("Golden:spinner-button:btn-1")
                .children(vec![
                    fret_ui_shadcn::Spinner::new().speed(0.0).into_element(cx),
                ])
                .into_element(cx),
            fret_ui_shadcn::Button::new("Processing")
                .variant(fret_ui_shadcn::ButtonVariant::Secondary)
                .size(fret_ui_shadcn::ButtonSize::Sm)
                .disabled(true)
                .test_id("Golden:spinner-button:btn-2")
                .children(vec![
                    fret_ui_shadcn::Spinner::new().speed(0.0).into_element(cx),
                ])
                .into_element(cx),
        ];

        vec![cx.column(
            ColumnProps {
                layout: fret_ui_kit::declarative::style::layout_style(
                    &Theme::global(&*cx.app),
                    LayoutRefinement::default().w_full(),
                ),
                gap: MetricRef::space(Space::N4).resolve(&Theme::global(&*cx.app)),
                ..Default::default()
            },
            move |_cx| buttons,
        )]
    });

    for (i, web_button) in web_buttons.iter().enumerate() {
        let id = format!("Golden:spinner-button:btn-{i}");
        let btn = find_by_test_id(&snap, &id);
        assert_close_px(
            &format!("spinner-button[{i}] height"),
            btn.bounds.size.height,
            web_button.rect.h,
            1.0,
        );
    }
}
#[test]
fn web_vs_fret_layout_spinner_badge_heights_match_web() {
    let web = read_web_golden("spinner-badge");
    let theme = web_theme(&web);

    let web_badges = web_find_badge_spans_with_spinner(&theme.root);
    assert_eq!(web_badges.len(), 3, "expected 3 web badges");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let badges = vec![
            fret_ui_shadcn::Badge::new("Syncing")
                .children(vec![
                    fret_ui_shadcn::Spinner::new().speed(0.0).into_element(cx),
                ])
                .refine_layout(LayoutRefinement::default())
                .into_element(cx),
            fret_ui_shadcn::Badge::new("Updating")
                .variant(fret_ui_shadcn::BadgeVariant::Secondary)
                .children(vec![
                    fret_ui_shadcn::Spinner::new().speed(0.0).into_element(cx),
                ])
                .into_element(cx),
            fret_ui_shadcn::Badge::new("Processing")
                .variant(fret_ui_shadcn::BadgeVariant::Outline)
                .children(vec![
                    fret_ui_shadcn::Spinner::new().speed(0.0).into_element(cx),
                ])
                .into_element(cx),
        ];

        let mut out = Vec::new();
        for (i, badge) in badges.into_iter().enumerate() {
            out.push(cx.semantics(
                fret_ui::element::SemanticsProps {
                    role: SemanticsRole::Panel,
                    test_id: Some(Arc::from(format!("Golden:spinner-badge:{i}"))),
                    ..Default::default()
                },
                move |_cx| vec![badge],
            ));
        }
        out
    });

    for (i, web_badge) in web_badges.iter().enumerate() {
        let id = format!("Golden:spinner-badge:{i}");
        let badge = find_by_test_id(&snap, &id);
        assert_close_px(
            &format!("spinner-badge[{i}] height"),
            badge.bounds.size.height,
            web_badge.rect.h,
            1.0,
        );
    }
}
#[test]
fn web_vs_fret_layout_spinner_demo_item_height_matches_web() {
    let web = read_web_golden("spinner-demo");
    let theme = web_theme(&web);

    let web_item = find_first(&theme.root, &|n| {
        n.tag == "div" && class_has_token(n, "group/item") && contains_text(n, "Processing payment")
    })
    .expect("web item");

    let web_media = find_first(web_item, &|n| {
        n.tag == "div" && class_has_all_tokens(n, &["flex", "shrink-0", "items-center", "gap-2"])
    })
    .expect("web item media");
    let web_content = find_first(web_item, &|n| {
        n.tag == "div" && class_has_all_tokens(n, &["flex", "flex-1", "flex-col", "gap-1"])
    })
    .expect("web item content");
    let web_price = find_first(web_item, &|n| {
        n.tag == "div" && class_has_all_tokens(n, &["flex", "flex-col", "flex-none", "justify-end"])
    })
    .expect("web item price container");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let wrapper_layout = fret_ui_kit::declarative::style::layout_style(
            &Theme::global(&*cx.app),
            LayoutRefinement::default()
                .w_full()
                .max_w(MetricRef::Px(Px(web_item.rect.w))),
        );
        let wrapper_gap = MetricRef::space(Space::N4).resolve(&Theme::global(&*cx.app));

        let item = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("Golden:spinner-demo:item")),
                ..Default::default()
            },
            move |cx| {
                let media = cx.semantics(
                    fret_ui::element::SemanticsProps {
                        role: SemanticsRole::Panel,
                        test_id: Some(Arc::from("Golden:spinner-demo:media")),
                        ..Default::default()
                    },
                    move |cx| {
                        vec![
                            fret_ui_shadcn::ItemMedia::new([fret_ui_shadcn::Spinner::new()
                                .speed(0.0)
                                .into_element(cx)])
                            .into_element(cx),
                        ]
                    },
                );

                let content = cx.semantics(
                    fret_ui::element::SemanticsProps {
                        role: SemanticsRole::Panel,
                        test_id: Some(Arc::from("Golden:spinner-demo:content")),
                        ..Default::default()
                    },
                    move |cx| {
                        vec![
                            fret_ui_shadcn::ItemContent::new([fret_ui_shadcn::ItemTitle::new(
                                "Processing payment...",
                            )
                            .into_element(cx)])
                            .into_element(cx),
                        ]
                    },
                );

                let price = cx.semantics(
                    fret_ui::element::SemanticsProps {
                        role: SemanticsRole::Panel,
                        test_id: Some(Arc::from("Golden:spinner-demo:price")),
                        ..Default::default()
                    },
                    move |cx| {
                        vec![
                            fret_ui_shadcn::ItemContent::new([ui::text(cx, "$100.00")
                                .text_size_px(Theme::global(&*cx.app).metric_required("font.size"))
                                .line_height_px(
                                    Theme::global(&*cx.app).metric_required("font.line_height"),
                                )
                                .into_element(cx)])
                            .justify(MainAlign::End)
                            .refine_layout(LayoutRefinement::default().flex_none())
                            .into_element(cx),
                        ]
                    },
                );

                let item = fret_ui_shadcn::Item::new([media, content, price])
                    .variant(fret_ui_shadcn::ItemVariant::Muted)
                    .into_element(cx);
                vec![item]
            },
        );

        vec![cx.column(
            ColumnProps {
                layout: wrapper_layout,
                gap: wrapper_gap,
                ..Default::default()
            },
            move |_cx| vec![item],
        )]
    });

    let item = find_by_test_id(&snap, "Golden:spinner-demo:item");
    assert_close_px(
        "spinner-demo item width",
        item.bounds.size.width,
        web_item.rect.w,
        2.0,
    );

    let media = find_by_test_id(&snap, "Golden:spinner-demo:media");
    assert_close_px(
        "spinner-demo media y",
        media.bounds.origin.y,
        web_media.rect.y,
        2.0,
    );

    let content = find_by_test_id(&snap, "Golden:spinner-demo:content");
    assert_close_px(
        "spinner-demo content y",
        content.bounds.origin.y,
        web_content.rect.y,
        2.0,
    );

    let price = find_by_test_id(&snap, "Golden:spinner-demo:price");
    assert_close_px(
        "spinner-demo price y",
        price.bounds.origin.y,
        web_price.rect.y,
        2.0,
    );

    assert_close_px(
        "spinner-demo item height",
        item.bounds.size.height,
        web_item.rect.h,
        2.0,
    );
}
#[test]
fn web_vs_fret_layout_spinner_item_height_matches_web() {
    let web = read_web_golden("spinner-item");
    let theme = web_theme(&web);

    let web_item = find_first(&theme.root, &|n| {
        n.tag == "div" && class_has_token(n, "group/item") && contains_text(n, "Downloading...")
    })
    .expect("web item");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let value: Model<f32> = cx.app.models_mut().insert(0.75);

        let item = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("Golden:spinner-item:item")),
                ..Default::default()
            },
            move |cx| {
                let item = fret_ui_shadcn::Item::new([
                    fret_ui_shadcn::ItemMedia::new([fret_ui_shadcn::Spinner::new()
                        .speed(0.0)
                        .into_element(cx)])
                    .variant(fret_ui_shadcn::ItemMediaVariant::Icon)
                    .into_element(cx),
                    fret_ui_shadcn::ItemContent::new([
                        fret_ui_shadcn::ItemTitle::new("Downloading...").into_element(cx),
                        fret_ui_shadcn::ItemDescription::new("129 MB / 1000 MB").into_element(cx),
                    ])
                    .into_element(cx),
                    fret_ui_shadcn::ItemActions::new([fret_ui_shadcn::Button::new("Cancel")
                        .variant(fret_ui_shadcn::ButtonVariant::Outline)
                        .size(fret_ui_shadcn::ButtonSize::Sm)
                        .into_element(cx)])
                    .into_element(cx),
                    fret_ui_shadcn::ItemFooter::new([
                        fret_ui_shadcn::Progress::new(value).into_element(cx)
                    ])
                    .into_element(cx),
                ])
                .variant(fret_ui_shadcn::ItemVariant::Outline)
                .into_element(cx);
                vec![item]
            },
        );
        vec![item]
    });

    let item = find_by_test_id(&snap, "Golden:spinner-item:item");
    assert_close_px(
        "spinner-item item height",
        item.bounds.size.height,
        web_item.rect.h,
        2.0,
    );
}
#[test]
fn web_vs_fret_layout_spinner_empty_icon_geometry_matches_web() {
    let web = read_web_golden("spinner-empty");
    let theme = web_theme(&web);

    let web_icon = find_first(&theme.root, &|n| {
        n.tag == "div" && class_has_all_tokens(n, &["mb-2", "size-10", "rounded-lg"])
    })
    .expect("web empty icon");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let mut services = StyleAwareServices::default();

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        |cx| {
            let empty = fret_ui_shadcn::Empty::new([
                EmptyHeader::new([
                    EmptyMedia::new([fret_ui_shadcn::Spinner::new().speed(0.0).into_element(cx)])
                        .variant(EmptyMediaVariant::Icon)
                        .into_element(cx),
                    EmptyTitle::new("Processing your request").into_element(cx),
                    EmptyDescription::new(
                        "Please wait while we process your request. Do not refresh the page.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
                EmptyContent::new([fret_ui_shadcn::Button::new("Cancel")
                    .variant(fret_ui_shadcn::ButtonVariant::Outline)
                    .size(fret_ui_shadcn::ButtonSize::Sm)
                    .into_element(cx)])
                .into_element(cx),
            ])
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx);

            vec![empty]
        },
    );

    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let expected_bg = web_icon
        .computed_style
        .get("backgroundColor")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web empty icon backgroundColor");

    let mut best: Option<(Rect, fret_core::Color, f32)> = None;
    for op in scene.ops() {
        let SceneOp::Quad {
            rect, background, ..
        } = *op
        else {
            continue;
        };

        if (rect.size.width.0 - web_icon.rect.w).abs() > 2.0 {
            continue;
        }
        if (rect.size.height.0 - web_icon.rect.h).abs() > 2.0 {
            continue;
        }

        let Some(background) = paint_solid_color(background) else {
            continue;
        };

        let diff = rgba_diff_metric(color_to_rgba(background), expected_bg);
        match best {
            Some((_best_rect, _best_bg, best_diff)) if diff >= best_diff => {}
            _ => best = Some((rect, background, diff)),
        }
    }

    let (rect, bg, _diff) = best.unwrap_or_else(|| {
        debug_dump_scene_quads_near_expected(&scene, web_icon.rect, Some(expected_bg));
        panic!("spinner-empty: missing icon background quad near expected size");
    });
    assert_close_px(
        "spinner-empty icon width",
        rect.size.width,
        web_icon.rect.w,
        1.0,
    );
    assert_close_px(
        "spinner-empty icon height",
        rect.size.height,
        web_icon.rect.h,
        1.0,
    );
    assert_rgba_close(
        "spinner-empty icon background",
        color_to_rgba(bg),
        expected_bg,
        0.02,
    );
}
