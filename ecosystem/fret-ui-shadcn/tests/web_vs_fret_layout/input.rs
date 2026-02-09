use super::*;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum LayoutInputRecipe {
    InputDemoGeometry,
    InputDisabledGeometry,
    InputFileGeometry,
    InputWithButtonGeometry,
    InputWithTextGeometry,
    InputGroupLabelGeometry,
    InputGroupButtonGroupGeometry,

    InputOtpRowRelativeGeometry,
    InputOtpDemoGeometry,
    InputOtpSeparatorGeometry,
    InputOtpPatternGeometry,
    InputOtpControlledGeometry,

    CommandDemoInputHeight,
    CommandDemoListboxHeight,
    CommandDemoListboxOptionHeight,
    CommandDemoListboxOptionInsets,

    InputWithLabelGeometry,
    InputGroupDropdownHeight,
    InputGroupIconGeometry,
    InputGroupSpinnerGeometry,
    InputGroupButtonGeometry,
    InputGroupTooltipGeometry,
    EmptyInputGroupGeometry,
    KbdInputGroupGeometry,
    InputGroupTextareaGeometry,
    InputGroupTextCurrencyGeometry,
    InputGroupTextUrlGeometry,
    InputGroupTextEmailGeometry,
    InputGroupTextTextareaCountGeometry,
    InputGroupCustomGeometry,
    InputGroupDemoBlockEndGeometry,
}

#[derive(Debug, Clone, Deserialize)]
struct LayoutInputCase {
    id: String,
    web_name: String,
    recipe: LayoutInputRecipe,
    #[serde(default)]
    row_tokens: Vec<String>,
}

#[test]
fn web_vs_fret_layout_input_geometry_matches_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/layout_input_cases_v1.json"
    ));
    let suite: FixtureSuite<LayoutInputCase> =
        serde_json::from_str(raw).expect("layout input fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("layout input case={}", case.id);
        match case.recipe {
            LayoutInputRecipe::InputDemoGeometry => {
                assert_eq!(case.web_name, "input-demo");
                web_vs_fret_layout_input_demo_geometry();
            }
            LayoutInputRecipe::InputDisabledGeometry => {
                assert_eq!(case.web_name, "input-disabled");
                web_vs_fret_layout_input_disabled_geometry_matches();
            }
            LayoutInputRecipe::InputFileGeometry => {
                assert_eq!(case.web_name, "input-file");
                web_vs_fret_layout_input_file_geometry_matches();
            }
            LayoutInputRecipe::InputWithButtonGeometry => {
                assert_eq!(case.web_name, "input-with-button");
                web_vs_fret_layout_input_with_button_geometry_matches();
            }
            LayoutInputRecipe::InputWithTextGeometry => {
                assert_eq!(case.web_name, "input-with-text");
                web_vs_fret_layout_input_with_text_geometry_matches();
            }
            LayoutInputRecipe::InputGroupLabelGeometry => {
                assert_eq!(case.web_name, "input-group-label");
                web_vs_fret_layout_input_group_label_geometry_matches();
            }
            LayoutInputRecipe::InputGroupButtonGroupGeometry => {
                assert_eq!(case.web_name, "input-group-button-group");
                web_vs_fret_layout_input_group_button_group_geometry_matches();
            }

            LayoutInputRecipe::InputOtpRowRelativeGeometry => {
                assert!(
                    case.row_tokens.len() >= 2,
                    "expected row_tokens in otp row case"
                );
                let row_tokens: Vec<&str> = case.row_tokens.iter().map(|s| s.as_str()).collect();
                assert_input_otp_block_relative_geometry_matches_web(&case.web_name, &row_tokens);
            }
            LayoutInputRecipe::InputOtpDemoGeometry => {
                assert_eq!(case.web_name, "input-otp-demo");
                web_vs_fret_layout_input_otp_demo_geometry_matches();
            }
            LayoutInputRecipe::InputOtpSeparatorGeometry => {
                assert_eq!(case.web_name, "input-otp-separator");
                web_vs_fret_layout_input_otp_separator_geometry_matches();
            }
            LayoutInputRecipe::InputOtpPatternGeometry => {
                assert_eq!(case.web_name, "input-otp-pattern");
                web_vs_fret_layout_input_otp_pattern_geometry_matches();
            }
            LayoutInputRecipe::InputOtpControlledGeometry => {
                assert_eq!(case.web_name, "input-otp-controlled");
                web_vs_fret_layout_input_otp_controlled_geometry_matches();
            }

            LayoutInputRecipe::CommandDemoInputHeight => {
                assert_eq!(case.web_name, "command-demo");
                web_vs_fret_layout_command_demo_input_height_matches();
            }
            LayoutInputRecipe::CommandDemoListboxHeight => {
                assert_eq!(case.web_name, "command-demo");
                web_vs_fret_layout_command_demo_listbox_height_matches();
            }
            LayoutInputRecipe::CommandDemoListboxOptionHeight => {
                assert_eq!(case.web_name, "command-demo");
                web_vs_fret_layout_command_demo_listbox_option_height_matches();
            }
            LayoutInputRecipe::CommandDemoListboxOptionInsets => {
                assert_eq!(case.web_name, "command-demo");
                web_vs_fret_layout_command_demo_listbox_option_insets_match();
            }

            LayoutInputRecipe::InputWithLabelGeometry => {
                assert_eq!(case.web_name, "input-with-label");
                web_vs_fret_layout_input_with_label_geometry();
            }
            LayoutInputRecipe::InputGroupDropdownHeight => {
                assert_eq!(case.web_name, "input-group-dropdown");
                web_vs_fret_layout_input_group_dropdown_height();
            }
            LayoutInputRecipe::InputGroupIconGeometry => {
                assert_eq!(case.web_name, "input-group-icon");
                web_vs_fret_layout_input_group_icon_geometry_matches();
            }
            LayoutInputRecipe::InputGroupSpinnerGeometry => {
                assert_eq!(case.web_name, "input-group-spinner");
                web_vs_fret_layout_input_group_spinner_geometry_matches();
            }
            LayoutInputRecipe::InputGroupButtonGeometry => {
                assert_eq!(case.web_name, "input-group-button");
                web_vs_fret_layout_input_group_button_geometry_matches();
            }
            LayoutInputRecipe::InputGroupTooltipGeometry => {
                assert_eq!(case.web_name, "input-group-tooltip");
                web_vs_fret_layout_input_group_tooltip_geometry_matches();
            }
            LayoutInputRecipe::EmptyInputGroupGeometry => {
                assert_eq!(case.web_name, "empty-input-group");
                web_vs_fret_layout_empty_input_group_geometry_matches();
            }
            LayoutInputRecipe::KbdInputGroupGeometry => {
                assert_eq!(case.web_name, "kbd-input-group");
                web_vs_fret_layout_kbd_input_group_geometry_matches();
            }
            LayoutInputRecipe::InputGroupTextareaGeometry => {
                assert_eq!(case.web_name, "input-group-textarea");
                web_vs_fret_layout_input_group_textarea_geometry_matches();
            }
            LayoutInputRecipe::InputGroupTextCurrencyGeometry => {
                assert_eq!(case.web_name, "input-group-text");
                web_vs_fret_layout_input_group_text_currency_geometry_matches();
            }
            LayoutInputRecipe::InputGroupTextUrlGeometry => {
                assert_eq!(case.web_name, "input-group-text");
                web_vs_fret_layout_input_group_text_url_geometry_matches();
            }
            LayoutInputRecipe::InputGroupTextEmailGeometry => {
                assert_eq!(case.web_name, "input-group-text");
                web_vs_fret_layout_input_group_text_email_geometry_matches();
            }
            LayoutInputRecipe::InputGroupTextTextareaCountGeometry => {
                assert_eq!(case.web_name, "input-group-text");
                web_vs_fret_layout_input_group_text_textarea_count_geometry_matches();
            }
            LayoutInputRecipe::InputGroupCustomGeometry => {
                assert_eq!(case.web_name, "input-group-custom");
                web_vs_fret_layout_input_group_custom_geometry_matches();
            }
            LayoutInputRecipe::InputGroupDemoBlockEndGeometry => {
                assert_eq!(case.web_name, "input-group-demo");
                web_vs_fret_layout_input_group_demo_block_end_geometry_matches();
            }
        }
    }
}

fn web_vs_fret_layout_input_demo_geometry() {
    let web = read_web_golden("input-demo");
    let theme = web_theme(&web);
    let web_input = find_first(&theme.root, &|n| n.tag == "input").expect("web input");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());
        vec![
            fret_ui_shadcn::Input::new(model)
                .a11y_label("Input")
                .into_element(cx),
        ]
    });

    let input = find_semantics(&snap, SemanticsRole::TextField, Some("Input"))
        .or_else(|| find_semantics(&snap, SemanticsRole::TextField, None))
        .expect("fret input semantics node");

    assert_close_px(
        "input width",
        input.bounds.size.width,
        web_input.rect.w,
        1.0,
    );
    assert_close_px(
        "input height",
        input.bounds.size.height,
        web_input.rect.h,
        1.0,
    );
}

fn web_vs_fret_layout_input_disabled_geometry_matches() {
    let web = read_web_golden("input-disabled");
    let theme = web_theme(&web);
    let web_input = find_first(&theme.root, &|n| n.tag == "input").expect("web input");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());
        vec![
            fret_ui_shadcn::Input::new(model)
                .disabled(true)
                .a11y_label("Golden:input-disabled:input")
                .into_element(cx),
        ]
    });

    let input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-disabled:input"),
    )
    .expect("fret disabled input semantics node");

    assert_close_px(
        "input-disabled width",
        input.bounds.size.width,
        web_input.rect.w,
        1.0,
    );
    assert_close_px(
        "input-disabled height",
        input.bounds.size.height,
        web_input.rect.h,
        1.0,
    );
}

fn web_vs_fret_layout_input_file_geometry_matches() {
    let web = read_web_golden("input-file");
    let theme = web_theme(&web);

    let web_label = web_find_by_tag_and_text(&theme.root, "label", "Picture").expect("web label");
    let web_input = find_first(&theme.root, &|n| n.tag == "input").expect("web input");

    let expected_gap_y = web_input.rect.y - (web_label.rect.y + web_label.rect.h);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());

        let label = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:input-file:label")),
                ..Default::default()
            },
            move |cx| vec![fret_ui_shadcn::Label::new("Picture").into_element(cx)],
        );

        let input = fret_ui_shadcn::Input::new(model)
            .a11y_label("Golden:input-file:input")
            .into_element(cx);

        let col = cx.flex(
            FlexProps {
                layout: decl_style::layout_style(
                    &Theme::global(&*cx.app),
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(web_input.rect.w)))
                        .min_w_0(),
                ),
                direction: fret_core::Axis::Vertical,
                gap: Px(expected_gap_y),
                padding: fret_core::Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Start,
                wrap: false,
            },
            move |_cx| vec![label, input],
        );

        vec![col]
    });

    let label = find_semantics(&snap, SemanticsRole::Panel, Some("Golden:input-file:label"))
        .expect("fret label");
    let input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-file:input"),
    )
    .expect("fret input");

    assert_close_px(
        "input-file label h",
        label.bounds.size.height,
        web_label.rect.h,
        1.0,
    );
    assert_close_px(
        "input-file input w",
        input.bounds.size.width,
        web_input.rect.w,
        1.0,
    );
    assert_close_px(
        "input-file input h",
        input.bounds.size.height,
        web_input.rect.h,
        1.0,
    );

    let gap_y = input.bounds.origin.y.0 - (label.bounds.origin.y.0 + label.bounds.size.height.0);
    assert_close_px("input-file gap", Px(gap_y), expected_gap_y, 1.0);
}

