use std::sync::Arc;

use fret_core::{Px, Rect};
use fret_ui_material3::{
    SelectItem,
    menu::{MenuEntry, MenuItem},
};
use serde::Deserialize;

use super::headless_fixture_primitives::{
    Material3HeadlessBoundsV1, Material3HeadlessIconV1, assert_material3_headless_schema_version,
};

const MATERIAL3_HEADLESS_OVERLAYS_CASES_V1: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/material3_headless_overlays_cases_v1.json"
));

#[derive(Debug, Deserialize)]
pub(crate) struct Material3OverlayGoldenSuiteV1 {
    schema_version: u32,
    cases: Vec<Material3OverlayGoldenCaseV1>,
}

impl Material3OverlayGoldenSuiteV1 {
    pub(crate) fn tooltip_menu_cases(&self) -> impl Iterator<Item = &Material3OverlayGoldenCaseV1> {
        self.cases
            .iter()
            .filter(|case| case.kind == Material3OverlayGoldenCaseKindV1::TooltipMenu)
    }

    pub(crate) fn select_case(&self) -> &Material3OverlayGoldenCaseV1 {
        self.cases
            .iter()
            .find(|case| case.kind == Material3OverlayGoldenCaseKindV1::Select)
            .expect("material3 overlays fixture must include a select case")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Material3OverlayGoldenCaseKindV1 {
    TooltipMenu,
    Select,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Material3OverlayGoldenCaseV1 {
    id: String,
    kind: Material3OverlayGoldenCaseKindV1,
    bounds: Material3HeadlessBoundsV1,
    padding: Option<f32>,
    open_wait_frames: usize,
    settle_from_frame: Option<usize>,
    total_frames: Option<usize>,
    open_settle_from_frame: Option<usize>,
    open_total_frames: Option<usize>,
    trigger_settle_from_frame: Option<usize>,
    trigger_total_frames: Option<usize>,
    hover_settle_from_frame: Option<usize>,
    hover_total_frames: Option<usize>,
    tooltip: Option<Material3OverlayTooltipV1>,
    menu: Option<Material3OverlayMenuV1>,
    snapshots: Option<Material3OverlaySelectSnapshotsV1>,
    select: Option<Material3OverlaySelectV1>,
}

impl Material3OverlayGoldenCaseV1 {
    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn bounds(&self) -> Rect {
        self.bounds.rect()
    }

    pub(crate) fn padding(&self) -> Px {
        Px(self.padding.unwrap_or(0.0))
    }

    pub(crate) fn open_wait_frames(&self) -> usize {
        self.open_wait_frames
    }

    pub(crate) fn settle_from_frame(&self) -> usize {
        self.settle_from_frame
            .unwrap_or_else(|| panic!("{}: expected settle_from_frame", self.id))
    }

    pub(crate) fn total_frames(&self) -> usize {
        self.total_frames
            .unwrap_or_else(|| panic!("{}: expected total_frames", self.id))
    }

    pub(crate) fn open_settle_from_frame(&self) -> usize {
        self.open_settle_from_frame
            .unwrap_or_else(|| panic!("{}: expected open_settle_from_frame", self.id))
    }

    pub(crate) fn open_total_frames(&self) -> usize {
        self.open_total_frames
            .unwrap_or_else(|| panic!("{}: expected open_total_frames", self.id))
    }

    pub(crate) fn trigger_settle_from_frame(&self) -> usize {
        self.trigger_settle_from_frame
            .unwrap_or_else(|| panic!("{}: expected trigger_settle_from_frame", self.id))
    }

    pub(crate) fn trigger_total_frames(&self) -> usize {
        self.trigger_total_frames
            .unwrap_or_else(|| panic!("{}: expected trigger_total_frames", self.id))
    }

    pub(crate) fn hover_settle_from_frame(&self) -> usize {
        self.hover_settle_from_frame
            .unwrap_or_else(|| panic!("{}: expected hover_settle_from_frame", self.id))
    }

    pub(crate) fn hover_total_frames(&self) -> usize {
        self.hover_total_frames
            .unwrap_or_else(|| panic!("{}: expected hover_total_frames", self.id))
    }

    pub(crate) fn tooltip(&self) -> &Material3OverlayTooltipV1 {
        self.tooltip
            .as_ref()
            .unwrap_or_else(|| panic!("{}: expected tooltip fixture", self.id))
    }

    pub(crate) fn menu(&self) -> &Material3OverlayMenuV1 {
        self.menu
            .as_ref()
            .unwrap_or_else(|| panic!("{}: expected menu fixture", self.id))
    }

    pub(crate) fn menu_entries(&self) -> Vec<MenuEntry> {
        self.menu()
            .entries
            .iter()
            .map(Material3OverlayMenuEntryV1::to_menu_entry)
            .collect()
    }

    pub(crate) fn selected_value(&self) -> Option<Arc<str>> {
        self.select()
            .selected_value
            .as_deref()
            .map(Arc::<str>::from)
    }

    pub(crate) fn error_selected_value(&self) -> Option<Arc<str>> {
        self.select()
            .error_selected_value
            .as_deref()
            .map(Arc::<str>::from)
    }

    pub(crate) fn select_items(&self) -> Arc<[SelectItem]> {
        self.select()
            .items
            .iter()
            .map(Material3OverlaySelectItemV1::to_select_item)
            .collect::<Vec<_>>()
            .into()
    }

    pub(crate) fn select_trigger(&self) -> &Material3OverlaySelectTriggerV1 {
        &self.select().trigger
    }

    pub(crate) fn select_error_trigger(&self) -> &Material3OverlaySelectTriggerV1 {
        &self.select().error_trigger
    }

    pub(crate) fn select_hover_selected_item_test_id(&self) -> &str {
        &self.select().hover_selected_item_test_id
    }

    pub(crate) fn select_open_snapshot_id(&self) -> &str {
        &self.snapshots().open
    }

    pub(crate) fn select_trigger_snapshot_id(&self) -> &str {
        &self.snapshots().trigger
    }

    pub(crate) fn select_hover_snapshot_id(&self) -> &str {
        &self.snapshots().hover_selected
    }

    fn select(&self) -> &Material3OverlaySelectV1 {
        self.select
            .as_ref()
            .unwrap_or_else(|| panic!("{}: expected select fixture", self.id))
    }

    fn snapshots(&self) -> &Material3OverlaySelectSnapshotsV1 {
        self.snapshots
            .as_ref()
            .unwrap_or_else(|| panic!("{}: expected select snapshot ids", self.id))
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Material3OverlayTooltipV1 {
    kind: Material3OverlayTooltipKindV1,
    trigger_label: String,
    trigger_test_id: String,
    title: Option<String>,
    supporting_text: String,
}

impl Material3OverlayTooltipV1 {
    pub(crate) fn kind(&self) -> Material3OverlayTooltipKindV1 {
        self.kind
    }

    pub(crate) fn trigger_label(&self) -> &str {
        &self.trigger_label
    }

    pub(crate) fn trigger_test_id(&self) -> &str {
        &self.trigger_test_id
    }

    pub(crate) fn title(&self) -> &str {
        self.title
            .as_deref()
            .expect("rich tooltip case must define title")
    }

    pub(crate) fn supporting_text(&self) -> &str {
        &self.supporting_text
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Material3OverlayTooltipKindV1 {
    Plain,
    Rich,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Material3OverlayMenuV1 {
    a11y_label: String,
    test_id: String,
    trigger_label: String,
    trigger_test_id: String,
    entries: Vec<Material3OverlayMenuEntryV1>,
}

impl Material3OverlayMenuV1 {
    pub(crate) fn a11y_label(&self) -> &str {
        &self.a11y_label
    }

    pub(crate) fn test_id(&self) -> &str {
        &self.test_id
    }

    pub(crate) fn trigger_label(&self) -> &str {
        &self.trigger_label
    }

    pub(crate) fn trigger_test_id(&self) -> &str {
        &self.trigger_test_id
    }
}

#[derive(Debug, Deserialize)]
struct Material3OverlayMenuEntryV1 {
    label: String,
    test_id: String,
}

impl Material3OverlayMenuEntryV1 {
    fn to_menu_entry(&self) -> MenuEntry {
        MenuEntry::Item(MenuItem::new(self.label.clone()).test_id(self.test_id.clone()))
    }
}

#[derive(Debug, Deserialize)]
struct Material3OverlaySelectSnapshotsV1 {
    open: String,
    trigger: String,
    hover_selected: String,
}

#[derive(Debug, Deserialize)]
struct Material3OverlaySelectV1 {
    selected_value: Option<String>,
    error_selected_value: Option<String>,
    trigger: Material3OverlaySelectTriggerV1,
    error_trigger: Material3OverlaySelectTriggerV1,
    hover_selected_item_test_id: String,
    items: Vec<Material3OverlaySelectItemV1>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Material3OverlaySelectTriggerV1 {
    leading_icon: Option<Material3HeadlessIconV1>,
    label: String,
    supporting_text: String,
    a11y_label: String,
    placeholder: String,
    test_id: String,
    #[serde(default)]
    error: bool,
}

impl Material3OverlaySelectTriggerV1 {
    pub(crate) fn leading_icon(&self) -> Option<fret_icons::IconId> {
        self.leading_icon.map(Material3HeadlessIconV1::icon_id)
    }

    pub(crate) fn label(&self) -> &str {
        &self.label
    }

    pub(crate) fn supporting_text(&self) -> &str {
        &self.supporting_text
    }

    pub(crate) fn a11y_label(&self) -> &str {
        &self.a11y_label
    }

    pub(crate) fn placeholder(&self) -> &str {
        &self.placeholder
    }

    pub(crate) fn test_id(&self) -> &str {
        &self.test_id
    }

    pub(crate) fn error(&self) -> bool {
        self.error
    }
}

#[derive(Debug, Deserialize)]
struct Material3OverlaySelectItemV1 {
    value: String,
    label: String,
    leading_icon: Option<Material3HeadlessIconV1>,
    trailing_icon: Option<Material3HeadlessIconV1>,
    #[serde(default)]
    disabled: bool,
    test_id: String,
}

impl Material3OverlaySelectItemV1 {
    fn to_select_item(&self) -> SelectItem {
        let mut item = SelectItem::new(self.value.clone(), self.label.clone());
        if let Some(icon) = self.leading_icon {
            item = item.leading_icon(icon.icon_id());
        }
        if let Some(icon) = self.trailing_icon {
            item = item.trailing_icon(icon.icon_id());
        }
        if self.disabled {
            item = item.disabled(true);
        }
        item.test_id(self.test_id.clone())
    }
}

pub(crate) fn load_material3_overlay_golden_suite_v1() -> Material3OverlayGoldenSuiteV1 {
    let suite: Material3OverlayGoldenSuiteV1 =
        serde_json::from_str(MATERIAL3_HEADLESS_OVERLAYS_CASES_V1)
            .expect("material3 overlays golden fixture must parse");
    assert_material3_headless_schema_version(suite.schema_version, 1, "overlays golden");
    suite
}
