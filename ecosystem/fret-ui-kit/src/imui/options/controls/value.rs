use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SliderOptions {
    pub enabled: bool,
    pub focusable: bool,
    pub a11y_label: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub min: f32,
    pub max: f32,
    pub step: f32,
}

impl Default for SliderOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            focusable: true,
            a11y_label: None,
            test_id: None,
            min: 0.0,
            max: 100.0,
            step: 1.0,
        }
    }
}
