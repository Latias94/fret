use fret_core::{Color, Corners, Edges, Px};
use fret_ui::Theme;
use fret_ui::element::{ContainerProps, LayoutStyle, Overflow, RingPlacement, RingStyle};

use crate::style::PaddingRefinement;
use crate::style::{ColorFallback, MetricFallback};
use crate::{ChromeRefinement, Size};

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
    pub padding: Edges,
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

pub fn input_chrome_container_props(
    mut layout: LayoutStyle,
    chrome: ResolvedInputChrome,
    border_color: Color,
) -> ContainerProps {
    layout.overflow = Overflow::Clip;
    ContainerProps {
        layout,
        padding: chrome.padding,
        background: Some(chrome.background),
        background_paint: None,
        shadow: None,
        border: Edges::all(chrome.border_width),
        border_color: Some(border_color),
        border_paint: None,
        focus_ring: None,
        focus_border_color: None,
        focus_within: false,
        corner_radii: Corners::all(chrome.radius),
        snap_to_device_pixels: false,
    }
}

pub fn resolve_input_chrome(
    theme: &Theme,
    size: Size,
    style: &ChromeRefinement,
    keys: InputTokenKeys,
) -> ResolvedInputChrome {
    // Priority:
    // 1) callsite style refinement
    // 2) component-specific token keys (if provided)
    // 3) shared input-family token keys
    // 4) size/baseline theme fallbacks

    // `ChromeRefinement` supports per-edge padding (`pt/pr/pb/pl`). Inputs honor that directly,
    // while falling back to component tokens / size defaults for any edge not explicitly set.
    //
    // Note: we intentionally do *not* treat a single-edge refinement (e.g. `pr-*`) as setting the
    // entire axis: in Tailwind, `pr-*` only affects the right edge, and inputs frequently use this
    // to reserve space for trailing icons.
    let padding_x = keys
        .padding_x
        .and_then(|k| theme.metric_by_key(k))
        .or_else(|| theme.metric_by_key("component.input.padding_x"))
        .unwrap_or_else(|| size.input_px(theme));
    let padding_y = keys
        .padding_y
        .and_then(|k| theme.metric_by_key(k))
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
        .unwrap_or_else(|| theme.color_required("background"));
    let border_color = style
        .border_color
        .as_ref()
        .map(|c| c.resolve(theme))
        .or_else(|| keys.border.and_then(|k| theme.color_by_key(k)))
        .or_else(|| theme.color_by_key("component.input.border"))
        .unwrap_or_else(|| theme.color_required("input"));
    let border_color_focused = keys
        .border_focus
        .and_then(|k| theme.color_by_key(k))
        .or_else(|| theme.color_by_key("component.input.border_focus"))
        .unwrap_or_else(|| theme.color_required("ring"));
    let text_color = style
        .text_color
        .as_ref()
        .map(|c| c.resolve(theme))
        .or_else(|| keys.fg.and_then(|k| theme.color_by_key(k)))
        .or_else(|| theme.color_by_key("component.input.fg"))
        .unwrap_or_else(|| theme.color_required("foreground"));
    let text_px = keys
        .text_px
        .and_then(|k| theme.metric_by_key(k))
        .or_else(|| theme.metric_by_key("component.input.text_px"))
        .unwrap_or_else(|| size.control_text_px(theme));
    let selection_color = keys
        .selection
        .and_then(|k| theme.color_by_key(k))
        .or_else(|| theme.color_by_key("component.input.selection"))
        .unwrap_or_else(|| theme.color_required("selection.background"));

    let padding_top = style
        .padding
        .as_ref()
        .and_then(|p| p.top.as_ref())
        .map(|m| m.resolve(theme))
        .unwrap_or(padding_y);
    let padding_bottom = style
        .padding
        .as_ref()
        .and_then(|p| p.bottom.as_ref())
        .map(|m| m.resolve(theme))
        .unwrap_or(padding_y);
    let padding_left = style
        .padding
        .as_ref()
        .and_then(|p| p.left.as_ref())
        .map(|m| m.resolve(theme))
        .unwrap_or(padding_x);
    let padding_right = style
        .padding
        .as_ref()
        .and_then(|p| p.right.as_ref())
        .map(|m| m.resolve(theme))
        .unwrap_or(padding_x);

    ResolvedInputChrome {
        padding: Edges {
            top: Px(padding_top.0.max(0.0)),
            right: Px(padding_right.0.max(0.0)),
            bottom: Px(padding_bottom.0.max(0.0)),
            left: Px(padding_left.0.max(0.0)),
        },
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
    let ring_width = theme
        .metric_by_key("component.ring.width")
        .unwrap_or(Px(2.0));
    let ring_offset = theme
        .metric_by_key("component.ring.offset")
        .unwrap_or(Px(2.0));
    // shadcn/new-york-v4 uses `ring-ring/50` for the ring color.
    let ring_color = theme
        .color_by_key("ring/50")
        .or_else(|| theme.color_by_key("ring"))
        .unwrap_or_else(|| theme.color_required("ring"));
    let ring_offset_color = theme
        .color_by_key("ring-offset-background")
        .unwrap_or_else(|| theme.color_required("ring-offset-background"));

    let background = theme
        .color_by_key("component.input.bg")
        .unwrap_or_else(|| theme.color_required("background"));
    let border_color = theme
        .color_by_key("component.input.border")
        .unwrap_or_else(|| theme.color_required("input"));
    // shadcn/new-york-v4 uses `focus-visible:border-ring`.
    let border_color_focused = theme
        .color_by_key("ring")
        .unwrap_or_else(|| theme.color_required("ring"));
    let radius = theme
        .metric_by_key("component.input.radius")
        .unwrap_or_else(|| theme.metric_required("metric.radius.sm"));
    let selection = theme
        .color_by_key("component.input.selection")
        .unwrap_or_else(|| theme.color_required("selection.background"));

    fret_ui::TextInputStyle {
        padding: Edges::all(Px(0.0)),
        background,
        border: Edges::all(Px(1.0)),
        border_color,
        border_color_focused,
        focus_ring: Some(RingStyle {
            placement: RingPlacement::Outset,
            width: ring_width,
            offset: ring_offset,
            color: ring_color,
            offset_color: (ring_offset.0 > 0.0).then_some(ring_offset_color),
            corner_radii: Corners::all(radius),
        }),
        corner_radii: Corners::all(radius),
        text_color: theme.color_required("foreground"),
        placeholder_color: theme.color_required("muted-foreground"),
        selection_color: Color {
            a: 1.0,
            ..selection
        },
        caret_color: theme.color_required("foreground"),
        preedit_color: theme.color_required("primary"),
    }
}

pub fn input_base_refinement() -> ChromeRefinement {
    ChromeRefinement {
        padding: Some(PaddingRefinement {
            top: Some(crate::MetricRef::Token {
                key: "component.input.padding_y",
                fallback: MetricFallback::ThemePaddingSm,
            }),
            right: Some(crate::MetricRef::Token {
                key: "component.input.padding_x",
                fallback: MetricFallback::ThemePaddingSm,
            }),
            bottom: Some(crate::MetricRef::Token {
                key: "component.input.padding_y",
                fallback: MetricFallback::ThemePaddingSm,
            }),
            left: Some(crate::MetricRef::Token {
                key: "component.input.padding_x",
                fallback: MetricFallback::ThemePaddingSm,
            }),
        }),
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
        ..ChromeRefinement::default()
    }
}
