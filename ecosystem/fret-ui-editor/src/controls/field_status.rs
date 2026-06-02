//! Field status helpers for property grids and inspector-like surfaces.
//!
//! This stays UI-light: it does not assume a specific async/query stack. Higher-level adapters
//! can live behind `state-*` features.

use std::sync::Arc;

use fret_core::{Color, Corners, Edges, Px};
use fret_ui::element::{AnyElement, ContainerProps, LayoutStyle, Length, SizeStyle};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::primitives::colors::{
    editor_accent, editor_border, editor_focus_ring, editor_foreground, editor_muted_foreground,
    editor_subtle_bg,
};
use crate::primitives::readout::editor_status_badge_text_props;
use crate::primitives::{EditorDensity, EditorTokenKeys};

#[cfg(test)]
mod tests;

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
            Self::Loading => Arc::from("Loading"),
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

        let (bg, border, fg, label) = status_badge_palette(theme, &self.status);

        let badge_h = Px((density.row_height.0 - 6.0).max(14.0));

        cx.container(
            ContainerProps {
                layout: self.options.layout,
                padding: Edges::symmetric(Px(5.0), Px(0.0)).into(),
                background: Some(bg),
                border: Edges::all(Px(1.0)),
                border_color: Some(border),
                corner_radii: Corners::all(Px(4.0)),
                ..Default::default()
            },
            move |cx| {
                vec![cx.text_props(editor_status_badge_text_props(label.clone(), fg, badge_h))]
            },
        )
    }
}

fn status_badge_palette(theme: &Theme, status: &FieldStatus) -> (Color, Color, Color, Arc<str>) {
    let field_bg = editor_subtle_bg(theme);
    let field_border = editor_border(theme);
    let foreground = editor_foreground(theme);
    let muted_foreground = editor_muted_foreground(theme);
    let accent = editor_accent(theme);
    let ring = editor_focus_ring(theme);
    let destructive = theme.color_token("destructive");

    let mixed_tint = theme
        .color_by_key(EditorTokenKeys::PROPERTY_HEADER_BORDER)
        .unwrap_or(ring);

    let (tint, fg_base, bg_mix, border_mix, fg_mix) = match status {
        FieldStatus::Loading => (muted_foreground, muted_foreground, 0.12, 0.16, 0.04),
        FieldStatus::Dirty => (accent, foreground, 0.18, 0.30, 0.16),
        FieldStatus::Mixed => (mixed_tint, muted_foreground, 0.16, 0.26, 0.12),
        FieldStatus::Error(_) => (destructive, foreground, 0.18, 0.38, 0.08),
    };

    (
        mix(field_bg, tint, bg_mix),
        mix(field_border, tint, border_mix),
        mix(fg_base, tint, fg_mix),
        status.label(),
    )
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color {
        r: lerp(a.r, b.r, t),
        g: lerp(a.g, b.g, t),
        b: lerp(a.b, b.b, t),
        a: lerp(a.a, b.a, t),
    }
}
