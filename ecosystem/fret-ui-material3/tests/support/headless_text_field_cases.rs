use fret_runtime::Model;
use fret_ui_material3::{TextField, TextFieldVariant};
use serde::Deserialize;

const MATERIAL3_HEADLESS_TEXT_FIELD_CASES_V1: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/material3_headless_text_field_cases_v1.json"
));

#[derive(Debug, Deserialize)]
pub(crate) struct Material3TextFieldGoldenSuiteV1 {
    schema_version: u32,
    fields: Vec<Material3TextFieldDefinitionV1>,
    cases: Vec<Material3TextFieldGoldenCaseV1>,
}

impl Material3TextFieldGoldenSuiteV1 {
    pub(crate) fn fields(&self) -> &[Material3TextFieldDefinitionV1] {
        &self.fields
    }

    pub(crate) fn cases(&self) -> &[Material3TextFieldGoldenCaseV1] {
        &self.cases
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Material3TextFieldDefinitionV1 {
    test_id: String,
    variant: Material3TextFieldVariantV1,
    label: String,
    placeholder: String,
    supporting_text: String,
    value: String,
    #[serde(default)]
    error: bool,
    #[serde(default)]
    disabled: bool,
}

impl Material3TextFieldDefinitionV1 {
    pub(crate) fn value(&self) -> &str {
        &self.value
    }

    pub(crate) fn text_field(&self, model: Model<String>) -> TextField {
        TextField::new(model)
            .variant(self.variant.to_text_field_variant())
            .label(self.label.clone())
            .placeholder(self.placeholder.clone())
            .supporting_text(self.supporting_text.clone())
            .error(self.error)
            .disabled(self.disabled)
            .test_id(self.test_id.clone())
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Material3TextFieldVariantV1 {
    Outlined,
    Filled,
}

impl Material3TextFieldVariantV1 {
    fn to_text_field_variant(self) -> TextFieldVariant {
        match self {
            Self::Outlined => TextFieldVariant::Outlined,
            Self::Filled => TextFieldVariant::Filled,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Material3TextFieldGoldenCaseV1 {
    id: String,
    hover_test_id: Option<String>,
    focus_test_id: Option<String>,
    settle_from_frame: usize,
    total_frames: usize,
}

impl Material3TextFieldGoldenCaseV1 {
    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn is_idle(&self) -> bool {
        self.hover_test_id.is_none() && self.focus_test_id.is_none()
    }

    pub(crate) fn hover_test_id(&self) -> Option<&str> {
        self.hover_test_id.as_deref()
    }

    pub(crate) fn focus_test_id(&self) -> Option<&str> {
        self.focus_test_id.as_deref()
    }

    pub(crate) fn settle_from_frame(&self) -> usize {
        self.settle_from_frame
    }

    pub(crate) fn total_frames(&self) -> usize {
        self.total_frames
    }
}

pub(crate) fn load_material3_text_field_golden_suite_v1() -> Material3TextFieldGoldenSuiteV1 {
    let suite: Material3TextFieldGoldenSuiteV1 =
        serde_json::from_str(MATERIAL3_HEADLESS_TEXT_FIELD_CASES_V1)
            .expect("material3 text field golden fixture must parse");
    assert_eq!(
        suite.schema_version, 1,
        "material3 text field golden fixture schema version"
    );
    suite
}
