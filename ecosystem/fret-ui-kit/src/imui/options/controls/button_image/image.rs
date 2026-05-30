use std::sync::Arc;

use fret_core::ViewportFit;
use fret_core::scene::{ImageSamplingHint, UvRect};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ImageItemVariant {
    #[default]
    Image,
    Button,
}

#[derive(Debug, Clone)]
pub struct ImageItemOptions {
    pub enabled: bool,
    pub focusable: bool,
    pub variant: ImageItemVariant,
    pub fit: ViewportFit,
    pub sampling: ImageSamplingHint,
    pub opacity: f32,
    pub uv: Option<UvRect>,
    pub a11y_label: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
}

impl Default for ImageItemOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            focusable: false,
            variant: ImageItemVariant::Image,
            fit: ViewportFit::Stretch,
            sampling: ImageSamplingHint::Default,
            opacity: 1.0,
            uv: None,
            a11y_label: None,
            test_id: None,
        }
    }
}

impl ImageItemOptions {
    pub fn button() -> Self {
        Self {
            focusable: true,
            variant: ImageItemVariant::Button,
            ..Self::default()
        }
    }

    pub fn fit(mut self, fit: ViewportFit) -> Self {
        self.fit = fit;
        self
    }

    pub fn sampling(mut self, sampling: ImageSamplingHint) -> Self {
        self.sampling = sampling;
        self
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    pub fn uv(mut self, uv: UvRect) -> Self {
        self.uv = Some(uv);
        self
    }

    pub fn with_a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn with_test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }
}
