use fret_core::Rect;
use serde::Deserialize;

use super::headless_fixture_primitives::{
    Material3HeadlessBoundsV1, Material3HeadlessSettleWindowV1,
    assert_material3_headless_schema_version,
};

const MATERIAL3_HEADLESS_TABS_CASES_V1: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/material3_headless_tabs_cases_v1.json"
));

#[derive(Debug, Deserialize)]
pub(crate) struct Material3TabsGoldenSuiteV1 {
    schema_version: u32,
    bounds: Material3HeadlessBoundsV1,
    cases: Vec<Material3TabsGoldenCaseV1>,
}

impl Material3TabsGoldenSuiteV1 {
    pub(crate) fn bounds(&self) -> Rect {
        self.bounds.rect()
    }

    pub(crate) fn cases(&self) -> &[Material3TabsGoldenCaseV1] {
        &self.cases
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Material3TabsGoldenCaseV1 {
    id: String,
    kind: Material3TabsGoldenCaseKindV1,
    #[serde(default)]
    layout_direction: Material3TabsGoldenLayoutDirectionV1,
    target_test_id: Option<String>,
    #[serde(flatten)]
    settle: Material3HeadlessSettleWindowV1,
}

impl Material3TabsGoldenCaseV1 {
    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn kind(&self) -> Material3TabsGoldenCaseKindV1 {
        self.kind
    }

    pub(crate) fn layout_direction(&self) -> Material3TabsGoldenLayoutDirectionV1 {
        self.layout_direction
    }

    pub(crate) fn target_test_id(&self) -> &str {
        self.target_test_id.as_deref().unwrap_or_else(|| {
            panic!(
                "{}: expected target_test_id for Material3 tabs {:?} case",
                self.id, self.kind
            )
        })
    }

    pub(crate) fn settle_from_frame(&self) -> usize {
        self.settle.settle_from_frame()
    }

    pub(crate) fn total_frames(&self) -> usize {
        self.settle.total_frames()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Material3TabsGoldenCaseKindV1 {
    Idle,
    Hover,
    FocusVisible,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Material3TabsGoldenLayoutDirectionV1 {
    #[default]
    Ltr,
    Rtl,
}

pub(crate) fn load_material3_tabs_golden_suite_v1() -> Material3TabsGoldenSuiteV1 {
    let suite: Material3TabsGoldenSuiteV1 = serde_json::from_str(MATERIAL3_HEADLESS_TABS_CASES_V1)
        .expect("material3 tabs golden fixture must parse");
    assert_material3_headless_schema_version(suite.schema_version, 1, "tabs golden");
    suite
}