fn web_vs_fret_layout_input_with_button_geometry_matches() {
    let web = read_web_golden("input-with-button");
    let theme = web_theme(&web);

    let web_input = find_first(&theme.root, &|n| n.tag == "input").expect("web input");
    let web_button =
        web_find_by_tag_and_text(&theme.root, "button", "Subscribe").expect("web button");

    let expected_gap_x = web_button.rect.x - (web_input.rect.x + web_input.rect.w);
    let expected_row_w = (web_button.rect.x + web_button.rect.w) - web_input.rect.x;
    let web_button_w = web_button.rect.w;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());

        let input = fret_ui_shadcn::Input::new(model)
            .a11y_label("Golden:input-with-button:input")
            .into_element(cx);

        let button = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:input-with-button:button")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::Button::new("Subscribe")
                        .variant(fret_ui_shadcn::ButtonVariant::Outline)
                        .refine_layout(
                            LayoutRefinement::default().w_px(MetricRef::Px(Px(web_button_w))),
                        )
                        .into_element(cx),
                ]
            },
        );

        let row = cx.flex(
            FlexProps {
                layout: decl_style::layout_style(
                    &Theme::global(&*cx.app),
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(expected_row_w)))
                        .min_w_0(),
                ),
                direction: fret_core::Axis::Horizontal,
                gap: Px(expected_gap_x),
                padding: fret_core::Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |_cx| vec![input, button],
        );

        vec![row]
    });

    let input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-with-button:input"),
    )
    .expect("fret input");
    let button = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-with-button:button"),
    )
    .expect("fret button wrapper");

    assert_close_px(
        "input-with-button input h",
        input.bounds.size.height,
        web_input.rect.h,
        1.0,
    );
    assert_close_px(
        "input-with-button button w",
        button.bounds.size.width,
        web_button.rect.w,
        1.0,
    );
    assert_close_px(
        "input-with-button button h",
        button.bounds.size.height,
        web_button.rect.h,
        1.0,
    );
    assert_close_px(
        "input-with-button input w",
        input.bounds.size.width,
        web_input.rect.w,
        1.0,
    );
}

fn web_vs_fret_layout_input_with_text_geometry_matches() {
    let web = read_web_golden("input-with-text");
    let theme = web_theme(&web);

    let web_label = web_find_by_tag_and_text(&theme.root, "label", "Email").expect("web label");
    let web_input = find_first(&theme.root, &|n| n.tag == "input").expect("web input");
    let web_p = web_find_by_tag_and_text(&theme.root, "p", "Enter your email address.")
        .expect("web helper text");

    let gap0 = web_input.rect.y - (web_label.rect.y + web_label.rect.h);
    let gap1 = web_p.rect.y - (web_input.rect.y + web_input.rect.h);
    let web_label_h = web_label.rect.h;
    let web_p_h = web_p.rect.h;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());

        let label = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:input-with-text:label")),
                ..Default::default()
            },
            move |cx| {
                vec![cx.container(
                    ContainerProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Fill,
                                height: Length::Px(Px(web_label_h)),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    move |cx| vec![fret_ui_shadcn::Label::new("Email").into_element(cx)],
                )]
            },
        );

        let input = fret_ui_shadcn::Input::new(model)
            .a11y_label("Golden:input-with-text:input")
            .into_element(cx);

        let helper = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:input-with-text:helper")),
                ..Default::default()
            },
            move |cx| {
                vec![cx.container(
                    ContainerProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Fill,
                                height: Length::Px(Px(web_p_h)),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    move |cx| vec![decl_text::text_sm(cx, "Enter your email address.")],
                )]
            },
        );

        let col = cx.flex(
            FlexProps {
                layout: decl_style::layout_style(
                    &Theme::global(&*cx.app),
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(web_input.rect.w)))
                        .min_w_0(),
                ),
                direction: fret_core::Axis::Vertical,
                gap: Px(gap0),
                padding: fret_core::Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Start,
                wrap: false,
            },
            move |_cx| vec![label, input, helper],
        );

        vec![col]
    });

    let label = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-with-text:label"),
    )
    .expect("fret label");
    let input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-with-text:input"),
    )
    .expect("fret input");
    let helper = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-with-text:helper"),
    )
    .expect("fret helper");

    assert_close_px(
        "input-with-text label h",
        label.bounds.size.height,
        web_label.rect.h,
        1.0,
    );
    assert_close_px(
        "input-with-text input w",
        input.bounds.size.width,
        web_input.rect.w,
        1.0,
    );
    assert_close_px(
        "input-with-text input h",
        input.bounds.size.height,
        web_input.rect.h,
        1.0,
    );
    assert_close_px(
        "input-with-text helper h",
        helper.bounds.size.height,
        web_p.rect.h,
        1.0,
    );

    let gap0_fret =
        input.bounds.origin.y.0 - (label.bounds.origin.y.0 + label.bounds.size.height.0);
    let gap1_fret =
        helper.bounds.origin.y.0 - (input.bounds.origin.y.0 + input.bounds.size.height.0);
    assert_close_px("input-with-text gap0", Px(gap0_fret), gap0, 1.0);
    assert_close_px("input-with-text gap1", Px(gap1_fret), gap1, 1.0);
}

fn web_vs_fret_layout_input_group_label_geometry_matches() {
    let web = read_web_golden("input-group-label");
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

    let web_input0 = find_first(web_group0, &|n| n.tag == "input").expect("web input0");
    let web_input1 = find_first(web_group1, &|n| n.tag == "input").expect("web input1");
    let web_addon_label0 = find_first(web_group0, &|n| n.tag == "label").expect("web label0");
    let web_addon_label0_w = web_addon_label0.rect.w;

    let expected_gap_y = web_group1.rect.y - (web_group0.rect.y + web_group0.rect.h);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model0: Model<String> = cx.app.models_mut().insert(String::new());
        let model1: Model<String> = cx.app.models_mut().insert(String::new());

        let group0 = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:input-group-label:0:root")),
                ..Default::default()
            },
            move |cx| {
                let addon = cx.semantics(
                    fret_ui::element::SemanticsProps {
                        role: SemanticsRole::Panel,
                        label: Some(Arc::from("Golden:input-group-label:0:addon")),
                        ..Default::default()
                    },
                    move |cx| {
                        let label = fret_ui_shadcn::Label::new("@").into_element(cx);
                        vec![
                            cx.container(
                                ContainerProps {
                                    layout: decl_style::layout_style(
                                        &Theme::global(&*cx.app),
                                        LayoutRefinement::default()
                                            .w_px(MetricRef::Px(Px(web_addon_label0_w)))
                                            .min_w_0(),
                                    ),
                                    ..Default::default()
                                },
                                move |_cx| vec![label],
                            ),
                        ]
                    },
                );

                vec![
                    fret_ui_shadcn::InputGroup::new(model0.clone())
                        .a11y_label("Golden:input-group-label:0:input")
                        .leading(vec![addon])
                        .into_element(cx),
                ]
            },
        );

        let group1 = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:input-group-label:1:root")),
                ..Default::default()
            },
            move |cx| {
                let info_icon = cx.semantics(
                    fret_ui::element::SemanticsProps {
                        role: SemanticsRole::Panel,
                        label: Some(Arc::from("Golden:input-group-label:1:icon")),
                        ..Default::default()
                    },
                    move |cx| vec![decl_icon::icon(cx, IconId::new_static("lucide.info"))],
                );

                let help_button = fret_ui_shadcn::InputGroupButton::new("")
                    .variant(fret_ui_shadcn::ButtonVariant::Ghost)
                    .size(fret_ui_shadcn::InputGroupButtonSize::IconXs)
                    .children(vec![info_icon])
                    .refine_style(ChromeRefinement::default().rounded(Radius::Full))
                    .into_element(cx);

                let header_row = cx.flex(
                    FlexProps {
                        layout: decl_style::layout_style(
                            &Theme::global(&*cx.app),
                            LayoutRefinement::default().w_full().min_w_0(),
                        ),
                        direction: fret_core::Axis::Horizontal,
                        gap: Px(8.0),
                        padding: fret_core::Edges::all(Px(0.0)),
                        justify: MainAlign::SpaceBetween,
                        align: CrossAlign::Center,
                        wrap: false,
                    },
                    move |cx| {
                        vec![
                            fret_ui_shadcn::Label::new("Email").into_element(cx),
                            help_button,
                        ]
                    },
                );

                vec![
                    fret_ui_shadcn::InputGroup::new(model1.clone())
                        .a11y_label("Golden:input-group-label:1:input")
                        .block_start(vec![header_row])
                        .into_element(cx),
                ]
            },
        );

        let col = cx.flex(
            FlexProps {
                layout: decl_style::layout_style(
                    &Theme::global(&*cx.app),
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(web_group0.rect.w)))
                        .min_w_0(),
                ),
                direction: fret_core::Axis::Vertical,
                gap: Px(expected_gap_y),
                padding: fret_core::Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Start,
                wrap: false,
            },
            move |_cx| vec![group0, group1],
        );

        vec![col]
    });

    let fret_group0 = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-label:0:root"),
    )
    .expect("fret group0");
    let fret_input0 = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-group-label:0:input"),
    )
    .expect("fret input0");

    let fret_group1 = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-label:1:root"),
    )
    .expect("fret group1");
    let fret_input1 = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-group-label:1:input"),
    )
    .expect("fret input1");

    assert_close_px(
        "input-group-label group0 h",
        fret_group0.bounds.size.height,
        web_group0.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-label input0 w",
        fret_input0.bounds.size.width,
        web_input0.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-label group1 h",
        fret_group1.bounds.size.height,
        web_group1.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-label input1 w",
        fret_input1.bounds.size.width,
        web_input1.rect.w,
        1.0,
    );

    let gap_y = fret_group1.bounds.origin.y.0
        - (fret_group0.bounds.origin.y.0 + fret_group0.bounds.size.height.0);
    assert_close_px("input-group-label gap", Px(gap_y), expected_gap_y, 1.0);
}

fn web_vs_fret_layout_input_group_button_group_geometry_matches() {
    let web = read_web_golden("input-group-button-group");
    let theme = web_theme(&web);

    let web_group = web_find_by_class_tokens(
        &theme.root,
        &[
            "flex",
            "w-fit",
            "items-stretch",
            "[&>*:not(:first-child)]:border-l-0",
        ],
    )
    .expect("web button-group");
    let web_input_group =
        web_find_by_class_tokens(&theme.root, &["group/input-group", "border-input"])
            .expect("web input-group");
    let web_input = find_first(web_input_group, &|n| n.tag == "input").expect("web input");

    let web_prefix = find_first(web_group, &|n| {
        class_has_token(n, "bg-muted") && contains_text(n, "https://")
    })
    .expect("web prefix");
    let web_suffix = find_first(web_group, &|n| {
        class_has_token(n, "bg-muted") && contains_text(n, ".com")
    })
    .expect("web suffix");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());

        let icon = decl_icon::icon(cx, IconId::new_static("lucide.link-2"));

        let input_group = fret_ui_shadcn::InputGroup::new(model)
            .a11y_label("Golden:input-group-button-group:input")
            .trailing(vec![icon]);

        let group = fret_ui_shadcn::ButtonGroup::new(vec![
            ButtonGroupText::new("https://")
                .refine_layout(
                    LayoutRefinement::default().w_px(MetricRef::Px(Px(web_prefix.rect.w))),
                )
                .into(),
            input_group.into(),
            ButtonGroupText::new(".com")
                .refine_layout(
                    LayoutRefinement::default().w_px(MetricRef::Px(Px(web_suffix.rect.w))),
                )
                .into(),
        ])
        .a11y_label("Golden:input-group-button-group:group")
        .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(web_group.rect.w))));

        vec![group.into_element(cx)]
    });

    let group = find_semantics(
        &snap,
        SemanticsRole::Group,
        Some("Golden:input-group-button-group:group"),
    )
    .expect("fret button-group");
    let input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-group-button-group:input"),
    )
    .expect("fret input");

    assert_close_px(
        "input-group-button-group group w",
        group.bounds.size.width,
        web_group.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-button-group group h",
        group.bounds.size.height,
        web_group.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-button-group input w",
        input.bounds.size.width,
        web_input.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-button-group input h",
        input.bounds.size.height,
        web_input.rect.h,
        1.0,
    );
}

