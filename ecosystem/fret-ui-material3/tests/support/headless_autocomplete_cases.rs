use std::sync::Arc;

use fret_ui_material3::AutocompleteItem;
use serde::Deserialize;

const MATERIAL3_HEADLESS_AUTOCOMPLETE_CASES_V1: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/material3_headless_autocomplete_cases_v1.json"
));

#[derive(Debug, Deserialize)]
pub(crate) struct Material3AutocompleteGoldenSuiteV1 {
    schema_version: u32,
    items: Vec<Material3AutocompleteItemV1>,
    cases: Vec<Material3AutocompleteGoldenCaseV1>,
}

impl Material3AutocompleteGoldenSuiteV1 {
    pub(crate) fn items(&self) -> Arc<[AutocompleteItem]> {
        self.items
            .iter()
            .map(Material3AutocompleteItemV1::to_autocomplete_item)
            .collect::<Vec<_>>()
            .into()
    }

    pub(crate) fn closed_case(&self) -> &Material3AutocompleteGoldenCaseV1 {
        self.cases
            .iter()
            .find(|case| case.kind == Material3AutocompleteGoldenCaseKindV1::Closed)
            .expect("material3 autocomplete golden fixture must include a closed case")
    }

    pub(crate) fn open_cases(&self) -> impl Iterator<Item = &Material3AutocompleteGoldenCaseV1> {
        self.cases
            .iter()
            .filter(|case| case.kind == Material3AutocompleteGoldenCaseKindV1::Open)
    }
}

#[derive(Debug, Deserialize)]
struct Material3AutocompleteItemV1 {
    value: String,
    label: String,
}

impl Material3AutocompleteItemV1 {
    fn to_autocomplete_item(&self) -> AutocompleteItem {
        AutocompleteItem::new(self.value.clone(), self.label.clone())
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Material3AutocompleteGoldenCaseV1 {
    id: String,
    kind: Material3AutocompleteGoldenCaseKindV1,
    focus_test_id: Option<String>,
}

impl Material3AutocompleteGoldenCaseV1 {
    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn focus_test_id(&self) -> &str {
        self.focus_test_id
            .as_deref()
            .unwrap_or_else(|| panic!("{}: expected focus_test_id for open case", self.id))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Material3AutocompleteGoldenCaseKindV1 {
    Closed,
    Open,
}

pub(crate) fn load_material3_autocomplete_golden_suite_v1() -> Material3AutocompleteGoldenSuiteV1 {
    let suite: Material3AutocompleteGoldenSuiteV1 =
        serde_json::from_str(MATERIAL3_HEADLESS_AUTOCOMPLETE_CASES_V1)
            .expect("material3 autocomplete golden fixture must parse");
    assert_eq!(
        suite.schema_version, 1,
        "material3 autocomplete golden fixture schema version"
    );
    suite
}
