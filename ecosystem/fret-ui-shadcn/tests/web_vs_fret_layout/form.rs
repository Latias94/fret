use super::*;

#[derive(Debug, Clone, Deserialize)]
struct FixtureSuite<T> {
    schema_version: u32,
    cases: Vec<T>,
}

#[derive(Debug, Clone, Deserialize)]
struct LayoutBugReportFormDemoCase {
    id: String,
    web_name: String,
}

fn web_find_by_id<'a>(root: &'a WebNode, id: &str) -> Option<&'a WebNode> {
    find_first(root, &|n| {
        n.id.as_deref() == Some(id) || n.attrs.get("id").is_some_and(|v| v == id)
    })
}

fn web_find_by_tag_and_text_within<'a>(
    root: &'a WebNode,
    within: WebRect,
    tag: &str,
    text: &str,
) -> Option<&'a WebNode> {
    find_first(root, &|n| {
        n.tag == tag && n.text.as_deref() == Some(text) && rect_contains(within, n.rect)
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FormControlKind {
    Input,
    Textarea,
}

fn assert_bug_report_form_demo_geometry_matches_web(web_name: &str) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let web_card = web_find_by_class_tokens(
        &theme.root,
        &[
            "bg-card",
            "text-card-foreground",
            "rounded-xl",
            "border",
            "py-6",
            "sm:max-w-md",
        ],
    )
    .expect("web card root");

    let web_title_input = find_all(&theme.root, &|n| n.tag == "input")
        .into_iter()
        .filter(|n| rect_contains(web_card.rect, n.rect))
        .min_by(|a, b| {
            a.rect
                .y
                .partial_cmp(&b.rect.y)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .expect("web title input");

    let web_description_group = find_all(&theme.root, &|n| {
        n.tag == "div"
            && n.class_name
                .as_deref()
                .is_some_and(|c| c.contains("group/input-group"))
    })
    .into_iter()
    .filter(|n| rect_contains(web_card.rect, n.rect))
    .min_by(|a, b| {
        a.rect
            .y
            .partial_cmp(&b.rect.y)
            .unwrap_or(std::cmp::Ordering::Equal)
    })
    .expect("web description input-group");

    let web_reset = web_find_by_tag_and_text_within(&theme.root, web_card.rect, "button", "Reset")
        .expect("web reset button");
    let web_submit =
        web_find_by_tag_and_text_within(&theme.root, web_card.rect, "button", "Submit")
            .expect("web submit button");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let title: Model<String> = cx.app.models_mut().insert(String::new());
        let description: Model<String> = cx.app.models_mut().insert(String::new());

        let title_field = fret_ui_shadcn::Field::new(vec![
            fret_ui_shadcn::FieldLabel::new("Bug Title").into_element(cx),
            cx.semantics(
                fret_ui::element::SemanticsProps {
                    role: SemanticsRole::TextField,
                    label: Some(Arc::from(format!("Golden:{web_name}:title"))),
                    ..Default::default()
                },
                move |cx| {
                    vec![
                        fret_ui_shadcn::Input::new(title)
                            .a11y_label("Bug Title")
                            .placeholder("Bug title")
                            .into_element(cx),
                    ]
                },
            ),
        ])
        .into_element(cx);

        let description_group = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from(format!("Golden:{web_name}:description_group"))),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::InputGroup::new(description)
                        .textarea()
                        .a11y_label("Description")
                        .textarea_min_height(Px(96.0))
                        .block_end(vec![
                            fret_ui_shadcn::InputGroupText::new("0/100 characters")
                                .into_element(cx),
                        ])
                        .into_element(cx),
                ]
            },
        );

        let description_field = fret_ui_shadcn::Field::new(vec![
            fret_ui_shadcn::FieldLabel::new("Description").into_element(cx),
            description_group,
            fret_ui_shadcn::FieldDescription::new(
                "Include steps to reproduce, expected behavior, and what actually happened.",
            )
            .into_element(cx),
        ])
        .into_element(cx);

        let card = fret_ui_shadcn::Card::new(vec![
            fret_ui_shadcn::CardHeader::new(vec![
                fret_ui_shadcn::CardTitle::new("Bug Report").into_element(cx),
                fret_ui_shadcn::CardDescription::new(
                    "Help us improve by reporting bugs you encounter.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            fret_ui_shadcn::CardContent::new(vec![
                fret_ui_shadcn::FieldGroup::new(vec![title_field, description_field])
                    .into_element(cx),
            ])
            .into_element(cx),
            fret_ui_shadcn::CardFooter::new(vec![cx.row(
                RowProps {
                    layout: LayoutStyle::default(),
                    gap: fret_ui_kit::MetricRef::space(Space::N2).resolve(&Theme::global(&*cx.app)),
                    justify: MainAlign::End,
                    align: CrossAlign::Center,
                    ..Default::default()
                },
                move |cx| {
                    vec![
                        fret_ui_shadcn::Button::new("Reset")
                            .variant(fret_ui_shadcn::ButtonVariant::Outline)
                            .into_element(cx),
                        fret_ui_shadcn::Button::new("Submit").into_element(cx),
                    ]
                },
            )])
            .into_element(cx),
        ])
        .refine_layout(
            fret_ui_kit::LayoutRefinement::default()
                .w_px(fret_ui_kit::MetricRef::Px(Px(web_card.rect.w))),
        )
        .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from(format!("Golden:{web_name}:card"))),
                ..Default::default()
            },
            move |_cx| vec![card],
        )]
    });

    let card = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some(&format!("Golden:{web_name}:card")),
    )
    .expect("fret card");
    let title = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some(&format!("Golden:{web_name}:title")),
    )
    .expect("fret title input");
    let description_group = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some(&format!("Golden:{web_name}:description_group")),
    )
    .expect("fret description group");
    let reset =
        find_semantics(&snap, SemanticsRole::Button, Some("Reset")).expect("fret reset button");
    let submit =
        find_semantics(&snap, SemanticsRole::Button, Some("Submit")).expect("fret submit button");

    assert_close_px("card width", card.bounds.size.width, web_card.rect.w, 1.0);
    assert_rect_xwh_close_px("title input", title.bounds, web_title_input.rect, 1.0);
    assert_rect_xwh_close_px(
        "description input-group",
        description_group.bounds,
        web_description_group.rect,
        1.0,
    );
    assert_close_px(
        "reset button height",
        reset.bounds.size.height,
        web_reset.rect.h,
        1.0,
    );
    assert_close_px(
        "submit button height",
        submit.bounds.size.height,
        web_submit.rect.h,
        1.0,
    );
}