fn web_collect_input_otp_slots<'a>(root: &'a WebNode) -> Vec<&'a WebNode> {
    let mut slots = find_all(root, &|n| {
        n.tag == "div"
            && n.class_name.as_deref().is_some_and(|c| {
                c.split_whitespace().any(|t| t == "h-9")
                    && c.split_whitespace().any(|t| t == "w-9")
                    && c.split_whitespace().any(|t| t == "border-input")
            })
    });
    slots.sort_by(|a, b| {
        a.rect
            .x
            .partial_cmp(&b.rect.x)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.rect
                    .y
                    .partial_cmp(&b.rect.y)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
    slots
}

fn web_collect_input_otp_separators<'a>(root: &'a WebNode) -> Vec<&'a WebNode> {
    let mut seps = find_all(root, &|n| {
        n.tag == "div" && n.attrs.get("role").is_some_and(|v| v == "separator")
    });
    seps.sort_by(|a, b| {
        a.rect
            .x
            .partial_cmp(&b.rect.x)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    seps
}

fn web_find_leftmost_by_class_tokens<'a>(root: &'a WebNode, tokens: &[&str]) -> &'a WebNode {
    let mut nodes = find_all(root, &|n| class_has_all_tokens(n, tokens));
    nodes.sort_by(|a, b| {
        a.rect
            .x
            .partial_cmp(&b.rect.x)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.rect
                    .y
                    .partial_cmp(&b.rect.y)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
    nodes[0]
}

fn web_collect_input_otp_slots_by_border_input<'a>(root: &'a WebNode) -> Vec<&'a WebNode> {
    let mut slots = find_all(root, &|n| {
        n.tag == "div"
            && n.class_name
                .as_deref()
                .is_some_and(|c| c.split_whitespace().any(|t| t == "border-input"))
    });
    slots.sort_by(|a, b| {
        a.rect
            .x
            .partial_cmp(&b.rect.x)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.rect
                    .y
                    .partial_cmp(&b.rect.y)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
    slots
}

fn fret_collect_rects_by_size(
    snap: &fret_core::SemanticsSnapshot,
    w: Px,
    h: Px,
    tol: f32,
) -> Vec<Rect> {
    let mut rects: Vec<Rect> = snap
        .nodes
        .iter()
        .filter(|n| {
            (n.bounds.size.width.0 - w.0).abs() <= tol
                && (n.bounds.size.height.0 - h.0).abs() <= tol
        })
        .map(|n| n.bounds)
        .collect();
    rects.sort_by(|a, b| {
        a.origin
            .x
            .0
            .partial_cmp(&b.origin.x.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.origin
                    .y
                    .0
                    .partial_cmp(&b.origin.y.0)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
    rects.dedup_by(|a, b| {
        (a.origin.x.0 - b.origin.x.0).abs() <= tol
            && (a.origin.y.0 - b.origin.y.0).abs() <= tol
            && (a.size.width.0 - b.size.width.0).abs() <= tol
            && (a.size.height.0 - b.size.height.0).abs() <= tol
    });
    rects
}

fn assert_input_otp_slots_match_web(
    name: &str,
    web_slots: &[&WebNode],
    fret_slots: &[Rect],
    tol: f32,
) {
    assert_eq!(
        fret_slots.len(),
        web_slots.len(),
        "{name}: expected {} slots, got {}",
        web_slots.len(),
        fret_slots.len()
    );
    for (idx, (w, f)) in web_slots.iter().zip(fret_slots.iter()).enumerate() {
        assert_close_px(&format!("{name} slot[{idx}] x"), f.origin.x, w.rect.x, tol);
        assert_close_px(&format!("{name} slot[{idx}] y"), f.origin.y, w.rect.y, tol);
        assert_close_px(
            &format!("{name} slot[{idx}] w"),
            f.size.width,
            w.rect.w,
            tol,
        );
        assert_close_px(
            &format!("{name} slot[{idx}] h"),
            f.size.height,
            w.rect.h,
            tol,
        );
    }
}

fn assert_input_otp_separators_match_web(
    name: &str,
    web_seps: &[&WebNode],
    fret_seps: &[Rect],
    tol: f32,
) {
    assert_eq!(
        fret_seps.len(),
        web_seps.len(),
        "{name}: expected {} separators, got {}",
        web_seps.len(),
        fret_seps.len()
    );
    for (idx, (w, f)) in web_seps.iter().zip(fret_seps.iter()).enumerate() {
        assert_close_px(&format!("{name} sep[{idx}] x"), f.origin.x, w.rect.x, tol);
        assert_close_px(&format!("{name} sep[{idx}] y"), f.origin.y, w.rect.y, tol);
        assert_close_px(&format!("{name} sep[{idx}] w"), f.size.width, w.rect.w, tol);
        assert_close_px(
            &format!("{name} sep[{idx}] h"),
            f.size.height,
            w.rect.h,
            tol,
        );
    }
}

fn assert_input_otp_slots_relative_to_container_match_web(
    name: &str,
    web_container: &WebNode,
    web_slots: &[&WebNode],
    fret_container: &Rect,
    fret_slots: &[Rect],
    tol: f32,
) {
    assert_eq!(
        fret_slots.len(),
        web_slots.len(),
        "{name}: expected {} slots, got {}",
        web_slots.len(),
        fret_slots.len()
    );
    for (idx, (w, f)) in web_slots.iter().zip(fret_slots.iter()).enumerate() {
        let web_dx = w.rect.x - web_container.rect.x;
        let web_dy = w.rect.y - web_container.rect.y;

        let fret_dx = f.origin.x - fret_container.origin.x;
        let fret_dy = f.origin.y - fret_container.origin.y;

        assert_close_px(&format!("{name} slot[{idx}] dx"), fret_dx, web_dx, tol);
        assert_close_px(&format!("{name} slot[{idx}] dy"), fret_dy, web_dy, tol);
        assert_close_px(
            &format!("{name} slot[{idx}] w"),
            f.size.width,
            w.rect.w,
            tol,
        );
        assert_close_px(
            &format!("{name} slot[{idx}] h"),
            f.size.height,
            w.rect.h,
            tol,
        );
    }
}

fn assert_input_otp_block_relative_geometry_matches_web(web_name: &str, row_tokens: &[&str]) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);
    let web_row = web_find_leftmost_by_class_tokens(&theme.root, row_tokens);
    let web_slots = web_collect_input_otp_slots_by_border_input(web_row);
    assert!(
        !web_slots.is_empty(),
        "{web_name}: expected input otp slots in web row"
    );

    let slot_w = web_slots[0].rect.w;
    let slot_h = web_slots[0].rect.h;
    let slot_gap = if web_slots.len() > 1 {
        (web_slots[1].rect.x - web_slots[0].rect.x - slot_w).max(0.0)
    } else {
        0.0
    };

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let label: Arc<str> = Arc::from(format!("Golden:{web_name}:otp-row"));
    let label_str: &str = &label;
    let snap = run_fret_root(bounds, |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());
        let otp = fret_ui_shadcn::InputOtp::new(model)
            .length(web_slots.len())
            .slot_gap_px(Px(slot_gap))
            .slot_size_px(Px(slot_w), Px(slot_h));
        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(label.clone()),
                ..Default::default()
            },
            move |cx| vec![otp.into_element(cx)],
        )]
    });

    let fret_row =
        find_semantics(&snap, SemanticsRole::Panel, Some(label_str)).expect("fret otp row");
    assert_close_px(
        &format!("{web_name} otp-row w"),
        fret_row.bounds.size.width,
        web_row.rect.w,
        1.0,
    );
    assert_close_px(
        &format!("{web_name} otp-row h"),
        fret_row.bounds.size.height,
        web_row.rect.h,
        1.0,
    );

    let fret_slots = fret_collect_rects_by_size(&snap, Px(slot_w), Px(slot_h), 1.0);
    assert_input_otp_slots_relative_to_container_match_web(
        web_name,
        web_row,
        &web_slots,
        &fret_row.bounds,
        &fret_slots,
        1.0,
    );
}

fn web_vs_fret_layout_input_otp_demo_geometry_matches() {
    let web = read_web_golden("input-otp-demo");
    let theme = web_theme(&web);
    let web_slots = web_collect_input_otp_slots(&theme.root);
    let web_seps = web_collect_input_otp_separators(&theme.root);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());
        vec![
            fret_ui_shadcn::InputOtp::new(model)
                .group_size(Some(3))
                .into_element(cx),
        ]
    });

    let fret_slots = fret_collect_rects_by_size(&snap, Px(36.0), Px(36.0), 1.0);
    let fret_seps = fret_collect_rects_by_size(&snap, Px(24.0), Px(24.0), 1.0);

    assert_input_otp_slots_match_web("input-otp-demo", &web_slots, &fret_slots, 1.0);
    assert_input_otp_separators_match_web("input-otp-demo", &web_seps, &fret_seps, 1.0);
}

fn web_vs_fret_layout_input_otp_separator_geometry_matches() {
    let web = read_web_golden("input-otp-separator");
    let theme = web_theme(&web);
    let web_slots = web_collect_input_otp_slots(&theme.root);
    let web_seps = web_collect_input_otp_separators(&theme.root);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());
        vec![
            fret_ui_shadcn::InputOtp::new(model)
                .group_size(Some(2))
                .into_element(cx),
        ]
    });

    let fret_slots = fret_collect_rects_by_size(&snap, Px(36.0), Px(36.0), 1.0);
    let fret_seps = fret_collect_rects_by_size(&snap, Px(24.0), Px(24.0), 1.0);

    assert_input_otp_slots_match_web("input-otp-separator", &web_slots, &fret_slots, 1.0);
    assert_input_otp_separators_match_web("input-otp-separator", &web_seps, &fret_seps, 1.0);
}

fn web_vs_fret_layout_input_otp_pattern_geometry_matches() {
    let web = read_web_golden("input-otp-pattern");
    let theme = web_theme(&web);
    let web_slots = web_collect_input_otp_slots(&theme.root);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());
        vec![
            fret_ui_shadcn::InputOtp::new(model)
                .numeric_only(false)
                .into_element(cx),
        ]
    });

    let fret_slots = fret_collect_rects_by_size(&snap, Px(36.0), Px(36.0), 1.0);
    assert_input_otp_slots_match_web("input-otp-pattern", &web_slots, &fret_slots, 1.0);
}

fn web_vs_fret_layout_input_otp_controlled_geometry_matches() {
    let web = read_web_golden("input-otp-controlled");
    let theme = web_theme(&web);
    let web_slots = web_collect_input_otp_slots(&theme.root);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());
        vec![fret_ui_shadcn::InputOtp::new(model).into_element(cx)]
    });

    let fret_slots = fret_collect_rects_by_size(&snap, Px(36.0), Px(36.0), 1.0);
    assert_input_otp_slots_match_web("input-otp-controlled", &web_slots, &fret_slots, 1.0);
}

