use super::*;

#[derive(Debug, Clone, Deserialize)]
struct FixtureSuite<T> {
    schema_version: u32,
    cases: Vec<T>,
}

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