fn assert_single_field_form_card_geometry_matches_web(
    web_name: &str,
    title: &str,
    description: &str,
    field_label: &str,
    field_description: &str,
    control_id: &str,
    control_kind: FormControlKind,
    primary_action: &str,
) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let web_card = web_find_by_class_tokens(
        &theme.root,
        &[
            "bg-card",
            "text-card-foreground",
            "rounded-xl",
            "border",
            "py-6",
            "sm:max-w-md",
        ],
    )
    .expect("web card root");

    let web_control = match control_kind {
        FormControlKind::Input => web_find_by_id(&theme.root, control_id).expect("web input"),
        FormControlKind::Textarea => web_find_by_id(&theme.root, control_id).expect("web textarea"),
    };

    let web_reset = web_find_by_tag_and_text_within(&theme.root, web_card.rect, "button", "Reset")
        .expect("web reset button");
    let web_primary =
        web_find_by_tag_and_text_within(&theme.root, web_card.rect, "button", primary_action)
            .expect("web primary button");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());

        let control = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::TextField,
                label: Some(Arc::from(format!("Golden:{web_name}:control"))),
                ..Default::default()
            },
            move |cx| {
                vec![match control_kind {
                    FormControlKind::Input => fret_ui_shadcn::Input::new(model)
                        .a11y_label(field_label)
                        .placeholder("...")
                        .into_element(cx),
                    FormControlKind::Textarea => fret_ui_shadcn::Textarea::new(model)
                        .a11y_label(field_label)
                        .min_height(Px(120.0))
                        .into_element(cx),
                }]
            },
        );

        let field = fret_ui_shadcn::Field::new(vec![
            fret_ui_shadcn::FieldLabel::new(field_label).into_element(cx),
            control,
            fret_ui_shadcn::FieldDescription::new(field_description).into_element(cx),
        ])
        .into_element(cx);

        let card = fret_ui_shadcn::Card::new(vec![
            fret_ui_shadcn::CardHeader::new(vec![
                fret_ui_shadcn::CardTitle::new(title).into_element(cx),
                fret_ui_shadcn::CardDescription::new(description).into_element(cx),
            ])
            .into_element(cx),
            fret_ui_shadcn::CardContent::new(vec![
                fret_ui_shadcn::FieldGroup::new(vec![field]).into_element(cx),
            ])
            .into_element(cx),
            fret_ui_shadcn::CardFooter::new(vec![cx.row(
                RowProps {
                    layout: LayoutStyle::default(),
                    gap: fret_ui_kit::MetricRef::space(Space::N2).resolve(&Theme::global(&*cx.app)),
                    justify: MainAlign::End,
                    align: CrossAlign::Center,
                    ..Default::default()
                },
                move |cx| {
                    vec![
                        fret_ui_shadcn::Button::new("Reset")
                            .variant(fret_ui_shadcn::ButtonVariant::Outline)
                            .into_element(cx),
                        fret_ui_shadcn::Button::new(primary_action).into_element(cx),
                    ]
                },
            )])
            .into_element(cx),
        ])
        .refine_layout(
            fret_ui_kit::LayoutRefinement::default()
                .w_px(fret_ui_kit::MetricRef::Px(Px(web_card.rect.w))),
        )
        .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from(format!("Golden:{web_name}:card"))),
                ..Default::default()
            },
            move |_cx| vec![card],
        )]
    });

    let card = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some(&format!("Golden:{web_name}:card")),
    )
    .expect("fret card");
    let control = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some(&format!("Golden:{web_name}:control")),
    )
    .expect("fret control");
    let reset =
        find_semantics(&snap, SemanticsRole::Button, Some("Reset")).expect("fret reset button");
    let primary = find_semantics(&snap, SemanticsRole::Button, Some(primary_action))
        .expect("fret primary button");

    assert_close_px("card width", card.bounds.size.width, web_card.rect.w, 1.0);
    assert_rect_xwh_close_px("control", control.bounds, web_control.rect, 1.0);
    assert_close_px(
        "reset button height",
        reset.bounds.size.height,
        web_reset.rect.h,
        1.0,
    );
    assert_close_px(
        "primary button height",
        primary.bounds.size.height,
        web_primary.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_form_bug_report_demo_geometry_matches_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/layout_form_bug_report_demo_cases_v1.json"
    ));
    let suite: FixtureSuite<LayoutBugReportFormDemoCase> =
        serde_json::from_str(raw).expect("layout bug report form demo fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!(
            "layout bug report form demo case={} web_name={}",
            case.id, case.web_name
        );
        assert_bug_report_form_demo_geometry_matches_web(&case.web_name);
    }
}