fn command_demo_snapshot(theme: &WebGoldenTheme) -> fret_core::SemanticsSnapshot {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    run_fret_root_with_services(bounds, &mut services, |cx| {
        use fret_ui_shadcn::{
            CommandEntry, CommandGroup, CommandItem, CommandPalette, CommandSeparator,
        };

        let query: Model<String> = cx.app.models_mut().insert(String::new());

        let entries: Vec<CommandEntry> = vec![
            CommandGroup::new(vec![
                CommandItem::new("Calendar"),
                CommandItem::new("Search Emoji"),
                CommandItem::new("Calculator"),
            ])
            .heading("Suggestions")
            .into(),
            CommandSeparator::new().into(),
            CommandGroup::new(vec![
                CommandItem::new("Profile"),
                CommandItem::new("Billing"),
                CommandItem::new("Settings"),
            ])
            .heading("Settings")
            .into(),
        ];

        vec![
            CommandPalette::new(query, Vec::new())
                .entries(entries)
                .into_element(cx),
        ]
    })
}

fn web_vs_fret_layout_command_demo_input_height_matches() {
    let web = read_web_golden("command-demo");
    let theme = web_theme(&web);
    let web_input = find_first(&theme.root, &|n| {
        n.tag == "input" && n.attrs.get("role").is_some_and(|v| v == "combobox")
    })
    .expect("web command-demo combobox input");

    let snap = command_demo_snapshot(theme);
    let combobox = find_semantics(&snap, SemanticsRole::ComboBox, None)
        .unwrap_or_else(|| panic!("missing fret command-demo combobox"));

    assert_close_px(
        "command-demo input height",
        combobox.bounds.size.height,
        web_input.rect.h,
        2.0,
    );
}

fn web_vs_fret_layout_command_demo_listbox_height_matches() {
    let web = read_web_golden("command-demo");
    let theme = web_theme(&web);
    let web_listbox = find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|v| v == "listbox")
    })
    .expect("web command-demo listbox");

    let snap = command_demo_snapshot(theme);
    let listbox = find_semantics(&snap, SemanticsRole::ListBox, None)
        .unwrap_or_else(|| panic!("missing fret command-demo listbox"));

    assert_close_px(
        "command-demo listbox height",
        listbox.bounds.size.height,
        web_listbox.rect.h,
        2.0,
    );
}

fn web_vs_fret_layout_command_demo_listbox_option_height_matches() {
    let web = read_web_golden("command-demo");
    let theme = web_theme(&web);
    let web_listbox = find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|v| v == "listbox")
    })
    .expect("web command-demo listbox");

    let mut all = Vec::new();
    web_collect_all(&theme.root, &mut all);
    let web_heights: std::collections::BTreeSet<i32> = all
        .into_iter()
        .filter(|n| n.attrs.get("role").is_some_and(|v| v == "option"))
        .filter(|n| rect_contains(web_listbox.rect, n.rect))
        .map(|n| n.rect.h.round() as i32)
        .collect();
    assert!(
        web_heights.len() == 1,
        "command-demo expected uniform web option height; got {web_heights:?}"
    );

    let snap = command_demo_snapshot(theme);
    let listbox = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ListBox)
        .unwrap_or_else(|| panic!("missing fret command-demo listbox"));
    let fret_heights: std::collections::BTreeSet<i32> = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::ListBoxOption)
        .filter(|n| fret_rect_contains(listbox.bounds, n.bounds))
        .map(|n| n.bounds.size.height.0.round() as i32)
        .collect();
    assert!(
        fret_heights.len() == 1,
        "command-demo expected uniform fret option height; got {fret_heights:?}"
    );

    let expected_h = web_heights.iter().next().copied().unwrap_or_default() as f32;
    let actual_h = fret_heights.iter().next().copied().unwrap_or_default() as f32;
    assert_close_px("command-demo option height", Px(actual_h), expected_h, 1.0);
}

fn web_vs_fret_layout_command_demo_listbox_option_insets_match() {
    let web = read_web_golden("command-demo");
    let theme = web_theme(&web);
    let web_listbox = find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|v| v == "listbox")
    })
    .expect("web command-demo listbox");
    let expected = web_listbox_option_inset(theme, web_listbox);

    let snap = command_demo_snapshot(theme);
    let actual = fret_listbox_option_inset(&snap);
    assert_inset_quad_close("command-demo", actual, expected, 1.0);
}

fn web_vs_fret_layout_input_with_label_geometry() {
    let web = read_web_golden("input-with-label");
    let theme = web_theme(&web);
    let web_label = find_first(&theme.root, &|n| n.tag == "label").expect("web label");
    let web_input = find_first(&theme.root, &|n| n.tag == "input").expect("web input");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let (_ui, snap, _root) = run_fret_root_with_ui(bounds, |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());

        let label = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:input-with-label:label")),
                ..Default::default()
            },
            move |cx| vec![fret_ui_shadcn::Label::new("Email").into_element(cx)],
        );

        let input = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:input-with-label:input")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::Input::new(model)
                        .a11y_label("Email")
                        .placeholder("Email")
                        .into_element(cx),
                ]
            },
        );

        let col = cx.flex(
            FlexProps {
                layout: LayoutStyle::default(),
                direction: fret_core::Axis::Vertical,
                gap: Px(12.0),
                padding: fret_core::Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Start,
                wrap: false,
            },
            move |_cx| vec![label, input],
        );

        let container = cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Px(Px(web_label.rect.w)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            move |_cx| vec![col],
        );

        vec![container]
    });

    let label = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-with-label:label"),
    )
    .expect("fret label");
    let input = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-with-label:input"),
    )
    .expect("fret input");

    assert_close_px(
        "input-with-label label h",
        label.bounds.size.height,
        web_label.rect.h,
        1.0,
    );
    assert_close_px(
        "input-with-label input w",
        input.bounds.size.width,
        web_input.rect.w,
        1.0,
    );
    assert_close_px(
        "input-with-label input h",
        input.bounds.size.height,
        web_input.rect.h,
        1.0,
    );

    let gap_y = input.bounds.origin.y.0 - (label.bounds.origin.y.0 + label.bounds.size.height.0);
    assert_close_px(
        "input-with-label gap",
        Px(gap_y),
        web_input.rect.y - (web_label.rect.y + web_label.rect.h),
        1.0,
    );
}

fn web_vs_fret_layout_input_group_dropdown_height() {
    let web = read_web_golden("input-group-dropdown");
    let theme = web_theme(&web);
    let web_group = web_find_by_class_tokens(&theme.root, &["group/input-group", "border-input"])
        .expect("web input group root");

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

    let model: Model<String> = app.models_mut().insert(String::new());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout-input-group",
        |cx| {
            let container_layout = fret_ui_kit::LayoutRefinement::default().w_px(Px(384.0));
            let container = cx.container(
                fret_ui::element::ContainerProps {
                    layout: fret_ui_kit::declarative::style::layout_style(
                        &fret_ui::Theme::global(&*cx.app),
                        container_layout,
                    ),
                    ..Default::default()
                },
                move |cx| {
                    let group = fret_ui_shadcn::InputGroup::new(model.clone())
                        .a11y_label("Input group")
                        .into_element(cx);

                    vec![cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-dropdown:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![group],
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

    let group = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-dropdown:root"),
    )
    .expect("fret input group root");

    assert_close_px(
        "input group height",
        group.bounds.size.height,
        web_group.rect.h,
        1.0,
    );
}

fn web_vs_fret_layout_input_group_icon_geometry_matches() {
    let web = read_web_golden("input-group-icon");
    let theme = web_theme(&web);
    let web_group = web_find_by_class_tokens(&theme.root, &["group/input-group", "border-input"])
        .expect("web input group root");

    let web_input = find_first(web_group, &|n| n.tag == "input").expect("web input node");
    let web_svg = find_first(web_group, &|n| n.tag == "svg").expect("web svg node");

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

    let model: Model<String> = app.models_mut().insert(String::new());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout-input-group-icon",
        |cx| {
            let container_layout = fret_ui_kit::LayoutRefinement::default().w_px(Px(384.0));
            let container = cx.container(
                fret_ui::element::ContainerProps {
                    layout: fret_ui_kit::declarative::style::layout_style(
                        &fret_ui::Theme::global(&*cx.app),
                        container_layout,
                    ),
                    ..Default::default()
                },
                move |cx| {
                    let icon = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-icon:icon")),
                            ..Default::default()
                        },
                        move |cx| vec![decl_icon::icon(cx, fret_icons::ids::ui::SEARCH)],
                    );

                    let group = fret_ui_shadcn::InputGroup::new(model.clone())
                        .a11y_label("Golden:input-group-icon:input")
                        .leading(vec![icon])
                        .into_element(cx);

                    vec![cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-icon:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![group],
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

    let group = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-icon:root"),
    )
    .expect("fret input group root");
    let input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-group-icon:input"),
    )
    .expect("fret input");
    let icon = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-icon:icon"),
    )
    .expect("fret icon");

    assert_close_px(
        "input-group-icon group w",
        group.bounds.size.width,
        web_group.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-icon group h",
        group.bounds.size.height,
        web_group.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-icon input x",
        input.bounds.origin.x,
        web_input.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-icon input w",
        input.bounds.size.width,
        web_input.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-icon svg x",
        icon.bounds.origin.x,
        web_svg.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-icon svg y",
        icon.bounds.origin.y,
        web_svg.rect.y,
        1.0,
    );
    assert_close_px(
        "input-group-icon svg w",
        icon.bounds.size.width,
        web_svg.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-icon svg h",
        icon.bounds.size.height,
        web_svg.rect.h,
        1.0,
    );
}

fn web_vs_fret_layout_input_group_spinner_geometry_matches() {
    let web = read_web_golden("input-group-spinner");
    let theme = web_theme(&web);
    let web_group = web_find_by_class_tokens(&theme.root, &["group/input-group", "border-input"])
        .expect("web input group root");

    let web_input = find_first(web_group, &|n| n.tag == "input").expect("web input node");
    let web_svg = find_first(web_group, &|n| n.tag == "svg").expect("web svg node");

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

    let model: Model<String> = app.models_mut().insert(String::new());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout-input-group-spinner",
        |cx| {
            let container_layout = fret_ui_kit::LayoutRefinement::default().w_px(Px(384.0));
            let container = cx.container(
                fret_ui::element::ContainerProps {
                    layout: fret_ui_kit::declarative::style::layout_style(
                        &fret_ui::Theme::global(&*cx.app),
                        container_layout,
                    ),
                    ..Default::default()
                },
                move |cx| {
                    let spinner = fret_ui_shadcn::Spinner::new().speed(0.0).into_element(cx);
                    let spinner = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-spinner:spinner")),
                            ..Default::default()
                        },
                        move |_cx| vec![spinner],
                    );

                    let group = fret_ui_shadcn::InputGroup::new(model.clone())
                        .a11y_label("Golden:input-group-spinner:input")
                        .trailing(vec![spinner])
                        .into_element(cx);

                    vec![cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-spinner:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![group],
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

    let group = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-spinner:root"),
    )
    .expect("fret input group root");
    let input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-group-spinner:input"),
    )
    .expect("fret input");
    let spinner = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-spinner:spinner"),
    )
    .expect("fret spinner");

    assert_close_px(
        "input-group-spinner group w",
        group.bounds.size.width,
        web_group.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-spinner group h",
        group.bounds.size.height,
        web_group.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-spinner input x",
        input.bounds.origin.x,
        web_input.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-spinner input w",
        input.bounds.size.width,
        web_input.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-spinner svg x",
        spinner.bounds.origin.x,
        web_svg.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-spinner svg y",
        spinner.bounds.origin.y,
        web_svg.rect.y,
        1.0,
    );
    assert_close_px(
        "input-group-spinner svg w",
        spinner.bounds.size.width,
        web_svg.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-spinner svg h",
        spinner.bounds.size.height,
        web_svg.rect.h,
        1.0,
    );
}

fn web_vs_fret_layout_input_group_button_geometry_matches() {
    let web = read_web_golden("input-group-button");
    let theme = web_theme(&web);
    let web_group = web_find_by_class_tokens(&theme.root, &["group/input-group", "border-input"])
        .expect("web input group root");

    let web_input = web_group
        .children
        .iter()
        .find(|n| n.tag == "input")
        .expect("web input node");
    let web_addon = web_group
        .children
        .iter()
        .find(|n| {
            n.tag == "div"
                && n.computed_style
                    .get("marginRight")
                    .is_some_and(|v| v == "-7.2px")
        })
        .expect("web addon node");
    let web_svg = find_first(web_addon, &|n| n.tag == "svg").expect("web svg node");

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

    let model: Model<String> = app.models_mut().insert(String::new());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout-input-group-button",
        |cx| {
            let container_layout = fret_ui_kit::LayoutRefinement::default().w_px(Px(384.0));
            let container = cx.container(
                fret_ui::element::ContainerProps {
                    layout: fret_ui_kit::declarative::style::layout_style(
                        &fret_ui::Theme::global(&*cx.app),
                        container_layout,
                    ),
                    ..Default::default()
                },
                move |cx| {
                    let icon = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-button:icon")),
                            ..Default::default()
                        },
                        move |cx| vec![decl_icon::icon(cx, fret_icons::ids::ui::SEARCH)],
                    );

                    let button = fret_ui_shadcn::Button::new("")
                        .variant(fret_ui_shadcn::ButtonVariant::Ghost)
                        .children(vec![icon])
                        .refine_style(ChromeRefinement::default().p(Space::N0))
                        .refine_layout(
                            fret_ui_kit::LayoutRefinement::default()
                                .w_px(Px(24.0))
                                .h_px(Px(24.0)),
                        )
                        .into_element(cx);

                    let group = fret_ui_shadcn::InputGroup::new(model.clone())
                        .a11y_label("Golden:input-group-button:input")
                        .trailing_has_button(true)
                        .trailing(vec![button])
                        .into_element(cx);

                    vec![cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-button:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![group],
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

    let group = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-button:root"),
    )
    .expect("fret input group root");
    let input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-group-button:input"),
    )
    .expect("fret input");
    let icon = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-button:icon"),
    )
    .expect("fret icon");

    assert_close_px(
        "input-group-button group w",
        group.bounds.size.width,
        web_group.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-button group h",
        group.bounds.size.height,
        web_group.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-button input x",
        input.bounds.origin.x,
        web_input.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-button input w",
        input.bounds.size.width,
        web_input.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-button svg x",
        icon.bounds.origin.x,
        web_svg.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-button svg y",
        icon.bounds.origin.y,
        web_svg.rect.y,
        1.0,
    );
    assert_close_px(
        "input-group-button svg w",
        icon.bounds.size.width,
        web_svg.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-button svg h",
        icon.bounds.size.height,
        web_svg.rect.h,
        1.0,
    );
}

