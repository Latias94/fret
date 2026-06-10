use serde::Deserialize;

use super::headless_fixture_primitives::assert_material3_headless_schema_version;

const MATERIAL3_HEADLESS_ICON_BUTTON_CASES_V1: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/material3_headless_icon_button_cases_v1.json"
));

#[derive(Debug, Deserialize)]
pub(crate) struct Material3IconButtonGoldenSuiteV1 {
    schema_version: u32,
    cases: Vec<Material3IconButtonGoldenCaseV1>,
}

impl Material3IconButtonGoldenSuiteV1 {
    pub(crate) fn cases(&self) -> &[Material3IconButtonGoldenCaseV1] {
        &self.cases
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Material3IconButtonGoldenCaseV1 {
    id: String,
    kind: Material3IconButtonGoldenCaseKindV1,
    target_test_id: Option<String>,
}

impl Material3IconButtonGoldenCaseV1 {
    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn kind(&self) -> Material3IconButtonGoldenCaseKindV1 {
        self.kind
    }

    pub(crate) fn target_test_id(&self) -> &str {
        self.target_test_id.as_deref().unwrap_or_else(|| {
            panic!(
                "{}: expected target_test_id for Material3 icon button {:?} case",
                self.id, self.kind
            )
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Material3IconButtonGoldenCaseKindV1 {
    Idle,
    Hover,
    FocusVisible,
}

pub(crate) fn load_material3_icon_button_golden_suite_v1() -> Material3IconButtonGoldenSuiteV1 {
    let suite: Material3IconButtonGoldenSuiteV1 =
        serde_json::from_str(MATERIAL3_HEADLESS_ICON_BUTTON_CASES_V1)
            .expect("material3 icon button golden fixture must parse");
    assert_material3_headless_schema_version(suite.schema_version, 1, "icon button golden");
    suite
}