#[test]
fn web_vs_fret_layout_form_rhf_input_geometry_matches_web() {
    assert_single_field_form_card_geometry_matches_web(
        "form-rhf-input",
        "Profile Settings",
        "Update your profile information below.",
        "Username",
        "This is your public display name. Must be between 3 and 10 characters. Must only contain letters, numbers, and underscores.",
        "form-rhf-input-username",
        FormControlKind::Input,
        "Save",
    );
}

#[test]
fn web_vs_fret_layout_form_tanstack_input_geometry_matches_web() {
    assert_single_field_form_card_geometry_matches_web(
        "form-tanstack-input",
        "Profile Settings",
        "Update your profile information below.",
        "Username",
        "This is your public display name. Must be between 3 and 10 characters. Must only contain letters, numbers, and underscores.",
        "form-tanstack-input-username",
        FormControlKind::Input,
        "Save",
    );
}

#[test]
fn web_vs_fret_layout_form_rhf_textarea_geometry_matches_web() {
    assert_single_field_form_card_geometry_matches_web(
        "form-rhf-textarea",
        "Personalization",
        "Customize your experience by telling us more about yourself.",
        "More about you",
        "Tell us more about yourself. This will be used to help us personalize your experience.",
        "form-rhf-textarea-about",
        FormControlKind::Textarea,
        "Save",
    );
}