fn web_vs_fret_layout_input_group_tooltip_geometry_matches() {
    let web = read_web_golden("input-group-tooltip");
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
    let web_group2 = *web_groups.get(2).expect("web group 2");

    let web_group0_input = web_group0
        .children
        .iter()
        .find(|n| n.tag == "input")
        .expect("web group0 input");
    let web_group0_svg = find_first(web_group0, &|n| n.tag == "svg").expect("web group0 svg");

    let web_group2_input = web_group2
        .children
        .iter()
        .find(|n| n.tag == "input")
        .expect("web group2 input");
    let web_group2_svg = find_first(web_group2, &|n| n.tag == "svg").expect("web group2 svg");

    let expected_gap_y = web_groups[1].rect.y - (web_groups[0].rect.y + web_groups[0].rect.h);

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
    let model2: Model<String> = app.models_mut().insert(String::new());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout-input-group-tooltip",
        |cx| {
            let container_layout = fret_ui_kit::LayoutRefinement::default().w_px(Px(384.0));
            let container = cx.container(
                fret_ui::element::ContainerProps {
                    layout: fret_ui_kit::declarative::style::layout_style(
                        &fret_ui::Theme::global(&*cx.app),
                        container_layout,
                    ),
                    ..Default::default()
                },
                move |cx| {
                    let button_icon0 = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-tooltip:0:icon")),
                            ..Default::default()
                        },
                        move |cx| vec![decl_icon::icon(cx, fret_icons::ids::ui::SEARCH)],
                    );
                    let button0 = fret_ui_shadcn::Button::new("")
                        .variant(fret_ui_shadcn::ButtonVariant::Ghost)
                        .children(vec![button_icon0])
                        .refine_style(ChromeRefinement::default().p(Space::N0))
                        .refine_layout(
                            fret_ui_kit::LayoutRefinement::default()
                                .w_px(Px(24.0))
                                .h_px(Px(24.0)),
                        )
                        .into_element(cx);

                    let group0 = fret_ui_shadcn::InputGroup::new(model0.clone())
                        .a11y_label("Golden:input-group-tooltip:0:input")
                        .trailing_has_button(true)
                        .trailing(vec![button0])
                        .into_element(cx);
                    let group0 = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-tooltip:0:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![group0],
                    );

                    let group1_button = fret_ui_shadcn::Button::new("")
                        .variant(fret_ui_shadcn::ButtonVariant::Ghost)
                        .children(vec![decl_icon::icon(cx, fret_icons::ids::ui::SEARCH)])
                        .refine_style(ChromeRefinement::default().p(Space::N0))
                        .refine_layout(
                            fret_ui_kit::LayoutRefinement::default()
                                .w_px(Px(24.0))
                                .h_px(Px(24.0)),
                        )
                        .into_element(cx);
                    let group1 = fret_ui_shadcn::InputGroup::new(model1.clone())
                        .a11y_label("Golden:input-group-tooltip:1:input")
                        .trailing_has_button(true)
                        .trailing(vec![group1_button])
                        .into_element(cx);
                    let group1 = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-tooltip:1:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![group1],
                    );

                    let button_icon2 = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-tooltip:2:icon")),
                            ..Default::default()
                        },
                        move |cx| vec![decl_icon::icon(cx, fret_icons::ids::ui::SEARCH)],
                    );
                    let button2 = fret_ui_shadcn::Button::new("")
                        .variant(fret_ui_shadcn::ButtonVariant::Ghost)
                        .children(vec![button_icon2])
                        .refine_style(ChromeRefinement::default().p(Space::N0))
                        .refine_layout(
                            fret_ui_kit::LayoutRefinement::default()
                                .w_px(Px(24.0))
                                .h_px(Px(24.0)),
                        )
                        .into_element(cx);

                    let group2 = fret_ui_shadcn::InputGroup::new(model2.clone())
                        .a11y_label("Golden:input-group-tooltip:2:input")
                        .leading_has_button(true)
                        .leading(vec![button2])
                        .into_element(cx);
                    let group2 = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-tooltip:2:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![group2],
                    );

                    vec![cx.column(
                        ColumnProps {
                            gap: Px(expected_gap_y),
                            ..Default::default()
                        },
                        move |_cx| vec![group0, group1, group2],
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

    let fret_group0 = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-tooltip:0:root"),
    )
    .expect("fret group0");
    let fret_input0 = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-group-tooltip:0:input"),
    )
    .expect("fret input0");
    let fret_icon0 = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-tooltip:0:icon"),
    )
    .expect("fret icon0");

    assert_close_px(
        "input-group-tooltip group0 y",
        fret_group0.bounds.origin.y,
        web_group0.rect.y,
        1.0,
    );
    assert_close_px(
        "input-group-tooltip group0 w",
        fret_group0.bounds.size.width,
        web_group0.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-tooltip group0 h",
        fret_group0.bounds.size.height,
        web_group0.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-tooltip input0 x",
        fret_input0.bounds.origin.x,
        web_group0_input.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-tooltip input0 y",
        fret_input0.bounds.origin.y,
        web_group0_input.rect.y,
        1.0,
    );
    assert_close_px(
        "input-group-tooltip input0 w",
        fret_input0.bounds.size.width,
        web_group0_input.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-tooltip svg0 x",
        fret_icon0.bounds.origin.x,
        web_group0_svg.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-tooltip svg0 y",
        fret_icon0.bounds.origin.y,
        web_group0_svg.rect.y,
        1.0,
    );
    assert_close_px(
        "input-group-tooltip svg0 w",
        fret_icon0.bounds.size.width,
        web_group0_svg.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-tooltip svg0 h",
        fret_icon0.bounds.size.height,
        web_group0_svg.rect.h,
        1.0,
    );

    let fret_group2 = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-tooltip:2:root"),
    )
    .expect("fret group2");
    let fret_input2 = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-group-tooltip:2:input"),
    )
    .expect("fret input2");
    let fret_icon2 = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-tooltip:2:icon"),
    )
    .expect("fret icon2");

    assert_close_px(
        "input-group-tooltip group2 y",
        fret_group2.bounds.origin.y,
        web_group2.rect.y,
        1.0,
    );
    assert_close_px(
        "input-group-tooltip input2 x",
        fret_input2.bounds.origin.x,
        web_group2_input.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-tooltip input2 y",
        fret_input2.bounds.origin.y,
        web_group2_input.rect.y,
        1.0,
    );
    assert_close_px(
        "input-group-tooltip input2 w",
        fret_input2.bounds.size.width,
        web_group2_input.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-tooltip svg2 x",
        fret_icon2.bounds.origin.x,
        web_group2_svg.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-tooltip svg2 y",
        fret_icon2.bounds.origin.y,
        web_group2_svg.rect.y,
        1.0,
    );
    assert_close_px(
        "input-group-tooltip svg2 w",
        fret_icon2.bounds.size.width,
        web_group2_svg.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-tooltip svg2 h",
        fret_icon2.bounds.size.height,
        web_group2_svg.rect.h,
        1.0,
    );
}

