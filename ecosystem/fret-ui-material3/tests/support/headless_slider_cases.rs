use fret_runtime::Model;
use fret_ui_material3::{RangeSlider, Slider};
use serde::Deserialize;

const MATERIAL3_HEADLESS_SLIDER_CASES_V1: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/material3_headless_slider_cases_v1.json"
));

#[derive(Debug, Deserialize)]
pub(crate) struct Material3SliderGoldenSuiteV1 {
    schema_version: u32,
    settings: Material3SliderSettingsV1,
    single_value_models: Vec<Material3SliderValueModelDefinitionV1>,
    range_value_models: Vec<Material3RangeSliderValueModelDefinitionV1>,
    sliders: Vec<Material3SliderDefinitionV1>,
    range_sliders: Vec<Material3RangeSliderDefinitionV1>,
    cases: Vec<Material3SliderGoldenCaseV1>,
}

impl Material3SliderGoldenSuiteV1 {
    pub(crate) fn single_value_models(&self) -> &[Material3SliderValueModelDefinitionV1] {
        &self.single_value_models
    }

    pub(crate) fn range_value_models(&self) -> &[Material3RangeSliderValueModelDefinitionV1] {
        &self.range_value_models
    }

    pub(crate) fn sliders(&self) -> &[Material3SliderDefinitionV1] {
        &self.sliders
    }

    pub(crate) fn range_sliders(&self) -> &[Material3RangeSliderDefinitionV1] {
        &self.range_sliders
    }

    pub(crate) fn cases(&self) -> &[Material3SliderGoldenCaseV1] {
        &self.cases
    }

    pub(crate) fn render_config_for(
        &self,
        case: &Material3SliderGoldenCaseV1,
    ) -> Material3SliderRenderConfigV1 {
        Material3SliderRenderConfigV1 {
            min: self.settings.min,
            max: self.settings.max,
            step: case.step.unwrap_or(self.settings.default_step),
            with_tick_marks: case.with_tick_marks,
            tick_marks_count: case.tick_marks_count,
        }
    }
}

#[derive(Debug, Deserialize)]
struct Material3SliderSettingsV1 {
    min: f32,
    max: f32,
    default_step: f32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Material3SliderRenderConfigV1 {
    min: f32,
    max: f32,
    step: f32,
    with_tick_marks: bool,
    tick_marks_count: Option<u16>,
}

impl Material3SliderRenderConfigV1 {
    fn slider(self, model: Model<f32>) -> Slider {
        let slider = Slider::new(model)
            .range(self.min, self.max)
            .step(self.step)
            .with_tick_marks(self.with_tick_marks);
        if let Some(count) = self.tick_marks_count {
            slider.tick_marks_count(count)
        } else {
            slider
        }
    }

    fn range_slider(self, model: Model<[f32; 2]>) -> RangeSlider {
        let slider = RangeSlider::new(model)
            .range(self.min, self.max)
            .step(self.step)
            .with_tick_marks(self.with_tick_marks);
        if let Some(count) = self.tick_marks_count {
            slider.tick_marks_count(count)
        } else {
            slider
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Material3SliderValueModelDefinitionV1 {
    id: String,
    value: f32,
}

impl Material3SliderValueModelDefinitionV1 {
    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn value(&self) -> f32 {
        self.value
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Material3RangeSliderValueModelDefinitionV1 {
    id: String,
    values: [f32; 2],
}

impl Material3RangeSliderValueModelDefinitionV1 {
    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn values(&self) -> [f32; 2] {
        self.values
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Material3SliderDefinitionV1 {
    test_id: String,
    model_id: String,
    #[serde(default)]
    disabled: bool,
}

impl Material3SliderDefinitionV1 {
    pub(crate) fn model_id(&self) -> &str {
        &self.model_id
    }

    pub(crate) fn slider(
        &self,
        model: Model<f32>,
        config: Material3SliderRenderConfigV1,
    ) -> Slider {
        let slider = config.slider(model);
        let slider = if self.disabled {
            slider.disabled(true)
        } else {
            slider
        };
        slider.test_id(self.test_id.clone())
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Material3RangeSliderDefinitionV1 {
    test_id: String,
    model_id: String,
    #[serde(default)]
    disabled: bool,
}

impl Material3RangeSliderDefinitionV1 {
    pub(crate) fn model_id(&self) -> &str {
        &self.model_id
    }

    pub(crate) fn range_slider(
        &self,
        model: Model<[f32; 2]>,
        config: Material3SliderRenderConfigV1,
    ) -> RangeSlider {
        let slider = config.range_slider(model);
        let slider = if self.disabled {
            slider.disabled(true)
        } else {
            slider
        };
        slider.test_id(self.test_id.clone())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Material3SliderPointerInteractionV1 {
    Pressed,
    Dragging,
    RangeDragging,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Material3SliderKeyboardInteractionV1 {
    SingleArrowCycle,
    SinglePageHomeEnd,
    SingleRtlArrowCycle,
    RangeThumbSwitch,
    RangePageHomeEnd,
    RangeRtlArrowCycle,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Material3SliderGoldenCaseV1 {
    id: String,
    #[serde(default)]
    rtl: bool,
    hover_test_id: Option<String>,
    focus_test_id: Option<String>,
    pointer_interaction: Option<Material3SliderPointerInteractionV1>,
    keyboard_interaction: Option<Material3SliderKeyboardInteractionV1>,
    assert_model_id: Option<String>,
    secondary_focus_test_id: Option<String>,
    step: Option<f32>,
    #[serde(default)]
    with_tick_marks: bool,
    tick_marks_count: Option<u16>,
    settle_from_frame: usize,
    total_frames: usize,
}

impl Material3SliderGoldenCaseV1 {
    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn is_rtl(&self) -> bool {
        self.rtl
    }

    pub(crate) fn hover_test_id(&self) -> Option<&str> {
        self.hover_test_id.as_deref()
    }

    pub(crate) fn focus_test_id(&self) -> Option<&str> {
        self.focus_test_id.as_deref()
    }

    pub(crate) fn pointer_interaction(&self) -> Option<Material3SliderPointerInteractionV1> {
        self.pointer_interaction
    }

    pub(crate) fn keyboard_interaction(&self) -> Option<Material3SliderKeyboardInteractionV1> {
        self.keyboard_interaction
    }

    pub(crate) fn assert_model_id(&self) -> Option<&str> {
        self.assert_model_id.as_deref()
    }

    pub(crate) fn secondary_focus_test_id(&self) -> Option<&str> {
        self.secondary_focus_test_id.as_deref()
    }

    pub(crate) fn settle_from_frame(&self) -> usize {
        self.settle_from_frame
    }

    pub(crate) fn total_frames(&self) -> usize {
        self.total_frames
    }
}

pub(crate) fn load_material3_slider_golden_suite_v1() -> Material3SliderGoldenSuiteV1 {
    let suite: Material3SliderGoldenSuiteV1 =
        serde_json::from_str(MATERIAL3_HEADLESS_SLIDER_CASES_V1)
            .expect("material3 slider golden fixture must parse");
    assert_eq!(
        suite.schema_version, 1,
        "material3 slider golden fixture schema version"
    );
    suite
}
