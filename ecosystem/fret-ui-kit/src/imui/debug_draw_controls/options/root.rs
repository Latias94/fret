use std::sync::Arc;

use fret_core::Px;
use fret_ui::element::{LayoutStyle, Length, SizeStyle};

#[derive(Debug, Clone)]
pub struct DebugDrawOptions {
    pub layout: LayoutStyle,
    pub test_id: Option<Arc<str>>,
    pub clip_to_bounds: bool,
    pub interaction: DebugDrawInteractionOptions,
}

impl Default for DebugDrawOptions {
    fn default() -> Self {
        Self {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Px(Px(120.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
            test_id: None,
            clip_to_bounds: true,
            interaction: DebugDrawInteractionOptions::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DebugDrawInteractionOptions {
    pub enabled: bool,
    pub focusable: bool,
    pub a11y_label: Option<Arc<str>>,
}

impl DebugDrawInteractionOptions {
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            ..Default::default()
        }
    }

    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    pub fn with_a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }
}