fn web_vs_fret_layout_empty_input_group_geometry_matches() {
    let web = read_web_golden("empty-input-group");
    let theme = web_theme(&web);
    let web_group = web_find_by_class_tokens(&theme.root, &["group/input-group", "border-input"])
        .expect("web input group root");
    let web_input = find_first(web_group, &|n| n.tag == "input").expect("web input");
    let web_svg = find_first(web_group, &|n| n.tag == "svg").expect("web icon");
    let web_kbd = find_first(web_group, &|n| n.tag == "kbd").expect("web kbd");

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

    let model: Model<String> = app.models_mut().insert(String::new());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout-empty-input-group",
        |cx| {
            let container_layout =
                fret_ui_kit::LayoutRefinement::default().w_px(Px(web_group.rect.w));
            let container = cx.container(
                fret_ui::element::ContainerProps {
                    layout: fret_ui_kit::declarative::style::layout_style(
                        &fret_ui::Theme::global(&*cx.app),
                        container_layout,
                    ),
                    ..Default::default()
                },
                move |cx| {
                    let icon = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:empty-input-group:icon")),
                            ..Default::default()
                        },
                        move |cx| vec![decl_icon::icon(cx, fret_icons::ids::ui::SEARCH)],
                    );

                    let kbd = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:empty-input-group:kbd")),
                            ..Default::default()
                        },
                        move |cx| vec![fret_ui_shadcn::Kbd::new("/").into_element(cx)],
                    );

                    let group = fret_ui_shadcn::InputGroup::new(model.clone())
                        .a11y_label("Golden:empty-input-group:input")
                        .leading(vec![icon])
                        .trailing_has_kbd(true)
                        .trailing(vec![kbd])
                        .into_element(cx);

                    vec![cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:empty-input-group:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![group],
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

    let group = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:empty-input-group:root"),
    )
    .expect("fret input group root");
    let input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:empty-input-group:input"),
    )
    .expect("fret input");
    let icon = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:empty-input-group:icon"),
    )
    .expect("fret icon");
    let kbd = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:empty-input-group:kbd"),
    )
    .expect("fret kbd");

    assert_close_px(
        "empty-input-group group w",
        group.bounds.size.width,
        web_group.rect.w,
        1.0,
    );
    assert_close_px(
        "empty-input-group group h",
        group.bounds.size.height,
        web_group.rect.h,
        1.0,
    );

    assert_close_px(
        "empty-input-group input x",
        Px(input.bounds.origin.x.0 - group.bounds.origin.x.0),
        web_input.rect.x - web_group.rect.x,
        1.0,
    );
    assert_close_px(
        "empty-input-group input y",
        Px(input.bounds.origin.y.0 - group.bounds.origin.y.0),
        web_input.rect.y - web_group.rect.y,
        1.0,
    );
    assert_close_px(
        "empty-input-group input w",
        input.bounds.size.width,
        web_input.rect.w,
        1.0,
    );

    assert_close_px(
        "empty-input-group svg x",
        Px(icon.bounds.origin.x.0 - group.bounds.origin.x.0),
        web_svg.rect.x - web_group.rect.x,
        1.0,
    );
    assert_close_px(
        "empty-input-group svg y",
        Px(icon.bounds.origin.y.0 - group.bounds.origin.y.0),
        web_svg.rect.y - web_group.rect.y,
        1.0,
    );
    assert_close_px(
        "empty-input-group svg w",
        icon.bounds.size.width,
        web_svg.rect.w,
        1.0,
    );
    assert_close_px(
        "empty-input-group svg h",
        icon.bounds.size.height,
        web_svg.rect.h,
        1.0,
    );

    assert_close_px(
        "empty-input-group kbd x",
        Px(kbd.bounds.origin.x.0 - group.bounds.origin.x.0),
        web_kbd.rect.x - web_group.rect.x,
        1.0,
    );
    assert_close_px(
        "empty-input-group kbd y",
        Px(kbd.bounds.origin.y.0 - group.bounds.origin.y.0),
        web_kbd.rect.y - web_group.rect.y,
        1.0,
    );
    assert_close_px(
        "empty-input-group kbd w",
        kbd.bounds.size.width,
        web_kbd.rect.w,
        1.0,
    );
    assert_close_px(
        "empty-input-group kbd h",
        kbd.bounds.size.height,
        web_kbd.rect.h,
        1.0,
    );
}

fn web_vs_fret_layout_kbd_input_group_geometry_matches() {
    let web = read_web_golden("kbd-input-group");
    let theme = web_theme(&web);
    let web_group = web_find_by_class_tokens(&theme.root, &["group/input-group", "border-input"])
        .expect("web input group root");
    let web_input = find_first(web_group, &|n| n.tag == "input").expect("web input");
    let web_svg = find_first(web_group, &|n| n.tag == "svg").expect("web icon");

    let mut web_kbds = Vec::new();
    web_collect_tag(web_group, "kbd", &mut web_kbds);
    web_kbds.sort_by(|a, b| {
        a.rect
            .x
            .partial_cmp(&b.rect.x)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let web_kbd0 = *web_kbds.get(0).expect("web kbd0");
    let web_kbd1 = *web_kbds.get(1).expect("web kbd1");

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

    let model: Model<String> = app.models_mut().insert(String::new());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout-kbd-input-group",
        |cx| {
            let container_layout =
                fret_ui_kit::LayoutRefinement::default().w_px(Px(web_group.rect.w));
            let container = cx.container(
                fret_ui::element::ContainerProps {
                    layout: fret_ui_kit::declarative::style::layout_style(
                        &fret_ui::Theme::global(&*cx.app),
                        container_layout,
                    ),
                    ..Default::default()
                },
                move |cx| {
                    let icon = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:kbd-input-group:icon")),
                            ..Default::default()
                        },
                        move |cx| vec![decl_icon::icon(cx, fret_icons::ids::ui::SEARCH)],
                    );

                    let kbd0 = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:kbd-input-group:kbd0")),
                            ..Default::default()
                        },
                        move |cx| vec![fret_ui_shadcn::Kbd::new("⌘").into_element(cx)],
                    );
                    let kbd1 = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:kbd-input-group:kbd1")),
                            ..Default::default()
                        },
                        move |cx| vec![fret_ui_shadcn::Kbd::new("K").into_element(cx)],
                    );

                    let group = fret_ui_shadcn::InputGroup::new(model.clone())
                        .a11y_label("Golden:kbd-input-group:input")
                        .leading(vec![icon])
                        .trailing_has_kbd(true)
                        .trailing(vec![kbd0, kbd1])
                        .into_element(cx);

                    vec![cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:kbd-input-group:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![group],
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

    let group = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:kbd-input-group:root"),
    )
    .expect("fret input group root");
    let input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:kbd-input-group:input"),
    )
    .expect("fret input");
    let icon = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:kbd-input-group:icon"),
    )
    .expect("fret icon");
    let kbd0 = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:kbd-input-group:kbd0"),
    )
    .expect("fret kbd0");
    let kbd1 = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:kbd-input-group:kbd1"),
    )
    .expect("fret kbd1");

    assert_close_px(
        "kbd-input-group group w",
        group.bounds.size.width,
        web_group.rect.w,
        1.0,
    );
    assert_close_px(
        "kbd-input-group group h",
        group.bounds.size.height,
        web_group.rect.h,
        1.0,
    );

    assert_close_px(
        "kbd-input-group input x",
        Px(input.bounds.origin.x.0 - group.bounds.origin.x.0),
        web_input.rect.x - web_group.rect.x,
        1.0,
    );
    assert_close_px(
        "kbd-input-group input w",
        input.bounds.size.width,
        web_input.rect.w,
        1.0,
    );

    assert_close_px(
        "kbd-input-group svg x",
        Px(icon.bounds.origin.x.0 - group.bounds.origin.x.0),
        web_svg.rect.x - web_group.rect.x,
        1.0,
    );
    assert_close_px(
        "kbd-input-group svg y",
        Px(icon.bounds.origin.y.0 - group.bounds.origin.y.0),
        web_svg.rect.y - web_group.rect.y,
        1.0,
    );
    assert_close_px(
        "kbd-input-group svg w",
        icon.bounds.size.width,
        web_svg.rect.w,
        1.0,
    );
    assert_close_px(
        "kbd-input-group svg h",
        icon.bounds.size.height,
        web_svg.rect.h,
        1.0,
    );

    assert_close_px(
        "kbd-input-group kbd0 x",
        Px(kbd0.bounds.origin.x.0 - group.bounds.origin.x.0),
        web_kbd0.rect.x - web_group.rect.x,
        1.0,
    );
    assert_close_px(
        "kbd-input-group kbd0 y",
        Px(kbd0.bounds.origin.y.0 - group.bounds.origin.y.0),
        web_kbd0.rect.y - web_group.rect.y,
        1.0,
    );
    assert_close_px(
        "kbd-input-group kbd0 w",
        kbd0.bounds.size.width,
        web_kbd0.rect.w,
        1.0,
    );
    assert_close_px(
        "kbd-input-group kbd0 h",
        kbd0.bounds.size.height,
        web_kbd0.rect.h,
        1.0,
    );

    assert_close_px(
        "kbd-input-group kbd1 x",
        Px(kbd1.bounds.origin.x.0 - group.bounds.origin.x.0),
        web_kbd1.rect.x - web_group.rect.x,
        1.0,
    );
    assert_close_px(
        "kbd-input-group kbd1 y",
        Px(kbd1.bounds.origin.y.0 - group.bounds.origin.y.0),
        web_kbd1.rect.y - web_group.rect.y,
        1.0,
    );
    assert_close_px(
        "kbd-input-group kbd1 w",
        kbd1.bounds.size.width,
        web_kbd1.rect.w,
        1.0,
    );
    assert_close_px(
        "kbd-input-group kbd1 h",
        kbd1.bounds.size.height,
        web_kbd1.rect.h,
        1.0,
    );
}

