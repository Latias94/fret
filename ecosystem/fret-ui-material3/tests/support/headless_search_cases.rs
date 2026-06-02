use std::sync::Arc;

use fret_ui_material3::SearchViewPresentation;
use serde::Deserialize;

const MATERIAL3_HEADLESS_SEARCH_CASES_V1: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/material3_headless_search_cases_v1.json"
));

#[derive(Debug, Deserialize)]
pub(crate) struct Material3SearchGoldenSuiteV1 {
    schema_version: u32,
    search_bar_cases: Vec<Material3SearchBarGoldenCaseV1>,
    search_view_results: Vec<String>,
    search_view_cases: Vec<Material3SearchViewGoldenCaseV1>,
}

impl Material3SearchGoldenSuiteV1 {
    pub(crate) fn search_bar_cases(&self) -> &[Material3SearchBarGoldenCaseV1] {
        &self.search_bar_cases
    }

    pub(crate) fn search_view_cases(&self) -> &[Material3SearchViewGoldenCaseV1] {
        &self.search_view_cases
    }

    pub(crate) fn search_view_results(&self) -> Arc<[Arc<str>]> {
        self.search_view_results
            .iter()
            .map(|label| Arc::<str>::from(label.as_str()))
            .collect::<Vec<_>>()
            .into()
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Material3SearchBarGoldenCaseV1 {
    id: String,
    hover: bool,
    pressed: bool,
    focus_visible: bool,
}

impl Material3SearchBarGoldenCaseV1 {
    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn hover(&self) -> bool {
        self.hover
    }

    pub(crate) fn pressed(&self) -> bool {
        self.pressed
    }

    pub(crate) fn focus_visible(&self) -> bool {
        self.focus_visible
    }

    pub(crate) fn settle_from_frame(&self) -> usize {
        if self.pressed { 48 } else { 28 }
    }

    pub(crate) fn total_frames(&self) -> usize {
        if self.pressed { 96 } else { 64 }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Material3SearchViewGoldenCaseV1 {
    id: String,
    open: bool,
    presentation: Material3SearchViewPresentationV1,
}

impl Material3SearchViewGoldenCaseV1 {
    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn open(&self) -> bool {
        self.open
    }

    pub(crate) fn presentation(&self) -> SearchViewPresentation {
        self.presentation.to_search_view_presentation()
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Material3SearchViewPresentationV1 {
    Docked,
    FullScreen,
}

impl Material3SearchViewPresentationV1 {
    fn to_search_view_presentation(self) -> SearchViewPresentation {
        match self {
            Self::Docked => SearchViewPresentation::Docked,
            Self::FullScreen => SearchViewPresentation::FullScreen,
        }
    }
}

pub(crate) fn load_material3_search_golden_suite_v1() -> Material3SearchGoldenSuiteV1 {
    let suite: Material3SearchGoldenSuiteV1 =
        serde_json::from_str(MATERIAL3_HEADLESS_SEARCH_CASES_V1)
            .expect("material3 search golden fixture must parse");
    assert_eq!(
        suite.schema_version, 1,
        "material3 search golden fixture schema version"
    );
    suite
}
