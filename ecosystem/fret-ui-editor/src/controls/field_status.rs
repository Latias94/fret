//! Field status helpers for property grids and inspector-like surfaces.
//!
//! This stays UI-light: it does not assume a specific async/query stack. Higher-level adapters
//! can live behind `state-*` features.

use std::sync::Arc;

use fret_core::text::{TextOverflow, TextWrap};
use fret_core::{Edges, Px, TextAlign, TextStyle};
use fret_ui::element::{AnyElement, ContainerProps, LayoutStyle, Length, SizeStyle, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::typography;

use crate::primitives::EditorDensity;

#[derive(Debug, Clone)]
pub enum FieldStatus {
    Loading,
    Dirty,
    Mixed,
    Error(Arc<str>),
}

impl FieldStatus {
    pub fn label(&self) -> Arc<str> {
        match self {
            Self::Loading => Arc::from("Loading…"),
            Self::Dirty => Arc::from("Dirty"),
            Self::Mixed => Arc::from("Mixed"),
            Self::Error(_) => Arc::from("Error"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FieldStatusBadgeOptions {
    pub layout: LayoutStyle,
}

impl Default for FieldStatusBadgeOptions {
    fn default() -> Self {
        Self {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Auto,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }
}

#[derive(Clone)]
pub struct FieldStatusBadge {
    status: FieldStatus,
    options: FieldStatusBadgeOptions,
}

impl FieldStatusBadge {
    pub fn new(status: FieldStatus) -> Self {
        Self {
            status,
            options: FieldStatusBadgeOptions::default(),
        }
    }

    pub fn options(mut self, options: FieldStatusBadgeOptions) -> Self {
        self.options = options;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app);
        let density = EditorDensity::resolve(theme);

        let (bg, fg, label) = match &self.status {
            FieldStatus::Loading => (
                theme.color_token("muted"),
                theme.color_token("muted-foreground"),
                self.status.label(),
            ),
            FieldStatus::Dirty => (
                theme.color_token("accent"),
                theme.color_token("accent-foreground"),
                self.status.label(),
            ),
            FieldStatus::Mixed => (
                theme.color_token("secondary"),
                theme.color_token("secondary-foreground"),
                self.status.label(),
            ),
            FieldStatus::Error(msg) => (
                theme.color_token("destructive"),
                theme.color_token("destructive-foreground"),
                Arc::from(format!("Error: {msg}")),
            ),
        };

        let badge_h = Px((density.row_height.0 * 0.8).max(12.0));
        let text_style = typography::as_control_text(TextStyle {
            size: Px(10.0),
            line_height: Some(badge_h),
            ..Default::default()
        });

        cx.container(
            ContainerProps {
                layout: self.options.layout,
                padding: Edges::symmetric(Px(6.0), Px(0.0)),
                background: Some(bg),
                corner_radii: fret_core::Corners::all(Px(6.0)),
                ..Default::default()
            },
            move |cx| {
                vec![cx.text_props(TextProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Auto,
                            height: Length::Px(badge_h),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    text: label.clone(),
                    style: Some(text_style),
                    color: Some(fg),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Ellipsis,
                    align: TextAlign::Center,
                    ink_overflow: Default::default(),
                })]
            },
        )
    }
}