fn web_vs_fret_layout_input_group_textarea_geometry_matches() {
    let web = read_web_golden("input-group-textarea");
    let theme = web_theme(&web);
    let web_group = web_find_by_class_tokens(&theme.root, &["group/input-group", "border-input"])
        .expect("web input group root");
    let web_textarea = find_first(web_group, &|n| n.tag == "textarea").expect("web textarea");

    let mut web_svgs = Vec::new();
    web_collect_tag(web_group, "svg", &mut web_svgs);
    web_svgs.sort_by(|a, b| {
        a.rect
            .y
            .partial_cmp(&b.rect.y)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.rect
                    .x
                    .partial_cmp(&b.rect.x)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });

    let web_js = *web_svgs.first().expect("web js icon");
    let web_refresh = *web_svgs.get(1).expect("web refresh icon");
    let web_copy = *web_svgs.get(2).expect("web copy icon");
    let web_run = *web_svgs.get(3).expect("web run icon");

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

    let model: Model<String> = app.models_mut().insert(String::new());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout-input-group-textarea",
        |cx| {
            let container_layout =
                fret_ui_kit::LayoutRefinement::default().w_px(Px(web_group.rect.w));
            let container = cx.container(
                fret_ui::element::ContainerProps {
                    layout: fret_ui_kit::declarative::style::layout_style(
                        &fret_ui::Theme::global(&*cx.app),
                        container_layout,
                    ),
                    ..Default::default()
                },
                move |cx| {
                    let js_icon = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-textarea:js")),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![decl_icon::icon_with(
                                cx,
                                fret_icons::ids::ui::SEARCH,
                                Some(Px(16.0)),
                                None,
                            )]
                        },
                    );

                    let refresh_icon = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-textarea:refresh")),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![decl_icon::icon_with(
                                cx,
                                fret_icons::ids::ui::SEARCH,
                                Some(Px(16.0)),
                                None,
                            )]
                        },
                    );
                    let copy_icon = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-textarea:copy")),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![decl_icon::icon_with(
                                cx,
                                fret_icons::ids::ui::SEARCH,
                                Some(Px(16.0)),
                                None,
                            )]
                        },
                    );

                    let run_icon = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-textarea:run")),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![decl_icon::icon_with(
                                cx,
                                fret_icons::ids::ui::SEARCH,
                                Some(Px(16.0)),
                                None,
                            )]
                        },
                    );

                    let script_label = cx.text("script.js");
                    let block_start_left = cx.flex(
                        FlexProps {
                            layout: LayoutStyle::default(),
                            direction: fret_core::Axis::Horizontal,
                            gap: MetricRef::space(Space::N2).resolve(&Theme::global(&*cx.app)),
                            padding: Edges::all(Px(0.0)),
                            justify: MainAlign::Start,
                            align: CrossAlign::Center,
                            wrap: false,
                        },
                        move |_cx| vec![js_icon, script_label],
                    );

                    let refresh_button = cx.container(
                        fret_ui::element::ContainerProps {
                            layout: fret_ui_kit::declarative::style::layout_style(
                                &Theme::global(&*cx.app),
                                fret_ui_kit::LayoutRefinement::default()
                                    .ml_auto()
                                    .w_px(Px(24.0))
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
                                    padding: Edges::all(Px(0.0)),
                                    justify: MainAlign::Center,
                                    align: CrossAlign::Center,
                                    wrap: false,
                                },
                                move |_cx| vec![refresh_icon],
                            )]
                        },
                    );
                    let copy_button = cx.container(
                        fret_ui::element::ContainerProps {
                            layout: fret_ui_kit::declarative::style::layout_style(
                                &Theme::global(&*cx.app),
                                fret_ui_kit::LayoutRefinement::default()
                                    .w_px(Px(24.0))
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
                                    padding: Edges::all(Px(0.0)),
                                    justify: MainAlign::Center,
                                    align: CrossAlign::Center,
                                    wrap: false,
                                },
                                move |_cx| vec![copy_icon],
                            )]
                        },
                    );

                    let block_end_text = cx.text("Line 1, Column 1");
                    let run_button = cx.container(
                        fret_ui::element::ContainerProps {
                            layout: fret_ui_kit::declarative::style::layout_style(
                                &Theme::global(&*cx.app),
                                fret_ui_kit::LayoutRefinement::default()
                                    .ml_auto()
                                    .w_px(Px(32.0))
                                    .h_px(Px(32.0)),
                            ),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![cx.flex(
                                FlexProps {
                                    layout: LayoutStyle::default(),
                                    direction: fret_core::Axis::Horizontal,
                                    gap: Px(0.0),
                                    padding: Edges::all(Px(0.0)),
                                    justify: MainAlign::Center,
                                    align: CrossAlign::Center,
                                    wrap: false,
                                },
                                move |_cx| vec![run_icon],
                            )]
                        },
                    );

                    let group = fret_ui_shadcn::InputGroup::new(model.clone())
                        .textarea()
                        .textarea_min_height(Px(web_textarea.rect.h))
                        .a11y_label("Golden:input-group-textarea:textarea")
                        .block_start_border_bottom(true)
                        .block_start(vec![block_start_left, refresh_button, copy_button])
                        .block_end_border_top(true)
                        .block_end(vec![block_end_text, run_button])
                        .into_element(cx);

                    vec![cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-textarea:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![group],
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

    let group = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-textarea:root"),
    )
    .expect("fret input group root");
    let textarea = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-group-textarea:textarea"),
    )
    .expect("fret textarea");
    let js = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-textarea:js"),
    )
    .expect("fret js icon");
    let refresh = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-textarea:refresh"),
    )
    .expect("fret refresh icon");
    let copy = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-textarea:copy"),
    )
    .expect("fret copy icon");
    let run = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-textarea:run"),
    )
    .expect("fret run icon");

    assert_close_px(
        "input-group-textarea group w",
        group.bounds.size.width,
        web_group.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-textarea group h",
        group.bounds.size.height,
        web_group.rect.h,
        1.0,
    );

    assert_close_px(
        "input-group-textarea textarea x",
        textarea.bounds.origin.x,
        web_textarea.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-textarea textarea y",
        textarea.bounds.origin.y,
        web_textarea.rect.y,
        1.0,
    );
    assert_close_px(
        "input-group-textarea textarea w",
        textarea.bounds.size.width,
        web_textarea.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-textarea textarea h",
        textarea.bounds.size.height,
        web_textarea.rect.h,
        1.0,
    );

    assert_close_px(
        "input-group-textarea js x",
        js.bounds.origin.x,
        web_js.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-textarea js y",
        js.bounds.origin.y,
        web_js.rect.y,
        1.0,
    );
    assert_close_px(
        "input-group-textarea js w",
        js.bounds.size.width,
        web_js.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-textarea js h",
        js.bounds.size.height,
        web_js.rect.h,
        1.0,
    );

    assert_close_px(
        "input-group-textarea refresh x",
        refresh.bounds.origin.x,
        web_refresh.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-textarea refresh y",
        refresh.bounds.origin.y,
        web_refresh.rect.y,
        1.0,
    );
    assert_close_px(
        "input-group-textarea refresh w",
        refresh.bounds.size.width,
        web_refresh.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-textarea refresh h",
        refresh.bounds.size.height,
        web_refresh.rect.h,
        1.0,
    );

    assert_close_px(
        "input-group-textarea copy x",
        copy.bounds.origin.x,
        web_copy.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-textarea copy y",
        copy.bounds.origin.y,
        web_copy.rect.y,
        1.0,
    );
    assert_close_px(
        "input-group-textarea copy w",
        copy.bounds.size.width,
        web_copy.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-textarea copy h",
        copy.bounds.size.height,
        web_copy.rect.h,
        1.0,
    );

    assert_close_px(
        "input-group-textarea run y",
        run.bounds.origin.y,
        web_run.rect.y,
        1.0,
    );
    assert_close_px(
        "input-group-textarea run w",
        run.bounds.size.width,
        web_run.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-textarea run h",
        run.bounds.size.height,
        web_run.rect.h,
        1.0,
    );
}

fn web_vs_fret_layout_input_group_text_currency_geometry_matches() {
    let web = read_web_golden("input-group-text");
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

    let web_group0 = *web_groups.first().expect("web group0");
    let web_input0 = web_group0
        .children
        .iter()
        .find(|n| n.tag == "input")
        .expect("web input0");
    let web_dollar = web_find_by_tag_and_text(web_group0, "span", "$").expect("web $ label");
    let web_usd = web_find_by_tag_and_text(web_group0, "span", "USD").expect("web USD label");

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

    let model: Model<String> = app.models_mut().insert(String::new());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout-input-group-text-currency",
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
                    let leading = fret_ui_shadcn::InputGroupText::new("$")
                        .refine_layout(LayoutRefinement::default().w_px(Px(web_dollar.rect.w)))
                        .into_element(cx);
                    let trailing = fret_ui_shadcn::InputGroupText::new("USD")
                        .refine_layout(LayoutRefinement::default().w_px(Px(web_usd.rect.w)))
                        .into_element(cx);

                    let group = fret_ui_shadcn::InputGroup::new(model.clone())
                        .a11y_label("Golden:input-group-text:currency:input")
                        .leading(vec![leading])
                        .trailing(vec![trailing])
                        .into_element(cx);
                    vec![cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-text:currency:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![group],
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

    let group = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-text:currency:root"),
    )
    .expect("fret group");
    let input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-group-text:currency:input"),
    )
    .expect("fret input");

    assert_close_px(
        "input-group-text currency group w",
        group.bounds.size.width,
        web_group0.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-text currency group h",
        group.bounds.size.height,
        web_group0.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-text currency input x",
        input.bounds.origin.x,
        web_input0.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-text currency input w",
        input.bounds.size.width,
        web_input0.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-text currency input h",
        input.bounds.size.height,
        web_input0.rect.h,
        1.0,
    );
}

fn web_vs_fret_layout_input_group_text_url_geometry_matches() {
    let web = read_web_golden("input-group-text");
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

    let web_group1 = *web_groups.get(1).expect("web group1");
    let web_input1 = web_group1
        .children
        .iter()
        .find(|n| n.tag == "input")
        .expect("web input1");
    let web_prefix =
        web_find_by_tag_and_text(web_group1, "span", "https://").expect("web https prefix");
    let web_suffix = web_find_by_tag_and_text(web_group1, "span", ".com").expect("web .com suffix");

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

    let model: Model<String> = app.models_mut().insert(String::new());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout-input-group-text-url",
        |cx| {
            let container_layout =
                fret_ui_kit::LayoutRefinement::default().w_px(Px(web_group1.rect.w));
            let container = cx.container(
                fret_ui::element::ContainerProps {
                    layout: fret_ui_kit::declarative::style::layout_style(
                        &fret_ui::Theme::global(&*cx.app),
                        container_layout,
                    ),
                    ..Default::default()
                },
                move |cx| {
                    let prefix = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-text:url:prefix")),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![
                                fret_ui_shadcn::InputGroupText::new("https://")
                                    .refine_layout(
                                        LayoutRefinement::default().w_px(Px(web_prefix.rect.w)),
                                    )
                                    .into_element(cx),
                            ]
                        },
                    );

                    let suffix = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-text:url:suffix")),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![
                                fret_ui_shadcn::InputGroupText::new(".com")
                                    .refine_layout(
                                        LayoutRefinement::default().w_px(Px(web_suffix.rect.w)),
                                    )
                                    .into_element(cx),
                            ]
                        },
                    );

                    let group = fret_ui_shadcn::InputGroup::new(model.clone())
                        .a11y_label("Golden:input-group-text:url:input")
                        .leading(vec![prefix])
                        .trailing(vec![suffix])
                        .into_element(cx);
                    vec![cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-text:url:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![group],
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

    let group = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-text:url:root"),
    )
    .expect("fret group");
    let input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-group-text:url:input"),
    )
    .expect("fret input");
    let prefix = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-text:url:prefix"),
    )
    .expect("fret prefix");
    let suffix = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-text:url:suffix"),
    )
    .expect("fret suffix");

    assert_close_px(
        "input-group-text url group w",
        group.bounds.size.width,
        web_group1.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-text url group h",
        group.bounds.size.height,
        web_group1.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-text url input x",
        input.bounds.origin.x,
        web_input1.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-text url input w",
        input.bounds.size.width,
        web_input1.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-text url prefix x",
        prefix.bounds.origin.x,
        web_prefix.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-text url prefix w",
        prefix.bounds.size.width,
        web_prefix.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-text url suffix x",
        suffix.bounds.origin.x,
        web_suffix.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-text url suffix w",
        suffix.bounds.size.width,
        web_suffix.rect.w,
        1.0,
    );
}

fn web_vs_fret_layout_input_group_text_email_geometry_matches() {
    let web = read_web_golden("input-group-text");
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

    let web_group2 = *web_groups.get(2).expect("web group2");
    let web_input2 = web_group2
        .children
        .iter()
        .find(|n| n.tag == "input")
        .expect("web input2");
    let web_suffix = web_find_by_tag_and_text(web_group2, "span", "@company.com")
        .expect("web @company.com suffix");

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

    let model: Model<String> = app.models_mut().insert(String::new());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout-input-group-text-email",
        |cx| {
            let container_layout =
                fret_ui_kit::LayoutRefinement::default().w_px(Px(web_group2.rect.w));
            let container = cx.container(
                fret_ui::element::ContainerProps {
                    layout: fret_ui_kit::declarative::style::layout_style(
                        &fret_ui::Theme::global(&*cx.app),
                        container_layout,
                    ),
                    ..Default::default()
                },
                move |cx| {
                    let suffix = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-text:email:suffix")),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![
                                fret_ui_shadcn::InputGroupText::new("@company.com")
                                    .refine_layout(
                                        LayoutRefinement::default().w_px(Px(web_suffix.rect.w)),
                                    )
                                    .into_element(cx),
                            ]
                        },
                    );

                    let group = fret_ui_shadcn::InputGroup::new(model.clone())
                        .a11y_label("Golden:input-group-text:email:input")
                        .trailing(vec![suffix])
                        .into_element(cx);
                    vec![cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-text:email:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![group],
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

    let group = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-text:email:root"),
    )
    .expect("fret group");
    let input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-group-text:email:input"),
    )
    .expect("fret input");
    let suffix = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-text:email:suffix"),
    )
    .expect("fret suffix");

    assert_close_px(
        "input-group-text email group w",
        group.bounds.size.width,
        web_group2.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-text email group h",
        group.bounds.size.height,
        web_group2.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-text email input x",
        input.bounds.origin.x,
        web_input2.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-text email input w",
        input.bounds.size.width,
        web_input2.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-text email suffix x",
        suffix.bounds.origin.x,
        web_suffix.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-text email suffix w",
        suffix.bounds.size.width,
        web_suffix.rect.w,
        1.0,
    );
}

