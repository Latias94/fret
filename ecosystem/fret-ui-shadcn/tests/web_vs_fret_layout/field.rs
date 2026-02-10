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