#[test]
fn web_vs_fret_layout_form_tanstack_textarea_geometry_matches_web() {
    assert_single_field_form_card_geometry_matches_web(
        "form-tanstack-textarea",
        "Personalization",
        "Customize your experience by telling us more about yourself.",
        "More about you",
        "Tell us more about yourself. This will be used to help us personalize your experience.",
        "form-tanstack-textarea-about",
        FormControlKind::Textarea,
        "Save",
    );
}

fn web_find_input_group_container_containing<'a>(
    theme: &'a WebGoldenTheme,
    input: &WebNode,
) -> &'a WebNode {
    let mut all = Vec::new();
    web_collect_all(&theme.root, &mut all);
    all.into_iter()
        .filter(|n| {
            n.tag == "div"
                && n.class_name
                    .as_deref()
                    .is_some_and(|c| c.contains("group/input-group"))
                && rect_contains(n.rect, input.rect)
        })
        .min_by(|a, b| {
            let area_a = a.rect.w * a.rect.h;
            let area_b = b.rect.w * b.rect.h;
            area_a
                .partial_cmp(&area_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .expect("web input-group container")
}

fn form_trailing_icon_button<H: fret_ui::UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    a11y_label: &str,
) -> fret_ui::element::AnyElement {
    let icon = cx.semantics(
        fret_ui::element::SemanticsProps {
            role: SemanticsRole::Panel,
            label: Some(Arc::from(format!("{a11y_label}:icon"))),
            ..Default::default()
        },
        move |cx| vec![decl_icon::icon(cx, fret_icons::ids::ui::SEARCH)],
    );

    fret_ui_shadcn::InputGroupButton::new("")
        .variant(fret_ui_shadcn::ButtonVariant::Ghost)
        .size(fret_ui_shadcn::InputGroupButtonSize::IconXs)
        .a11y_label(a11y_label)
        .children(vec![icon])
        .into_element(cx)
}

fn assert_form_input_group_control_geometry_matches_web(
    web_name: &str,
    title: &str,
    description: &str,
    input_id: &str,
) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let web_card = web_find_by_class_tokens(
        &theme.root,
        &[
            "bg-card",
            "text-card-foreground",
            "rounded-xl",
            "border",
            "py-6",
            "sm:max-w-md",
        ],
    )
    .expect("web card root");

    let web_input = web_find_by_id(&theme.root, input_id).expect("web input");
    let web_group = web_find_input_group_container_containing(theme, web_input);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());

        let group = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from(format!("Golden:{web_name}:group"))),
                ..Default::default()
            },
            move |cx| {
                let trailing = form_trailing_icon_button(cx, &format!("Golden:{web_name}:trail"));
                vec![
                    fret_ui_shadcn::InputGroup::new(model)
                        .trailing(vec![trailing])
                        .trailing_has_button(true)
                        .a11y_label("Golden:form-input-group")
                        .into_element(cx),
                ]
            },
        );

        let card = fret_ui_shadcn::Card::new(vec![
            fret_ui_shadcn::CardHeader::new(vec![
                fret_ui_shadcn::CardTitle::new(title).into_element(cx),
                fret_ui_shadcn::CardDescription::new(description).into_element(cx),
            ])
            .into_element(cx),
            fret_ui_shadcn::CardContent::new(vec![group]).into_element(cx),
        ])
        .refine_layout(
            fret_ui_kit::LayoutRefinement::default()
                .w_px(fret_ui_kit::MetricRef::Px(Px(web_card.rect.w))),
        )
        .into_element(cx);

        vec![card]
    });

    let group = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some(&format!("Golden:{web_name}:group")),
    )
    .expect("fret input-group");

    assert_rect_xwh_close_px("input-group", group.bounds, web_group.rect, 1.0);
}

