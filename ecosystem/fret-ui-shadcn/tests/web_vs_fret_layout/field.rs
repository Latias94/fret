use super::*;

#[derive(Debug, Clone, Deserialize)]
struct FixtureSuite<T> {
    schema_version: u32,
    cases: Vec<T>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum LayoutFieldRecipe {
    Input,
    Checkbox,
    Switch,
    Select,
    Radio,
    Textarea,
    Group,
    Fieldset,
    ChoiceCard,
    SliderTrackGeometry,
    SliderThumbInsets,
    DemoSeparatorHeight,
    ResponsiveOrientation,
}

#[derive(Debug, Clone, Deserialize)]
struct LayoutFieldCase {
    id: String,
    web_name: String,
    recipe: LayoutFieldRecipe,
}

#[test]
fn web_vs_fret_layout_field_geometry_matches_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/layout_field_cases_v1.json"
    ));
    let suite: FixtureSuite<LayoutFieldCase> =
        serde_json::from_str(raw).expect("layout field fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("layout field case={}", case.id);
        match case.recipe {
            LayoutFieldRecipe::Input => {
                assert_eq!(case.web_name, "field-input");
                web_vs_fret_layout_field_input_geometry();
            }
            LayoutFieldRecipe::Checkbox => {
                assert_eq!(case.web_name, "field-checkbox");
                web_vs_fret_layout_field_checkbox_geometry();
            }
            LayoutFieldRecipe::Switch => {
                assert_eq!(case.web_name, "field-switch");
                web_vs_fret_layout_field_switch_geometry();
            }
            LayoutFieldRecipe::Select => {
                assert_eq!(case.web_name, "field-select");
                web_vs_fret_layout_field_select_geometry();
            }
            LayoutFieldRecipe::Radio => {
                assert_eq!(case.web_name, "field-radio");
                web_vs_fret_layout_field_radio_geometry();
            }
            LayoutFieldRecipe::Textarea => {
                assert_eq!(case.web_name, "field-textarea");
                web_vs_fret_layout_field_textarea_geometry();
            }
            LayoutFieldRecipe::Group => {
                assert_eq!(case.web_name, "field-group");
                web_vs_fret_layout_field_group_geometry();
            }
            LayoutFieldRecipe::Fieldset => {
                assert_eq!(case.web_name, "field-fieldset");
                web_vs_fret_layout_field_fieldset_geometry();
            }
            LayoutFieldRecipe::ChoiceCard => {
                assert_eq!(case.web_name, "field-choice-card");
                web_vs_fret_layout_field_choice_card_geometry();
            }
            LayoutFieldRecipe::SliderTrackGeometry => {
                assert_eq!(case.web_name, "field-slider");
                web_vs_fret_layout_field_slider_track_geometry_matches_web();
            }
            LayoutFieldRecipe::SliderThumbInsets => {
                assert_eq!(case.web_name, "field-slider");
                web_vs_fret_layout_field_slider_thumb_insets_match_web();
            }
            LayoutFieldRecipe::DemoSeparatorHeight => {
                assert_eq!(case.web_name, "field-demo");
                web_vs_fret_layout_field_demo_separator_height_matches_web();
            }
            LayoutFieldRecipe::ResponsiveOrientation => {
                assert_eq!(case.web_name, "field-responsive");
                web_vs_fret_layout_field_responsive_orientation_places_input_beside_content();
            }
        }
    }
}

fn web_vs_fret_layout_field_responsive_orientation_places_input_beside_content() {
    let web = read_web_golden("field-responsive");
    let theme = web_theme(&web);

    let web_max_w = find_first(&theme.root, &|n| {
        n.tag == "div" && class_has_token(n, "max-w-4xl")
    })
    .expect("web max-w-4xl container");

    let web_content = find_first(&theme.root, &|n| {
        n.tag == "div"
            && class_has_token(n, "group/field-content")
            && contains_text(n, "Provide your full name")
    })
    .expect("web field-content");

    let web_input = find_first(&theme.root, &|n| n.tag == "input" && contains_id(n, "name"))
        .expect("web input");

    let web_dx = web_input.rect.x - web_content.rect.x;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root_frames(bounds, 2, |cx| {
        let theme = Theme::global(&*cx.app).clone();
        let content_layout =
            decl_style::layout_style(&theme, LayoutRefinement::default().flex_1().min_w_0());

        let content = fret_ui_shadcn::FieldContent::new(vec![
            fret_ui_shadcn::FieldLabel::new("Name").into_element(cx),
            fret_ui_shadcn::FieldDescription::new("Provide your full name for identification")
                .into_element(cx),
        ])
        .into_element(cx);

        let content = cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: content_layout,
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-responsive:content")),
                ..Default::default()
            },
            move |_cx| vec![content],
        );

        let model: Model<String> = cx.app.models_mut().insert(String::new());
        let input = fret_ui_shadcn::Input::new(model)
            .a11y_label("NameInput")
            .placeholder("Evil Rabbit")
            .into_element(cx);

        let field = fret_ui_shadcn::Field::new(vec![content, input])
            .orientation(fret_ui_shadcn::FieldOrientation::Responsive)
            .into_element(cx);

        vec![cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Px(Px(web_max_w.rect.w)),
                        height: Length::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            move |_cx| vec![field],
        )]
    });

    let fret_content = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-responsive:content"),
    )
    .expect("fret field-content");
    let fret_input = find_semantics(&snap, SemanticsRole::TextField, Some("NameInput"))
        .or_else(|| find_semantics(&snap, SemanticsRole::TextField, None))
        .expect("fret input");

    let fret_dx = fret_input.bounds.origin.x.0 - fret_content.bounds.origin.x.0;

    assert!(
        fret_dx >= 1.0,
        "expected responsive field to place input beside content; dx={fret_dx} (content={:?} input={:?})",
        fret_content.bounds,
        fret_input.bounds
    );
    assert_close_px("field-responsive input dx", Px(fret_dx), web_dx, 12.0);
}
