use fret_core::{Color, Px};
use fret_ui::Theme;

use crate::{ChromeRefinement, ColorRef, MetricRef};

#[derive(Debug, Clone, Copy)]
pub struct ControlTokenKeys {
    pub padding_x: Option<&'static str>,
    pub padding_y: Option<&'static str>,
    pub min_height: Option<&'static str>,
    pub radius: Option<&'static str>,
    pub border_width: Option<&'static str>,
    pub background: Option<&'static str>,
    pub border_color: Option<&'static str>,
    pub text_color: Option<&'static str>,
    pub text_px: Option<&'static str>,
}

#[derive(Debug, Clone, Copy)]
pub struct ControlFallbacks {
    pub padding_x: Px,
    pub padding_y: Px,
    pub min_height: Px,
    pub radius: Px,
    pub border_width: Px,
    pub background: Color,
    pub border_color: Color,
    pub text_color: Color,
    pub text_px: Px,
}

#[derive(Debug, Clone, Copy)]
pub struct ResolvedControlChrome {
    pub padding_x: Px,
    pub padding_y: Px,
    pub min_height: Px,
    pub radius: Px,
    pub border_width: Px,
    pub background: Color,
    pub border_color: Color,
    pub text_color: Color,
    pub text_px: Px,
}

fn resolve_metric(
    theme: &Theme,
    style: Option<&MetricRef>,
    key: Option<&'static str>,
    fallback: Px,
) -> Px {
    let v = style
        .map(|m| m.resolve(theme))
        .or_else(|| key.and_then(|k| theme.metric_by_key(k)))
        .unwrap_or(fallback);
    Px(v.0.max(0.0))
}

fn resolve_color(
    theme: &Theme,
    style: Option<&ColorRef>,
    key: Option<&'static str>,
    fallback: Color,
) -> Color {
    style
        .map(|c| c.resolve(theme))
        .or_else(|| key.and_then(|k| theme.color_by_key(k)))
        .unwrap_or(fallback)
}

pub fn resolve_control_chrome(
    theme: &Theme,
    style: &ChromeRefinement,
    keys: ControlTokenKeys,
    fallback: ControlFallbacks,
) -> ResolvedControlChrome {
    ResolvedControlChrome {
        padding_x: resolve_metric(
            theme,
            style
                .padding
                .as_ref()
                .and_then(|p| p.left.as_ref().or(p.right.as_ref())),
            keys.padding_x,
            fallback.padding_x,
        ),
        padding_y: resolve_metric(
            theme,
            style
                .padding
                .as_ref()
                .and_then(|p| p.top.as_ref().or(p.bottom.as_ref())),
            keys.padding_y,
            fallback.padding_y,
        ),
        min_height: resolve_metric(
            theme,
            style.min_height.as_ref(),
            keys.min_height,
            fallback.min_height,
        ),
        radius: resolve_metric(theme, style.radius.as_ref(), keys.radius, fallback.radius),
        border_width: resolve_metric(
            theme,
            style.border_width.as_ref(),
            keys.border_width,
            fallback.border_width,
        ),
        background: resolve_color(
            theme,
            style.background.as_ref(),
            keys.background,
            fallback.background,
        ),
        border_color: resolve_color(
            theme,
            style.border_color.as_ref(),
            keys.border_color,
            fallback.border_color,
        ),
        text_color: resolve_color(
            theme,
            style.text_color.as_ref(),
            keys.text_color,
            fallback.text_color,
        ),
        text_px: resolve_metric(theme, None, keys.text_px, fallback.text_px),
    }
}