fn assert_form_checkbox_control_size_matches_web(web_name: &str, checkbox_id: &str) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);
    let web_checkbox = web_find_by_id(&theme.root, checkbox_id).expect("web checkbox");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let checked: Model<bool> = cx.app.models_mut().insert(true);
        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from(format!("Golden:{web_name}:checkbox"))),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::Checkbox::new(checked)
                        .disabled(true)
                        .a11y_label("Golden:form-checkbox")
                        .into_element(cx),
                ]
            },
        )]
    });

    let checkbox = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some(&format!("Golden:{web_name}:checkbox")),
    )
    .expect("fret checkbox wrapper");

    assert_close_px(
        "checkbox w",
        checkbox.bounds.size.width,
        web_checkbox.rect.w,
        1.0,
    );
    assert_close_px(
        "checkbox h",
        checkbox.bounds.size.height,
        web_checkbox.rect.h,
        1.0,
    );
}

fn assert_form_switch_control_size_matches_web(web_name: &str, switch_id: &str) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);
    let web_switch = web_find_by_id(&theme.root, switch_id).expect("web switch");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let checked: Model<bool> = cx.app.models_mut().insert(false);
        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from(format!("Golden:{web_name}:switch"))),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::Switch::new(checked)
                        .a11y_label("Golden:form-switch")
                        .into_element(cx),
                ]
            },
        )]
    });

    let switch = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some(&format!("Golden:{web_name}:switch")),
    )
    .expect("fret switch wrapper");

    assert_close_px("switch w", switch.bounds.size.width, web_switch.rect.w, 1.0);
    assert_close_px(
        "switch h",
        switch.bounds.size.height,
        web_switch.rect.h,
        1.0,
    );
}

fn assert_form_select_control_size_matches_web(web_name: &str, select_id: &str) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);
    let web_select = web_find_by_id(&theme.root, select_id).expect("web select trigger");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
        let open: Model<bool> = cx.app.models_mut().insert(false);
        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from(format!("Golden:{web_name}:select"))),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::Select::new(value, open)
                        .placeholder("Select")
                        .items([
                            fret_ui_shadcn::SelectItem::new("auto", "Auto"),
                            fret_ui_shadcn::SelectItem::new("english", "English"),
                            fret_ui_shadcn::SelectItem::new("spanish", "Spanish"),
                        ])
                        .refine_layout(
                            LayoutRefinement::default().w_px(MetricRef::Px(Px(web_select.rect.w))),
                        )
                        .into_element(cx),
                ]
            },
        )]
    });

    let select = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some(&format!("Golden:{web_name}:select")),
    )
    .expect("fret select wrapper");

    assert_close_px("select w", select.bounds.size.width, web_select.rect.w, 1.0);
    assert_close_px(
        "select h",
        select.bounds.size.height,
        web_select.rect.h,
        1.0,
    );
}

