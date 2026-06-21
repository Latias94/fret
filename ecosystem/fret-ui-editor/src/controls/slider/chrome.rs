use crate::primitives::EditorTokenKeys;
use crate::primitives::colors::{editor_accent, editor_border, editor_subtle_bg};
use fret_core::{Axis, Color, Corners, Edges, Px};
use fret_ui::Theme;
use fret_ui::element::{
    ContainerProps, CrossAlign, FlexItemStyle, FlexProps, LayoutStyle, Length, MainAlign,
    SizeStyle, SpacingLength,
};

#[cfg(test)]
mod tests;

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub(super) fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color {
        r: lerp(a.r, b.r, t),
        g: lerp(a.g, b.g, t),
        b: lerp(a.b, b.b, t),
        a: lerp(a.a, b.a, t),
    }
}

pub(super) fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ResolvedSliderGeometry {
    pub(super) track_h: Px,
    pub(super) thumb_d: Px,
    pub(super) track_radius: Px,
    pub(super) thumb_radius: Px,
}

pub(super) fn resolve_slider_geometry(theme: &Theme) -> ResolvedSliderGeometry {
    let track_h = theme
        .metric_by_key(EditorTokenKeys::SLIDER_TRACK_HEIGHT)
        .unwrap_or(Px(4.0));
    let thumb_d = theme
        .metric_by_key(EditorTokenKeys::SLIDER_THUMB_DIAMETER)
        .unwrap_or(Px(12.0));

    let track_h = Px(track_h.0.max(1.0));
    let thumb_d = Px(thumb_d.0.max(track_h.0));

    ResolvedSliderGeometry {
        track_h,
        thumb_d,
        track_radius: Px(track_h.0 * 0.5),
        thumb_radius: Px(thumb_d.0 * 0.5),
    }
}

pub(super) fn slider_track_flex_props(padding: Edges) -> FlexProps {
    FlexProps {
        layout: LayoutStyle {
            size: SizeStyle {
                width: Length::Fill,
                height: Length::Fill,
                ..Default::default()
            },
            flex: FlexItemStyle {
                order: 0,
                grow: 1.0,
                shrink: 1.0,
                basis: Length::Px(Px(0.0)),
                align_self: None,
            },
            ..Default::default()
        },
        direction: Axis::Horizontal,
        gap: SpacingLength::Px(Px(0.0)),
        padding: padding.into(),
        justify: MainAlign::Start,
        align: CrossAlign::Center,
        wrap: false,
    }
}

pub(super) fn slider_track_segment_props(
    geometry: ResolvedSliderGeometry,
    grow: f32,
    bg: Color,
    left: bool,
) -> ContainerProps {
    ContainerProps {
        layout: LayoutStyle {
            size: SizeStyle {
                width: Length::Auto,
                height: Length::Px(geometry.track_h),
                ..Default::default()
            },
            flex: FlexItemStyle {
                order: 0,
                grow,
                shrink: 1.0,
                basis: Length::Px(Px(0.0)),
                align_self: None,
            },
            ..Default::default()
        },
        background: Some(bg),
        corner_radii: if left {
            Corners {
                top_left: geometry.track_radius,
                bottom_left: geometry.track_radius,
                top_right: Px(0.0),
                bottom_right: Px(0.0),
            }
        } else {
            Corners {
                top_left: Px(0.0),
                bottom_left: Px(0.0),
                top_right: geometry.track_radius,
                bottom_right: geometry.track_radius,
            }
        },
        ..Default::default()
    }
}

pub(super) fn slider_thumb_props(
    geometry: ResolvedSliderGeometry,
    paint: ResolvedSliderPaint,
) -> ContainerProps {
    ContainerProps {
        layout: LayoutStyle {
            size: SizeStyle {
                width: Length::Px(geometry.thumb_d),
                height: Length::Px(geometry.thumb_d),
                ..Default::default()
            },
            flex: FlexItemStyle {
                order: 0,
                grow: 0.0,
                shrink: 0.0,
                basis: Length::Px(geometry.thumb_d),
                align_self: None,
            },
            ..Default::default()
        },
        background: Some(paint.thumb_bg),
        border: Edges::all(Px(1.0)),
        border_color: Some(paint.thumb_border),
        corner_radii: Corners::all(geometry.thumb_radius),
        ..Default::default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ResolvedSliderChrome {
    pub(super) track_bg: Color,
    pub(super) fill_bg: Color,
    pub(super) thumb_bg: Color,
    pub(super) thumb_border: Color,
}

pub(super) fn resolve_slider_chrome(theme: &Theme) -> ResolvedSliderChrome {
    let track_bg = theme
        .color_by_key(EditorTokenKeys::SLIDER_TRACK_BG)
        .or_else(|| theme.color_by_key("component.slider.track_bg"))
        .unwrap_or_else(|| editor_subtle_bg(theme));
    let fill_bg = theme
        .color_by_key(EditorTokenKeys::SLIDER_FILL_BG)
        .or_else(|| theme.color_by_key("component.slider.fill_bg"))
        .unwrap_or_else(|| editor_accent(theme));
    let thumb_bg = theme
        .color_by_key(EditorTokenKeys::SLIDER_THUMB_BG)
        .or_else(|| theme.color_by_key("component.slider.thumb_bg"))
        .unwrap_or_else(|| editor_subtle_bg(theme));
    let thumb_border = theme
        .color_by_key(EditorTokenKeys::SLIDER_THUMB_BORDER)
        .or_else(|| theme.color_by_key("component.slider.thumb_border"))
        .unwrap_or_else(|| editor_border(theme));

    ResolvedSliderChrome {
        track_bg,
        fill_bg,
        thumb_bg,
        thumb_border,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ResolvedSliderPaint {
    pub(super) track_bg: Color,
    pub(super) fill_bg: Color,
    pub(super) thumb_bg: Color,
    pub(super) thumb_border: Color,
}

pub(super) fn resolve_slider_paint(
    theme: &Theme,
    interactive_enabled: bool,
    enabled: bool,
    hovered: bool,
    pressed: bool,
) -> ResolvedSliderPaint {
    let accent = editor_accent(theme);
    let disabled_alpha = if interactive_enabled { 1.0 } else { 0.55 };
    let chrome = resolve_slider_chrome(theme);

    let mut track_bg = chrome.track_bg;
    let mut fill_bg = chrome.fill_bg;
    let thumb_bg = chrome.thumb_bg;
    let thumb_border = chrome.thumb_border;

    if hovered && enabled {
        track_bg = mix(track_bg, accent, 0.06);
        fill_bg = mix(fill_bg, accent, 0.04);
    }
    if pressed && enabled {
        track_bg = mix(track_bg, accent, 0.10);
        fill_bg = mix(fill_bg, accent, 0.08);
    }

    ResolvedSliderPaint {
        track_bg: alpha_mul(track_bg, disabled_alpha),
        fill_bg: alpha_mul(fill_bg, disabled_alpha),
        thumb_bg: alpha_mul(thumb_bg, disabled_alpha),
        thumb_border: alpha_mul(thumb_border, disabled_alpha),
    }
}
