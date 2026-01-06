use fret_core::{Color, Corners, Edges, Px};

use crate::ThemeSnapshot;
use crate::element::RingStyle;

#[derive(Debug, Clone)]
pub struct TextInputStyle {
    pub padding: Edges,
    pub background: Color,
    pub border: Edges,
    pub border_color: Color,
    pub border_color_focused: Color,
    pub focus_ring: Option<RingStyle>,
    pub corner_radii: Corners,
    pub text_color: Color,
    pub placeholder_color: Color,
    pub selection_color: Color,
    pub caret_color: Color,
    pub preedit_color: Color,
}

impl Default for TextInputStyle {
    fn default() -> Self {
        Self {
            padding: Edges {
                top: Px(6.0),
                right: Px(8.0),
                bottom: Px(6.0),
                left: Px(8.0),
            },
            background: Color {
                r: 0.12,
                g: 0.12,
                b: 0.16,
                a: 1.0,
            },
            border: Edges::all(Px(1.0)),
            border_color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.35,
            },
            border_color_focused: Color {
                r: 0.6,
                g: 0.75,
                b: 1.0,
                a: 0.9,
            },
            focus_ring: None,
            corner_radii: Corners::all(Px(6.0)),
            text_color: Color {
                r: 0.92,
                g: 0.92,
                b: 0.92,
                a: 1.0,
            },
            placeholder_color: Color {
                r: 0.92,
                g: 0.92,
                b: 0.92,
                a: 0.5,
            },
            selection_color: Color {
                r: 0.24,
                g: 0.34,
                b: 0.52,
                a: 1.0,
            },
            caret_color: Color {
                r: 0.90,
                g: 0.90,
                b: 0.92,
                a: 1.0,
            },
            preedit_color: Color {
                r: 0.85,
                g: 0.65,
                b: 0.95,
                a: 1.0,
            },
        }
    }
}

impl TextInputStyle {
    pub fn from_theme(theme: ThemeSnapshot) -> Self {
        Self {
            padding: Edges {
                top: Px(6.0),
                right: theme.metric_required("metric.padding.sm"),
                bottom: Px(6.0),
                left: theme.metric_required("metric.padding.sm"),
            },
            background: theme.color_required("card"),
            border: Edges::all(Px(1.0)),
            border_color: theme.color_required("border"),
            border_color_focused: theme.color_required("ring"),
            focus_ring: None,
            corner_radii: Corners::all(theme.metric_required("metric.radius.sm")),
            text_color: theme.color_required("foreground"),
            placeholder_color: theme.color_required("muted-foreground"),
            selection_color: Color {
                a: 1.0,
                ..theme.color_required("selection.background")
            },
            caret_color: theme.color_required("foreground"),
            preedit_color: theme.color_required("primary"),
        }
    }
}
