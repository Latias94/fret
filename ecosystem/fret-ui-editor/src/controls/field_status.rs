//! Field status helpers for property grids and inspector-like surfaces.
//!
//! This stays UI-light: it does not assume a specific async/query stack. Higher-level adapters
//! can live behind `state-*` features.

use std::sync::Arc;

use fret_core::text::{TextOverflow, TextWrap};
use fret_core::{Color, Corners, Edges, FontWeight, Px, TextAlign, TextStyle};
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

        let (bg, border, fg, label) = status_badge_palette(theme, &self.status);

        let badge_h = Px((density.row_height.0 - 4.0).max(14.0));
        let text_style = typography::as_control_text(TextStyle {
            size: Px(10.0),
            weight: FontWeight::SEMIBOLD,
            line_height: Some(badge_h),
            ..Default::default()
        });

        cx.container(
            ContainerProps {
                layout: self.options.layout,
                padding: Edges::symmetric(Px(6.0), Px(0.0)).into(),
                background: Some(bg),
                border: Edges::all(Px(1.0)),
                border_color: Some(border),
                corner_radii: Corners::all(Px(6.0)),
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

fn status_badge_palette(theme: &Theme, status: &FieldStatus) -> (Color, Color, Color, Arc<str>) {
    let surface = theme.color_token("background");
    let border_base = theme
        .color_by_key("border")
        .or_else(|| theme.color_by_key("component.text_field.border"))
        .unwrap_or_else(|| theme.color_token("foreground"));
    let foreground = theme.color_token("foreground");
    let muted_foreground = theme
        .color_by_key("muted-foreground")
        .or_else(|| theme.color_by_key("muted_foreground"))
        .unwrap_or(foreground);
    let accent = theme.color_token("accent");
    let secondary = theme
        .color_by_key("secondary")
        .unwrap_or_else(|| theme.color_token("muted"));
    let destructive = theme.color_token("destructive");

    let (tint, bg_mix, border_mix, fg_mix) = match status {
        FieldStatus::Loading => (muted_foreground, 0.10, 0.20, 0.35),
        FieldStatus::Dirty => (accent, 0.22, 0.50, 0.12),
        FieldStatus::Mixed => (secondary, 0.18, 0.34, 0.08),
        FieldStatus::Error(_) => (destructive, 0.24, 0.62, 0.04),
    };

    (
        mix(surface, tint, bg_mix),
        mix(border_base, tint, border_mix),
        mix(foreground, tint, fg_mix),
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use fret_app::App;
    use fret_ui::Theme;

    use super::{FieldStatus, status_badge_palette};

    #[test]
    fn error_badge_palette_keeps_short_visible_label() {
        let app = App::new();
        let theme = Theme::global(&app);

        let (_, _, _, label) = status_badge_palette(theme, &FieldStatus::Error(Arc::from("stub")));

        assert_eq!(label.as_ref(), "Error");
    }
}
