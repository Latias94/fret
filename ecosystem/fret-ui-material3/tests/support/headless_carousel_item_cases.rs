use fret_core::Px;
use fret_ui::action::OnActivate;
use fret_ui_material3::{CarouselItem, CarouselItemVariant};
use serde::Deserialize;

const MATERIAL3_HEADLESS_CAROUSEL_ITEM_CASES_V1: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/material3_headless_carousel_item_cases_v1.json"
));

#[derive(Debug, Deserialize)]
pub(crate) struct Material3CarouselItemGoldenSuiteV1 {
    schema_version: u32,
    items: Vec<Material3CarouselItemDefinitionV1>,
    cases: Vec<Material3CarouselItemGoldenCaseV1>,
}

impl Material3CarouselItemGoldenSuiteV1 {
    pub(crate) fn items(&self) -> &[Material3CarouselItemDefinitionV1] {
        &self.items
    }

    pub(crate) fn cases(&self) -> &[Material3CarouselItemGoldenCaseV1] {
        &self.cases
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Material3CarouselItemDefinitionV1 {
    test_id: String,
    variant: Material3CarouselItemVariantV1,
    label: String,
    width: f32,
    height: f32,
    #[serde(default)]
    disabled: bool,
}

impl Material3CarouselItemDefinitionV1 {
    pub(crate) fn label(&self) -> &str {
        &self.label
    }

    pub(crate) fn carousel_item(&self, on_activate: OnActivate) -> CarouselItem {
        let item = CarouselItem::new()
            .variant(self.variant.to_carousel_item_variant())
            .width(Px(self.width))
            .height(Px(self.height))
            .on_activate(on_activate)
            .test_id(self.test_id.clone());

        if self.disabled {
            item.disabled(true)
        } else {
            item
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Material3CarouselItemVariantV1 {
    Standard,
    WithOutline,
}

impl Material3CarouselItemVariantV1 {
    fn to_carousel_item_variant(self) -> CarouselItemVariant {
        match self {
            Self::Standard => CarouselItemVariant::Standard,
            Self::WithOutline => CarouselItemVariant::WithOutline,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Material3CarouselItemGoldenCaseV1 {
    id: String,
    hover_test_id: Option<String>,
    focus_test_id: Option<String>,
    settle_from_frame: usize,
    total_frames: usize,
}

impl Material3CarouselItemGoldenCaseV1 {
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

pub(crate) fn load_material3_carousel_item_golden_suite_v1() -> Material3CarouselItemGoldenSuiteV1 {
    let suite: Material3CarouselItemGoldenSuiteV1 =
        serde_json::from_str(MATERIAL3_HEADLESS_CAROUSEL_ITEM_CASES_V1)
            .expect("material3 carousel item golden fixture must parse");
    assert_eq!(
        suite.schema_version, 1,
        "material3 carousel item golden fixture schema version"
    );
    suite
}
