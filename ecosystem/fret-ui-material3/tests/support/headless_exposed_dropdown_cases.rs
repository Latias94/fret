use std::sync::Arc;

use fret_ui_material3::AutocompleteItem;
use serde::Deserialize;

use super::headless_fixture_primitives::assert_material3_headless_schema_version;

const MATERIAL3_HEADLESS_EXPOSED_DROPDOWN_CASES_V1: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/material3_headless_exposed_dropdown_cases_v1.json"
));

#[derive(Debug, Deserialize)]
pub(crate) struct Material3ExposedDropdownGoldenSuiteV1 {
    schema_version: u32,
    items: Vec<Material3ExposedDropdownItemV1>,
    cases: Vec<Material3ExposedDropdownGoldenCaseV1>,
}

impl Material3ExposedDropdownGoldenSuiteV1 {
    pub(crate) fn items(&self) -> Arc<[AutocompleteItem]> {
        self.items
            .iter()
            .map(Material3ExposedDropdownItemV1::to_autocomplete_item)
            .collect::<Vec<_>>()
            .into()
    }

    pub(crate) fn closed_case(&self) -> &Material3ExposedDropdownGoldenCaseV1 {
        self.cases
            .iter()
            .find(|case| case.kind == Material3ExposedDropdownGoldenCaseKindV1::Closed)
            .expect("material3 exposed dropdown golden fixture must include a closed case")
    }

    pub(crate) fn open_cases(&self) -> impl Iterator<Item = &Material3ExposedDropdownGoldenCaseV1> {
        self.cases
            .iter()
            .filter(|case| case.kind == Material3ExposedDropdownGoldenCaseKindV1::Open)
    }
}

#[derive(Debug, Deserialize)]
struct Material3ExposedDropdownItemV1 {
    value: String,
    label: String,
    #[serde(default)]
    disabled: bool,
    test_id: Option<String>,
}

impl Material3ExposedDropdownItemV1 {
    fn to_autocomplete_item(&self) -> AutocompleteItem {
        let mut item = AutocompleteItem::new(self.value.clone(), self.label.clone());
        if self.disabled {
            item = item.disabled(true);
        }
        if let Some(id) = &self.test_id {
            item = item.test_id(id.clone());
        }
        item
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Material3ExposedDropdownGoldenCaseV1 {
    id: String,
    kind: Material3ExposedDropdownGoldenCaseKindV1,
    target_test_id: Option<String>,
    trailing_icon_test_id: Option<String>,
}

impl Material3ExposedDropdownGoldenCaseV1 {
    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn target_test_id(&self) -> &str {
        self.target_test_id.as_deref().unwrap_or_else(|| {
            panic!(
                "{}: expected target_test_id for exposed dropdown open case",
                self.id
            )
        })
    }

    pub(crate) fn trailing_icon_test_id(&self) -> &str {
        self.trailing_icon_test_id.as_deref().unwrap_or_else(|| {
            panic!(
                "{}: expected trailing_icon_test_id for exposed dropdown open case",
                self.id
            )
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Material3ExposedDropdownGoldenCaseKindV1 {
    Closed,
    Open,
}

pub(crate) fn load_material3_exposed_dropdown_golden_suite_v1()
-> Material3ExposedDropdownGoldenSuiteV1 {
    let suite: Material3ExposedDropdownGoldenSuiteV1 =
        serde_json::from_str(MATERIAL3_HEADLESS_EXPOSED_DROPDOWN_CASES_V1)
            .expect("material3 exposed dropdown golden fixture must parse");
    assert_material3_headless_schema_version(suite.schema_version, 1, "exposed dropdown golden");
    suite
}