fn web_vs_fret_layout_input_group_text_textarea_count_geometry_matches() {
    let web = read_web_golden("input-group-text");
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

    let web_group3 = *web_groups.get(3).expect("web group3");
    let web_textarea3 = web_group3
        .children
        .iter()
        .find(|n| n.tag == "textarea")
        .expect("web textarea3");
    let web_count = web_find_by_tag_and_text(web_group3, "span", "120 characters left")
        .expect("web count label");

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

    let model: Model<String> = app.models_mut().insert(String::new());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout-input-group-text-textarea-count",
        |cx| {
            let container_layout =
                fret_ui_kit::LayoutRefinement::default().w_px(Px(web_group3.rect.w));
            let container = cx.container(
                fret_ui::element::ContainerProps {
                    layout: fret_ui_kit::declarative::style::layout_style(
                        &fret_ui::Theme::global(&*cx.app),
                        container_layout,
                    ),
                    ..Default::default()
                },
                move |cx| {
                    let count = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-text:count:text")),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![
                                fret_ui_shadcn::InputGroupText::new("120 characters left")
                                    .size(fret_ui_shadcn::InputGroupTextSize::Xs)
                                    .refine_layout(
                                        LayoutRefinement::default().w_px(Px(web_count.rect.w)),
                                    )
                                    .into_element(cx),
                            ]
                        },
                    );

                    let group = fret_ui_shadcn::InputGroup::new(model.clone())
                        .textarea()
                        .textarea_min_height(Px(web_textarea3.rect.h))
                        .a11y_label("Golden:input-group-text:count:textarea")
                        .block_end(vec![count])
                        .into_element(cx);

                    vec![cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-text:count:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![group],
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

    let group = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-text:count:root"),
    )
    .expect("fret group");
    let textarea = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-group-text:count:textarea"),
    )
    .expect("fret textarea");
    let count = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-text:count:text"),
    )
    .expect("fret count text");

    assert_close_px(
        "input-group-text textarea count group w",
        group.bounds.size.width,
        web_group3.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-text textarea count group h",
        group.bounds.size.height,
        web_group3.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-text textarea count textarea x",
        textarea.bounds.origin.x,
        web_textarea3.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-text textarea count textarea w",
        textarea.bounds.size.width,
        web_textarea3.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-text textarea count textarea h",
        textarea.bounds.size.height,
        web_textarea3.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-text textarea count text x",
        count.bounds.origin.x,
        web_count.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-text textarea count text w",
        count.bounds.size.width,
        web_count.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-text textarea count text y",
        Px(count.bounds.origin.y.0 - group.bounds.origin.y.0),
        web_count.rect.y - web_group3.rect.y,
        1.0,
    );
    assert_close_px(
        "input-group-text textarea count text h",
        count.bounds.size.height,
        web_count.rect.h,
        1.0,
    );
}

fn web_vs_fret_layout_input_group_custom_geometry_matches() {
    let web = read_web_golden("input-group-custom");
    let theme = web_theme(&web);
    let web_group = web_find_by_class_tokens(&theme.root, &["group/input-group", "border-input"])
        .expect("web input group root");

    let web_textarea = web_group
        .children
        .iter()
        .find(|n| n.tag == "textarea")
        .expect("web textarea node");
    let web_submit =
        web_find_by_tag_and_text(web_group, "button", "Submit").expect("web submit button node");
    let expected_submit_w = Px(web_submit.rect.w);

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

    let model: Model<String> = app.models_mut().insert(String::new());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout-input-group-custom",
        |cx| {
            let container_layout =
                fret_ui_kit::LayoutRefinement::default().w_px(Px(web_group.rect.w));
            let container = cx.container(
                fret_ui::element::ContainerProps {
                    layout: fret_ui_kit::declarative::style::layout_style(
                        &fret_ui::Theme::global(&*cx.app),
                        container_layout,
                    ),
                    ..Default::default()
                },
                move |cx| {
                    let submit = fret_ui_shadcn::InputGroupButton::new("Submit")
                        .variant(fret_ui_shadcn::ButtonVariant::Default)
                        .size(fret_ui_shadcn::InputGroupButtonSize::Sm)
                        .a11y_label("Golden:input-group-custom:submit")
                        .refine_layout(
                            LayoutRefinement::default()
                                .ml_auto()
                                .w_px(expected_submit_w),
                        )
                        .into_element(cx);

                    let group = fret_ui_shadcn::InputGroup::new(model.clone())
                        .textarea()
                        .textarea_min_height(Px(web_textarea.rect.h))
                        .a11y_label("Golden:input-group-custom:textarea")
                        .block_end(vec![submit])
                        .into_element(cx);

                    vec![cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-custom:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![group],
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

    let group = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-custom:root"),
    )
    .expect("fret input group root");
    let textarea = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-group-custom:textarea"),
    )
    .expect("fret textarea");
    let submit = find_semantics(
        &snap,
        SemanticsRole::Button,
        Some("Golden:input-group-custom:submit"),
    )
    .expect("fret submit button");

    assert_close_px(
        "input-group-custom group w",
        group.bounds.size.width,
        web_group.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-custom group h",
        group.bounds.size.height,
        web_group.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-custom textarea w",
        textarea.bounds.size.width,
        web_textarea.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-custom textarea h",
        textarea.bounds.size.height,
        web_textarea.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-custom textarea y",
        Px(textarea.bounds.origin.y.0 - group.bounds.origin.y.0),
        web_textarea.rect.y - web_group.rect.y,
        1.0,
    );
    assert_close_px(
        "input-group-custom submit w",
        submit.bounds.size.width,
        web_submit.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-custom submit h",
        submit.bounds.size.height,
        web_submit.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-custom submit x",
        submit.bounds.origin.x,
        web_submit.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-custom submit y",
        Px(submit.bounds.origin.y.0 - group.bounds.origin.y.0),
        web_submit.rect.y - web_group.rect.y,
        1.0,
    );
}

fn web_vs_fret_layout_input_group_demo_block_end_geometry_matches() {
    let web = read_web_golden("input-group-demo");
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

    let web_group = *web_groups
        .get(2)
        .expect("web input group (textarea + block-end addon)");
    let web_textarea = web_group
        .children
        .iter()
        .find(|n| n.tag == "textarea")
        .expect("web textarea node");
    let web_auto =
        web_find_by_tag_and_text(web_group, "button", "Auto").expect("web Auto button node");
    let web_used =
        web_find_by_tag_and_text(web_group, "span", "52% used").expect("web usage label node");
    let web_send = {
        let mut buttons = Vec::new();
        web_collect_tag(web_group, "button", &mut buttons);
        *buttons
            .iter()
            .max_by(|a, b| {
                a.rect
                    .x
                    .partial_cmp(&b.rect.x)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .expect("web send button node")
    };
    let web_separator = find_first(web_group, &|n| {
        n.class_name
            .as_deref()
            .is_some_and(|c| c.contains("bg-border shrink-0"))
            && n.attrs
                .get("data-orientation")
                .is_some_and(|o| o == "vertical")
    })
    .expect("web separator node");
    let expected_auto_w = Px(web_auto.rect.w);
    let expected_used_w = Px(web_used.rect.w);

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

    let model: Model<String> = app.models_mut().insert(String::new());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout-input-group-demo-block-end",
        |cx| {
            let container_layout =
                fret_ui_kit::LayoutRefinement::default().w_px(Px(web_group.rect.w));
            let container = cx.container(
                fret_ui::element::ContainerProps {
                    layout: fret_ui_kit::declarative::style::layout_style(
                        &fret_ui::Theme::global(&*cx.app),
                        container_layout,
                    ),
                    ..Default::default()
                },
                move |cx| {
                    let plus_icon = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-demo:block:plus-icon")),
                            ..Default::default()
                        },
                        move |cx| vec![decl_icon::icon(cx, fret_icons::ids::ui::SEARCH)],
                    );
                    let plus_button = fret_ui_shadcn::InputGroupButton::new("")
                        .variant(fret_ui_shadcn::ButtonVariant::Outline)
                        .size(fret_ui_shadcn::InputGroupButtonSize::IconXs)
                        .refine_style(ChromeRefinement::default().rounded(Radius::Full))
                        .children(vec![plus_icon])
                        .into_element(cx);

                    let auto = fret_ui_shadcn::InputGroupButton::new("Auto")
                        .variant(fret_ui_shadcn::ButtonVariant::Ghost)
                        .a11y_label("Golden:input-group-demo:block:auto")
                        .refine_layout(LayoutRefinement::default().w_px(expected_auto_w))
                        .into_element(cx);

                    let used = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-demo:block:used")),
                            layout: fret_ui_kit::declarative::style::layout_style(
                                &Theme::global(&*cx.app),
                                LayoutRefinement::default().ml_auto(),
                            ),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![
                                fret_ui_shadcn::InputGroupText::new("52% used")
                                    .refine_layout(
                                        LayoutRefinement::default().w_px(expected_used_w),
                                    )
                                    .into_element(cx),
                            ]
                        },
                    );

                    let separator = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-demo:block:separator")),
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Px(Px(web_separator.rect.w)),
                                    height: Length::Px(Px(web_separator.rect.h)),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        move |cx| {
                            vec![
                                fret_ui_shadcn::Separator::new()
                                    .orientation(fret_ui_shadcn::SeparatorOrientation::Vertical)
                                    .into_element(cx),
                            ]
                        },
                    );

                    let send_icon = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-demo:block:send-icon")),
                            ..Default::default()
                        },
                        move |cx| vec![decl_icon::icon(cx, fret_icons::ids::ui::SEARCH)],
                    );
                    let send_button = fret_ui_shadcn::InputGroupButton::new("")
                        .variant(fret_ui_shadcn::ButtonVariant::Default)
                        .size(fret_ui_shadcn::InputGroupButtonSize::IconXs)
                        .a11y_label("Golden:input-group-demo:block:send")
                        .disabled(true)
                        .refine_style(ChromeRefinement::default().rounded(Radius::Full))
                        .children(vec![send_icon])
                        .into_element(cx);

                    let group = fret_ui_shadcn::InputGroup::new(model.clone())
                        .textarea()
                        .textarea_min_height(Px(web_textarea.rect.h))
                        .a11y_label("Golden:input-group-demo:block:textarea")
                        .block_end(vec![plus_button, auto, used, separator, send_button])
                        .into_element(cx);

                    vec![cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-demo:block:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![group],
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

    let group = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-demo:block:root"),
    )
    .expect("fret input group root");
    let textarea = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-group-demo:block:textarea"),
    )
    .expect("fret textarea");
    let auto = find_semantics(
        &snap,
        SemanticsRole::Button,
        Some("Golden:input-group-demo:block:auto"),
    )
    .expect("fret Auto button");
    let used = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-demo:block:used"),
    )
    .expect("fret usage label");
    let separator = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-demo:block:separator"),
    )
    .expect("fret separator");
    let send = find_semantics(
        &snap,
        SemanticsRole::Button,
        Some("Golden:input-group-demo:block:send"),
    )
    .expect("fret send button");

    assert_close_px(
        "input-group-demo block-end group w",
        group.bounds.size.width,
        web_group.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-demo block-end group h",
        group.bounds.size.height,
        web_group.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-demo block-end textarea w",
        textarea.bounds.size.width,
        web_textarea.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-demo block-end textarea h",
        textarea.bounds.size.height,
        web_textarea.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-demo block-end auto w",
        auto.bounds.size.width,
        web_auto.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-demo block-end auto h",
        auto.bounds.size.height,
        web_auto.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-demo block-end used w",
        used.bounds.size.width,
        web_used.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-demo block-end used h",
        used.bounds.size.height,
        web_used.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-demo block-end used y",
        Px(used.bounds.origin.y.0 - group.bounds.origin.y.0),
        web_used.rect.y - web_group.rect.y,
        1.0,
    );
    assert_close_px(
        "input-group-demo block-end separator w",
        separator.bounds.size.width,
        web_separator.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-demo block-end separator h",
        separator.bounds.size.height,
        web_separator.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-demo block-end send w",
        send.bounds.size.width,
        web_send.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-demo block-end send h",
        send.bounds.size.height,
        web_send.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-demo block-end send x",
        send.bounds.origin.x,
        web_send.rect.x,
        1.0,
    );
}
