use std::sync::Arc;

use fret_core::Rect;
use fret_runtime::Model;
use fret_ui_material3::{
    NavigationBar, NavigationBarItem, NavigationDrawer, NavigationDrawerItem,
    NavigationDrawerVariant, NavigationRail, NavigationRailItem,
};
use serde::Deserialize;

use super::headless_fixture_primitives::{
    Material3HeadlessBoundsV1, Material3HeadlessIconV1, Material3HeadlessSettleWindowV1,
    assert_material3_headless_schema_version,
};

const MATERIAL3_HEADLESS_NAVIGATION_CASES_V1: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/material3_headless_navigation_cases_v1.json"
));

#[derive(Debug, Deserialize)]
pub(crate) struct Material3NavigationGoldenSuiteV1 {
    schema_version: u32,
    cases: Vec<Material3NavigationGoldenCaseV1>,
}

impl Material3NavigationGoldenSuiteV1 {
    pub(crate) fn case(&self, id: &str) -> &Material3NavigationGoldenCaseV1 {
        self.cases
            .iter()
            .find(|case| case.id == id)
            .unwrap_or_else(|| panic!("expected material3 navigation fixture case {id}"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Material3NavigationGoldenCaseKindV1 {
    NavigationBar,
    NavigationRail,
    NavigationDrawer,
    ModalNavigationDrawer,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Material3NavigationGoldenCaseV1 {
    id: String,
    kind: Material3NavigationGoldenCaseKindV1,
    bounds: Material3HeadlessBoundsV1,
    selected_value: String,
    a11y_label: String,
    test_id: String,
    modal_test_id: Option<String>,
    underlay_label: Option<String>,
    underlay_test_id: Option<String>,
    #[serde(flatten)]
    settle: Material3HeadlessSettleWindowV1,
    items: Vec<Material3NavigationItemDefinitionV1>,
}

impl Material3NavigationGoldenCaseV1 {
    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn bounds(&self) -> Rect {
        self.bounds.rect()
    }

    pub(crate) fn selected_value(&self) -> Arc<str> {
        Arc::<str>::from(self.selected_value.as_str())
    }

    pub(crate) fn modal_test_id(&self) -> &str {
        self.modal_test_id
            .as_deref()
            .expect("modal navigation drawer case must define modal_test_id")
    }

    pub(crate) fn underlay_label(&self) -> &str {
        self.underlay_label
            .as_deref()
            .expect("modal navigation drawer case must define underlay_label")
    }

    pub(crate) fn underlay_test_id(&self) -> &str {
        self.underlay_test_id
            .as_deref()
            .expect("modal navigation drawer case must define underlay_test_id")
    }

    pub(crate) fn settle_from_frame(&self) -> usize {
        self.settle.settle_from_frame()
    }

    pub(crate) fn total_frames(&self) -> usize {
        self.settle.total_frames()
    }

    pub(crate) fn navigation_bar(&self, model: Model<Arc<str>>) -> NavigationBar {
        assert_eq!(
            self.kind,
            Material3NavigationGoldenCaseKindV1::NavigationBar
        );
        NavigationBar::new(model)
            .a11y_label(self.a11y_label.clone())
            .test_id(self.test_id.clone())
            .items(
                self.items
                    .iter()
                    .map(Material3NavigationItemDefinitionV1::navigation_bar_item)
                    .collect(),
            )
    }

    pub(crate) fn navigation_rail(&self, model: Model<Arc<str>>) -> NavigationRail {
        assert_eq!(
            self.kind,
            Material3NavigationGoldenCaseKindV1::NavigationRail
        );
        NavigationRail::new(model)
            .a11y_label(self.a11y_label.clone())
            .test_id(self.test_id.clone())
            .items(
                self.items
                    .iter()
                    .map(Material3NavigationItemDefinitionV1::navigation_rail_item)
                    .collect(),
            )
    }

    pub(crate) fn navigation_drawer(
        &self,
        model: Model<Arc<str>>,
        variant: NavigationDrawerVariant,
    ) -> NavigationDrawer {
        assert!(
            matches!(
                self.kind,
                Material3NavigationGoldenCaseKindV1::NavigationDrawer
                    | Material3NavigationGoldenCaseKindV1::ModalNavigationDrawer
            ),
            "navigation drawer fixture case kind"
        );
        NavigationDrawer::new(model)
            .variant(variant)
            .a11y_label(self.a11y_label.clone())
            .test_id(self.test_id.clone())
            .items(
                self.items
                    .iter()
                    .map(Material3NavigationItemDefinitionV1::navigation_drawer_item)
                    .collect(),
            )
    }
}

#[derive(Debug, Deserialize)]
struct Material3NavigationItemDefinitionV1 {
    value: String,
    label: String,
    icon: Material3HeadlessIconV1,
    badge: Option<Material3NavigationBadgeV1>,
    #[serde(default)]
    disabled: bool,
    a11y_label: String,
    test_id: String,
}

impl Material3NavigationItemDefinitionV1 {
    fn navigation_bar_item(&self) -> NavigationBarItem {
        let item =
            NavigationBarItem::new(self.value.clone(), self.label.clone(), self.icon.icon_id());
        let item = match self.badge.as_ref() {
            Some(Material3NavigationBadgeV1::Dot) => item.badge_dot(),
            Some(Material3NavigationBadgeV1::Text { text })
            | Some(Material3NavigationBadgeV1::Label { text }) => item.badge_text(text.clone()),
            None => item,
        };
        self.apply_navigation_bar_common(item)
    }

    fn navigation_rail_item(&self) -> NavigationRailItem {
        let item =
            NavigationRailItem::new(self.value.clone(), self.label.clone(), self.icon.icon_id());
        let item = match self.badge.as_ref() {
            Some(Material3NavigationBadgeV1::Dot) => item.badge_dot(),
            Some(Material3NavigationBadgeV1::Text { text })
            | Some(Material3NavigationBadgeV1::Label { text }) => item.badge_text(text.clone()),
            None => item,
        };
        let item = if self.disabled {
            item.disabled(true)
        } else {
            item
        };
        item.a11y_label(self.a11y_label.clone())
            .test_id(self.test_id.clone())
    }

    fn navigation_drawer_item(&self) -> NavigationDrawerItem {
        let item =
            NavigationDrawerItem::new(self.value.clone(), self.label.clone(), self.icon.icon_id());
        let item = match self.badge.as_ref() {
            Some(Material3NavigationBadgeV1::Text { text })
            | Some(Material3NavigationBadgeV1::Label { text }) => item.badge_label(text.clone()),
            Some(Material3NavigationBadgeV1::Dot) | None => item,
        };
        let item = if self.disabled {
            item.disabled(true)
        } else {
            item
        };
        item.a11y_label(self.a11y_label.clone())
            .test_id(self.test_id.clone())
    }

    fn apply_navigation_bar_common(&self, item: NavigationBarItem) -> NavigationBarItem {
        let item = if self.disabled {
            item.disabled(true)
        } else {
            item
        };
        item.a11y_label(self.a11y_label.clone())
            .test_id(self.test_id.clone())
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Material3NavigationBadgeV1 {
    Dot,
    Text { text: String },
    Label { text: String },
}

pub(crate) fn load_material3_navigation_golden_suite_v1() -> Material3NavigationGoldenSuiteV1 {
    let suite: Material3NavigationGoldenSuiteV1 =
        serde_json::from_str(MATERIAL3_HEADLESS_NAVIGATION_CASES_V1)
            .expect("material3 navigation golden fixture must parse");
    assert_material3_headless_schema_version(suite.schema_version, 1, "navigation golden");
    suite
}
