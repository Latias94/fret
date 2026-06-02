use std::sync::Arc;

use fret_icons::IconId;
use fret_runtime::Model;
use fret_ui_material3::{List, ListItem};
use serde::Deserialize;

const MATERIAL3_HEADLESS_LIST_CASES_V1: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/material3_headless_list_cases_v1.json"
));

#[derive(Debug, Deserialize)]
pub(crate) struct Material3ListGoldenSuiteV1 {
    schema_version: u32,
    list: Material3ListDefinitionV1,
    cases: Vec<Material3ListGoldenCaseV1>,
}

impl Material3ListGoldenSuiteV1 {
    pub(crate) fn list(&self) -> &Material3ListDefinitionV1 {
        &self.list
    }

    pub(crate) fn cases(&self) -> &[Material3ListGoldenCaseV1] {
        &self.cases
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Material3ListDefinitionV1 {
    test_id: String,
    selected_value: String,
    items: Vec<Material3ListItemDefinitionV1>,
}

impl Material3ListDefinitionV1 {
    pub(crate) fn selected_value(&self) -> Arc<str> {
        Arc::<str>::from(self.selected_value.as_str())
    }

    pub(crate) fn list(&self, selected: Model<Arc<str>>) -> List {
        List::new(selected)
            .test_id(self.test_id.clone())
            .items(self.items.iter().map(|item| item.list_item()).collect())
    }
}

#[derive(Debug, Deserialize)]
struct Material3ListItemDefinitionV1 {
    value: String,
    label: String,
    overline_text: Option<String>,
    supporting_text: Option<String>,
    trailing_supporting_text: Option<String>,
    leading_icon: Option<Material3ListIconV1>,
    trailing_icon: Option<Material3ListIconV1>,
    #[serde(default)]
    disabled: bool,
    test_id: Option<String>,
}

impl Material3ListItemDefinitionV1 {
    fn list_item(&self) -> ListItem {
        let mut item = ListItem::new(self.value.clone(), self.label.clone());

        if let Some(overline_text) = self.overline_text.as_ref() {
            item = item.overline_text(overline_text.clone());
        }

        if let Some(supporting_text) = self.supporting_text.as_ref() {
            item = item.supporting_text(supporting_text.clone());
        }

        if let Some(trailing_supporting_text) = self.trailing_supporting_text.as_ref() {
            item = item.trailing_supporting_text(trailing_supporting_text.clone());
        }

        if let Some(icon) = self.leading_icon {
            item = item.leading_icon(icon.to_icon_id());
        }

        if let Some(icon) = self.trailing_icon {
            item = item.trailing_icon(icon.to_icon_id());
        }

        if self.disabled {
            item = item.disabled(true);
        }

        if let Some(test_id) = self.test_id.as_ref() {
            item = item.test_id(test_id.clone());
        }

        item
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Material3ListIconV1 {
    Search,
    Settings,
    ChevronRight,
    Slash,
}

impl Material3ListIconV1 {
    fn to_icon_id(self) -> IconId {
        match self {
            Self::Search => fret_icons::ids::ui::SEARCH,
            Self::Settings => fret_icons::ids::ui::SETTINGS,
            Self::ChevronRight => fret_icons::ids::ui::CHEVRON_RIGHT,
            Self::Slash => fret_icons::ids::ui::SLASH,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Material3ListGoldenCaseV1 {
    id: String,
    hover_test_id: Option<String>,
    focus_test_id: Option<String>,
    settle_from_frame: usize,
    total_frames: usize,
}

impl Material3ListGoldenCaseV1 {
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

pub(crate) fn load_material3_list_golden_suite_v1() -> Material3ListGoldenSuiteV1 {
    let suite: Material3ListGoldenSuiteV1 = serde_json::from_str(MATERIAL3_HEADLESS_LIST_CASES_V1)
        .expect("material3 list golden fixture must parse");
    assert_eq!(
        suite.schema_version, 1,
        "material3 list golden fixture schema version"
    );
    suite
}
