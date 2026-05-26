use std::sync::Arc;

use fret_core::scene::{ImageSamplingHint, UvRect};
use fret_core::{Size, ViewportFit};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonArrowDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum ButtonVariant {
    #[default]
    Default,
    Small,
    Arrow(ButtonArrowDirection),
    Invisible {
        size: Size,
    },
}

#[derive(Debug, Clone)]
pub struct ButtonOptions {
    pub enabled: bool,
    pub focusable: bool,
    pub variant: ButtonVariant,
    pub a11y_label: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    /// Exact key chord that activates the button while it is focused.
    ///
    /// This is an item-local shortcut seam. It does not participate in global shortcut ownership
    /// arbitration.
    pub activate_shortcut: Option<fret_runtime::KeyChord>,
    /// Whether `activate_shortcut` should fire on repeated keydown events.
    pub shortcut_repeat: bool,
}

impl Default for ButtonOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            focusable: true,
            variant: ButtonVariant::Default,
            a11y_label: None,
            test_id: None,
            activate_shortcut: None,
            shortcut_repeat: false,
        }
    }
}

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
