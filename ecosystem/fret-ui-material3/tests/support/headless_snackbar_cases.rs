use fret_runtime::CommandId;
use fret_ui_material3::{Snackbar, SnackbarDuration};
use serde::Deserialize;

const MATERIAL3_HEADLESS_SNACKBAR_CASES_V1: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/material3_headless_snackbar_cases_v1.json"
));

#[derive(Debug, Deserialize)]
pub(crate) struct Material3SnackbarGoldenSuiteV1 {
    schema_version: u32,
    cases: Vec<Material3SnackbarGoldenCaseV1>,
}

impl Material3SnackbarGoldenSuiteV1 {
    pub(crate) fn cases(&self) -> &[Material3SnackbarGoldenCaseV1] {
        &self.cases
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Material3SnackbarGoldenCaseV1 {
    id: String,
    message: String,
    supporting_text: Option<String>,
    action: Option<Material3SnackbarGoldenActionV1>,
    duration: Material3SnackbarDurationV1,
}

impl Material3SnackbarGoldenCaseV1 {
    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn to_snackbar(&self) -> Snackbar {
        let mut snackbar =
            Snackbar::new(self.message.clone()).duration(self.duration.to_snackbar_duration());

        if let Some(supporting_text) = self.supporting_text.as_ref() {
            snackbar = snackbar.supporting_text(supporting_text.clone());
        }

        if let Some(action) = self.action.as_ref() {
            snackbar =
                snackbar.action_id(action.label.clone(), CommandId::new(action.command.clone()));
        }

        snackbar
    }
}

#[derive(Debug, Deserialize)]
struct Material3SnackbarGoldenActionV1 {
    label: String,
    command: String,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Material3SnackbarDurationV1 {
    Short,
    Long,
    Indefinite,
}

impl Material3SnackbarDurationV1 {
    fn to_snackbar_duration(self) -> SnackbarDuration {
        match self {
            Self::Short => SnackbarDuration::Short,
            Self::Long => SnackbarDuration::Long,
            Self::Indefinite => SnackbarDuration::Indefinite,
        }
    }
}

pub(crate) fn load_material3_snackbar_golden_suite_v1() -> Material3SnackbarGoldenSuiteV1 {
    let suite: Material3SnackbarGoldenSuiteV1 =
        serde_json::from_str(MATERIAL3_HEADLESS_SNACKBAR_CASES_V1)
            .expect("material3 snackbar golden fixture must parse");
    assert_eq!(
        suite.schema_version, 1,
        "material3 snackbar golden fixture schema version"
    );
    suite
}