fn assert_form_radio_control_size_matches_web(web_name: &str, radio_id: &str) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);
    let web_radio = web_find_by_id(&theme.root, radio_id).expect("web radio");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let (ui, _snap, root) = run_fret_root_with_ui(bounds, |cx| {
        let group = fret_ui_shadcn::RadioGroup::uncontrolled(Some("other"))
            .item(fret_ui_shadcn::RadioGroupItem::new("starter", "Starter"))
            .a11y_label("Golden:form-radio-group")
            .into_element(cx);
        vec![group]
    });

    let radio_bounds =
        find_node_with_size_close(&ui, root, web_radio.rect.w, web_radio.rect.h, 1.0)
            .expect("missing fret radio indicator bounds");

    assert_close_px("radio w", radio_bounds.size.width, web_radio.rect.w, 1.0);
    assert_close_px("radio h", radio_bounds.size.height, web_radio.rect.h, 1.0);
}

#[test]
fn web_vs_fret_layout_form_rhf_array_geometry_matches_web() {
    assert_form_input_group_control_geometry_matches_web(
        "form-rhf-array",
        "Contact Emails",
        "Manage your contact email addresses.",
        "form-rhf-array-email-0",
    );
}

#[test]
fn web_vs_fret_layout_form_tanstack_array_geometry_matches_web() {
    assert_form_input_group_control_geometry_matches_web(
        "form-tanstack-array",
        "Contact Emails",
        "Manage your contact email addresses.",
        "form-tanstack-array-email-0",
    );
}

#[test]
fn web_vs_fret_layout_form_rhf_password_geometry_matches_web() {
    assert_form_input_group_control_geometry_matches_web(
        "form-rhf-password",
        "Create Password",
        "Choose a strong password to secure your account.",
        "form-rhf-password-input",
    );
}

#[test]
fn web_vs_fret_layout_form_rhf_checkbox_geometry_matches_web() {
    assert_form_checkbox_control_size_matches_web(
        "form-rhf-checkbox",
        "form-rhf-checkbox-responses",
    );
}

#[test]
fn web_vs_fret_layout_form_tanstack_checkbox_geometry_matches_web() {
    assert_form_checkbox_control_size_matches_web(
        "form-tanstack-checkbox",
        "form-tanstack-checkbox-responses",
    );
}

#[test]
fn web_vs_fret_layout_form_rhf_switch_geometry_matches_web() {
    assert_form_switch_control_size_matches_web("form-rhf-switch", "form-rhf-switch-twoFactor");
}

#[test]
fn web_vs_fret_layout_form_tanstack_switch_geometry_matches_web() {
    assert_form_switch_control_size_matches_web(
        "form-tanstack-switch",
        "form-tanstack-switch-twoFactor",
    );
}

#[test]
fn web_vs_fret_layout_form_rhf_select_geometry_matches_web() {
    assert_form_select_control_size_matches_web("form-rhf-select", "form-rhf-select-language");
}

#[test]
fn web_vs_fret_layout_form_tanstack_select_geometry_matches_web() {
    assert_form_select_control_size_matches_web(
        "form-tanstack-select",
        "form-tanstack-select-language",
    );
}

#[test]
fn web_vs_fret_layout_form_rhf_radiogroup_geometry_matches_web() {
    assert_form_radio_control_size_matches_web(
        "form-rhf-radiogroup",
        "form-rhf-radiogroup-starter",
    );
}

#[test]
fn web_vs_fret_layout_form_tanstack_radiogroup_geometry_matches_web() {
    assert_form_radio_control_size_matches_web(
        "form-tanstack-radiogroup",
        "form-tanstack-radiogroup-starter",
    );
}

#[test]
fn web_vs_fret_layout_form_rhf_complex_geometry_matches_web() {
    assert_form_radio_control_size_matches_web("form-rhf-complex", "form-rhf-complex-basic");
}

#[test]
fn web_vs_fret_layout_form_tanstack_complex_geometry_matches_web() {
    assert_form_radio_control_size_matches_web("form-tanstack-complex", "basic");
}
