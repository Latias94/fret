use fret_core::{Color, Corners, Edges, Px};
use fret_ui::Theme;

use crate::style::{ColorFallback, MetricFallback};
use crate::{Size, StyleRefinement};

#[derive(Debug, Clone, Copy)]
pub struct InputTokenKeys {
    pub padding_x: Option<&'static str>,
    pub padding_y: Option<&'static str>,
    pub min_height: Option<&'static str>,
    pub radius: Option<&'static str>,
    pub border_width: Option<&'static str>,
    pub bg: Option<&'static str>,
    pub border: Option<&'static str>,
    pub border_focus: Option<&'static str>,
    pub fg: Option<&'static str>,
    pub text_px: Option<&'static str>,
    pub selection: Option<&'static str>,
}

impl InputTokenKeys {
    pub const fn none() -> Self {
        Self {
            padding_x: None,
            padding_y: None,
            min_height: None,
            radius: None,
            border_width: None,
            bg: None,
            border: None,
            border_focus: None,
            fg: None,
            text_px: None,
            selection: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ResolvedInputChrome {
    pub padding_x: Px,
    pub padding_y: Px,
    pub min_height: Px,
    pub radius: Px,
    pub border_width: Px,
    pub background: Color,
    pub border_color: Color,
    pub border_color_focused: Color,
    pub text_color: Color,
    pub text_px: Px,
    pub selection_color: Color,
}

pub fn resolve_input_chrome(
    theme: &Theme,
    size: Size,
    style: &StyleRefinement,
    keys: InputTokenKeys,
) -> ResolvedInputChrome {
    // Priority:
    // 1) callsite style refinement
    // 2) component-specific token keys (if provided)
    // 3) shared input-family token keys
    // 4) size/baseline theme fallbacks

    let padding_x = style
        .padding_x
        .as_ref()
        .map(|m| m.resolve(theme))
        .or_else(|| keys.padding_x.and_then(|k| theme.metric_by_key(k)))
        .or_else(|| theme.metric_by_key("component.input.padding_x"))
        .unwrap_or_else(|| size.input_px(theme));
    let padding_y = style
        .padding_y
        .as_ref()
        .map(|m| m.resolve(theme))
        .or_else(|| keys.padding_y.and_then(|k| theme.metric_by_key(k)))
        .or_else(|| theme.metric_by_key("component.input.padding_y"))
        .unwrap_or_else(|| size.input_py(theme));
    let min_height = style
        .min_height
        .as_ref()
        .map(|m| m.resolve(theme))
        .or_else(|| keys.min_height.and_then(|k| theme.metric_by_key(k)))
        .or_else(|| theme.metric_by_key("component.input.min_height"))
        .unwrap_or_else(|| size.input_h(theme));
    let radius = style
        .radius
        .as_ref()
        .map(|m| m.resolve(theme))
        .or_else(|| keys.radius.and_then(|k| theme.metric_by_key(k)))
        .or_else(|| theme.metric_by_key("component.input.radius"))
        .unwrap_or_else(|| size.control_radius(theme));
    let border_width = style
        .border_width
        .as_ref()
        .map(|m| m.resolve(theme))
        .or_else(|| keys.border_width.and_then(|k| theme.metric_by_key(k)))
        .or_else(|| theme.metric_by_key("component.input.border_width"))
        .unwrap_or(Px(1.0));

    let background = style
        .background
        .as_ref()
        .map(|c| c.resolve(theme))
        .or_else(|| keys.bg.and_then(|k| theme.color_by_key(k)))
        .or_else(|| theme.color_by_key("component.input.bg"))
        .unwrap_or(theme.colors.panel_background);
    let border_color = style
        .border_color
        .as_ref()
        .map(|c| c.resolve(theme))
        .or_else(|| keys.border.and_then(|k| theme.color_by_key(k)))
        .or_else(|| theme.color_by_key("component.input.border"))
        .unwrap_or(theme.colors.panel_border);
    let border_color_focused = keys
        .border_focus
        .and_then(|k| theme.color_by_key(k))
        .or_else(|| theme.color_by_key("component.input.border_focus"))
        .unwrap_or(theme.colors.focus_ring);
    let text_color = style
        .text_color
        .as_ref()
        .map(|c| c.resolve(theme))
        .or_else(|| keys.fg.and_then(|k| theme.color_by_key(k)))
        .or_else(|| theme.color_by_key("component.input.fg"))
        .unwrap_or(theme.colors.text_primary);
    let text_px = keys
        .text_px
        .and_then(|k| theme.metric_by_key(k))
        .or_else(|| theme.metric_by_key("component.input.text_px"))
        .unwrap_or_else(|| size.control_text_px(theme));
    let selection_color = keys
        .selection
        .and_then(|k| theme.color_by_key(k))
        .or_else(|| theme.color_by_key("component.input.selection"))
        .unwrap_or(theme.colors.selection_background);

    ResolvedInputChrome {
        padding_x: Px(padding_x.0.max(0.0)),
        padding_y: Px(padding_y.0.max(0.0)),
        min_height: Px(min_height.0.max(0.0)),
        radius: Px(radius.0.max(0.0)),
        border_width: Px(border_width.0.max(0.0)),
        background,
        border_color,
        border_color_focused,
        text_color,
        text_px,
        selection_color,
    }
}

pub fn default_text_input_style(theme: &Theme) -> fret_ui::TextInputStyle {
    fret_ui::TextInputStyle {
        padding_x: Px(0.0),
        padding_y: Px(0.0),
        background: theme.colors.panel_background,
        border: Edges::all(Px(1.0)),
        border_color: theme.colors.panel_border,
        border_color_focused: theme.colors.focus_ring,
        corner_radii: Corners::all(theme.metrics.radius_sm),
        text_color: theme.colors.text_primary,
        selection_color: Color {
            a: 1.0,
            ..theme.colors.selection_background
        },
        caret_color: theme.colors.text_primary,
        preedit_color: theme.colors.accent,
    }
}

pub fn input_base_refinement() -> StyleRefinement {
    StyleRefinement {
        border_width: Some(crate::MetricRef::Token {
            key: "component.input.border_width",
            fallback: MetricFallback::Px(Px(1.0)),
        }),
        radius: Some(crate::MetricRef::Token {
            key: "component.input.radius",
            fallback: MetricFallback::ThemeRadiusSm,
        }),
        background: Some(crate::ColorRef::Token {
            key: "component.input.bg",
            fallback: ColorFallback::ThemePanelBackground,
        }),
        border_color: Some(crate::ColorRef::Token {
            key: "component.input.border",
            fallback: ColorFallback::ThemePanelBorder,
        }),
        text_color: Some(crate::ColorRef::Token {
            key: "component.input.fg",
            fallback: ColorFallback::ThemeTextPrimary,
        }),
        ..StyleRefinement::default()
    }
}
