use fret_core::{Color, Corners, Px};
use fret_ui_kit::{ColorRef, WidgetStateProperty};
use fret_ui_material3::menu::{MenuEntry, MenuItem, MenuStyle};
use fret_ui_material3::{DialogAction, DialogStyle};
use serde::Deserialize;

const MATERIAL3_HEADLESS_MENU_DIALOG_STYLE_CASES_V1: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/material3_headless_menu_dialog_style_cases_v1.json"
));

#[derive(Debug, Deserialize)]
pub(crate) struct Material3MenuDialogStyleGoldenSuiteV1 {
    schema_version: u32,
    menu_entries: Vec<Material3MenuEntryV1>,
    cases: Vec<Material3MenuDialogStyleGoldenCaseV1>,
}

impl Material3MenuDialogStyleGoldenSuiteV1 {
    pub(crate) fn menu_entries(&self) -> Vec<MenuEntry> {
        self.menu_entries
            .iter()
            .map(Material3MenuEntryV1::to_menu_entry)
            .collect()
    }

    pub(crate) fn case(
        &self,
        kind: Material3MenuDialogStyleGoldenCaseKindV1,
    ) -> &Material3MenuDialogStyleGoldenCaseV1 {
        self.cases
            .iter()
            .find(|case| case.kind == kind)
            .unwrap_or_else(|| panic!("material3 menu/dialog style fixture missing {kind:?}"))
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Material3MenuEntryV1 {
    Item {
        label: String,
        test_id: Option<String>,
        #[serde(default)]
        disabled: bool,
    },
    Separator,
}

impl Material3MenuEntryV1 {
    fn to_menu_entry(&self) -> MenuEntry {
        match self {
            Self::Item {
                label,
                test_id,
                disabled,
            } => {
                let mut item = MenuItem::new(label.clone()).disabled(*disabled);
                if let Some(test_id) = test_id.as_ref() {
                    item = item.test_id(test_id.clone());
                }
                MenuEntry::Item(item)
            }
            Self::Separator => MenuEntry::Separator,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Material3MenuDialogStyleGoldenCaseV1 {
    id: String,
    kind: Material3MenuDialogStyleGoldenCaseKindV1,
    settle_from_frame: usize,
    total_frames: usize,
    headline: Option<String>,
    supporting_text: Option<String>,
    #[serde(default)]
    actions: Vec<String>,
    style: Option<Material3MenuDialogStyleOverrideV1>,
}

impl Material3MenuDialogStyleGoldenCaseV1 {
    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn settle_from_frame(&self) -> usize {
        self.settle_from_frame
    }

    pub(crate) fn total_frames(&self) -> usize {
        self.total_frames
    }

    pub(crate) fn headline(&self) -> &str {
        self.headline
            .as_deref()
            .unwrap_or_else(|| panic!("{}: expected dialog headline", self.id))
    }

    pub(crate) fn supporting_text(&self) -> &str {
        self.supporting_text
            .as_deref()
            .unwrap_or_else(|| panic!("{}: expected dialog supporting text", self.id))
    }

    pub(crate) fn dialog_actions(&self) -> Vec<DialogAction> {
        self.actions
            .iter()
            .map(|label| DialogAction::new(label.clone()))
            .collect()
    }

    pub(crate) fn menu_style(&self) -> MenuStyle {
        let style = self
            .style
            .as_ref()
            .unwrap_or_else(|| panic!("{}: expected menu style override", self.id));

        let mut menu_style = MenuStyle::default();
        if let Some(color) = style.container_background {
            menu_style = menu_style.container_background(WidgetStateProperty::new(Some(
                ColorRef::Color(color.to_color()),
            )));
        }
        if let Some(radius) = style.container_corner_radius {
            menu_style = menu_style
                .container_corner_radii(WidgetStateProperty::new(Some(Corners::all(Px(radius)))));
        }
        if let Some(elevation) = style.container_elevation {
            menu_style =
                menu_style.container_elevation(WidgetStateProperty::new(Some(Px(elevation))));
        }
        if let Some(color) = style.item_label_color {
            menu_style = menu_style.item_label_color(WidgetStateProperty::new(Some(
                ColorRef::Color(color.to_color()),
            )));
        }
        menu_style
    }

    pub(crate) fn dialog_style(&self) -> DialogStyle {
        let style = self
            .style
            .as_ref()
            .unwrap_or_else(|| panic!("{}: expected dialog style override", self.id));

        let mut dialog_style = DialogStyle::default();
        if let Some(color) = style.container_background {
            dialog_style = dialog_style.container_background(WidgetStateProperty::new(Some(
                ColorRef::Color(color.to_color()),
            )));
        }
        if let Some(radius) = style.container_corner_radius {
            dialog_style = dialog_style
                .container_corner_radii(WidgetStateProperty::new(Some(Corners::all(Px(radius)))));
        }
        if let Some(elevation) = style.container_elevation {
            dialog_style =
                dialog_style.container_elevation(WidgetStateProperty::new(Some(Px(elevation))));
        }
        if let Some(color) = style.headline_color {
            dialog_style = dialog_style.headline_color(WidgetStateProperty::new(Some(
                ColorRef::Color(color.to_color()),
            )));
        }
        if let Some(color) = style.supporting_text_color {
            dialog_style = dialog_style.supporting_text_color(WidgetStateProperty::new(Some(
                ColorRef::Color(color.to_color()),
            )));
        }
        dialog_style
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Material3MenuDialogStyleGoldenCaseKindV1 {
    MenuDefaultVsOverride,
    DialogDefault,
    DialogOverride,
}

#[derive(Debug, Deserialize)]
struct Material3MenuDialogStyleOverrideV1 {
    container_background: Option<Material3ColorV1>,
    container_corner_radius: Option<f32>,
    container_elevation: Option<f32>,
    item_label_color: Option<Material3ColorV1>,
    headline_color: Option<Material3ColorV1>,
    supporting_text_color: Option<Material3ColorV1>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct Material3ColorV1 {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Material3ColorV1 {
    fn to_color(self) -> Color {
        Color {
            r: self.r,
            g: self.g,
            b: self.b,
            a: self.a,
        }
    }
}

pub(crate) fn load_material3_menu_dialog_style_golden_suite_v1()
-> Material3MenuDialogStyleGoldenSuiteV1 {
    let suite: Material3MenuDialogStyleGoldenSuiteV1 =
        serde_json::from_str(MATERIAL3_HEADLESS_MENU_DIALOG_STYLE_CASES_V1)
            .expect("material3 menu/dialog style golden fixture must parse");
    assert_eq!(
        suite.schema_version, 1,
        "material3 menu/dialog style golden fixture schema version"
    );
    suite
}
