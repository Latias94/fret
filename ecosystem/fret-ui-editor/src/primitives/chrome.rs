//! Shared chrome resolution helpers for editor controls.
//!
//! v1 goal: keep "frame" defaults consistent across controls (inputs, triggers, scrub surfaces)
//! without hard-binding `fret-ui-editor` to a specific design system crate.

use fret_core::{Color, Corners, Edges, Px, TextStyle};
use fret_ui::element::{RingPlacement, RingStyle};
use fret_ui::{TextAreaStyle, TextInputStyle, Theme};
use fret_ui_kit::recipes::input::ResolvedInputChrome;
use fret_ui_kit::typography;
use fret_ui_kit::{ChromeRefinement, Size};

use super::EditorTokenKeys;
use super::colors::editor_muted_foreground;

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

fn is_effectively_transparent(c: Color) -> bool {
    c.a <= 0.02
}

fn opaque_over(theme: &Theme, fg: Color) -> Color {
    // Approximate the effective color a translucent surface would produce when rendered over the
    // theme background, then make it opaque so cached layers don't leak stale pixels.
    let bg = theme.color_token("background");
    let t = fg.a;
    let mut out = mix(bg, fg, t);
    out.a = 1.0;
    out
}

fn editor_fallback_input_bg(theme: &Theme) -> Color {
    // Shadcn themes sometimes set `component.input.bg` to fully transparent. For editor controls we
    // need a stable, non-transparent surface so frames are visible and we don't expose stale
    // pixels from cached overlay layers.
    let bg = theme.color_token("background");
    let muted = theme
        .color_by_key("muted")
        .unwrap_or_else(|| theme.color_token("muted"));
    let mut out = mix(bg, muted, 0.10);
    out.a = 1.0;
    out
}

pub(crate) fn sanitize_editor_surface_bg(theme: &Theme, bg: Color) -> Color {
    if is_effectively_transparent(bg) {
        return editor_fallback_input_bg(theme);
    }

    // Even when not fully transparent (e.g. shadcn `bg-input/30`), keep editor input surfaces
    // opaque to reduce ghosting/artifacts under paint caching and overlay reuse.
    if bg.a < 0.98 {
        return opaque_over(theme, bg);
    }

    bg
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ResolvedEditorFrameChrome {
    pub(crate) padding: Edges,
    pub(crate) radius: Px,
    pub(crate) border_width: Px,
    pub(crate) bg: Color,
    pub(crate) border: Color,
    pub(crate) border_focus: Color,
    pub(crate) fg: Color,
    pub(crate) text_px: Px,
}

fn editor_text_field_metric(theme: &Theme, editor_key: &str, legacy_key: &str) -> Option<Px> {
    theme
        .metric_by_key(editor_key)
        .or_else(|| theme.metric_by_key(legacy_key))
}

fn editor_text_field_color(theme: &Theme, editor_key: &str, legacy_key: &str) -> Option<Color> {
    theme
        .color_by_key(editor_key)
        .or_else(|| theme.color_by_key(legacy_key))
}

fn resolve_editor_text_field_input_chrome(
    theme: &Theme,
    size: Size,
    style: &ChromeRefinement,
) -> ResolvedInputChrome {
    let padding_x = editor_text_field_metric(
        theme,
        EditorTokenKeys::TEXT_FIELD_PADDING_X,
        "component.text_field.padding_x",
    )
    .or_else(|| theme.metric_by_key("component.input.padding_x"))
    .unwrap_or_else(|| size.input_px(theme));
    let padding_y = editor_text_field_metric(
        theme,
        EditorTokenKeys::TEXT_FIELD_PADDING_Y,
        "component.text_field.padding_y",
    )
    .or_else(|| theme.metric_by_key("component.input.padding_y"))
    .unwrap_or_else(|| size.input_py(theme));
    let min_height = style
        .min_height
        .as_ref()
        .map(|m| m.resolve(theme))
        .or_else(|| {
            editor_text_field_metric(
                theme,
                EditorTokenKeys::TEXT_FIELD_MIN_HEIGHT,
                "component.text_field.min_height",
            )
        })
        .or_else(|| theme.metric_by_key("component.input.min_height"))
        .unwrap_or_else(|| size.input_h(theme));
    let radius = style
        .radius
        .as_ref()
        .map(|m| m.resolve(theme))
        .or_else(|| {
            editor_text_field_metric(
                theme,
                EditorTokenKeys::TEXT_FIELD_RADIUS,
                "component.text_field.radius",
            )
        })
        .or_else(|| theme.metric_by_key("component.input.radius"))
        .unwrap_or_else(|| size.control_radius(theme));
    let border_width = style
        .border_width
        .as_ref()
        .map(|m| m.resolve(theme))
        .or_else(|| {
            editor_text_field_metric(
                theme,
                EditorTokenKeys::TEXT_FIELD_BORDER_WIDTH,
                "component.text_field.border_width",
            )
        })
        .or_else(|| theme.metric_by_key("component.input.border_width"))
        .unwrap_or(Px(1.0));

    let background = style
        .background
        .as_ref()
        .map(|c| c.resolve(theme))
        .or_else(|| {
            editor_text_field_color(
                theme,
                EditorTokenKeys::TEXT_FIELD_BG,
                "component.text_field.bg",
            )
        })
        .or_else(|| theme.color_by_key("component.input.bg"))
        .unwrap_or_else(|| theme.color_token("background"));
    let border_color = style
        .border_color
        .as_ref()
        .map(|c| c.resolve(theme))
        .or_else(|| {
            editor_text_field_color(
                theme,
                EditorTokenKeys::TEXT_FIELD_BORDER,
                "component.text_field.border",
            )
        })
        .or_else(|| theme.color_by_key("component.input.border"))
        .unwrap_or_else(|| theme.color_token("input"));
    let border_color_focused = editor_text_field_color(
        theme,
        EditorTokenKeys::TEXT_FIELD_BORDER_FOCUS,
        "component.text_field.border_focus",
    )
    .or_else(|| theme.color_by_key("component.input.border_focus"))
    .unwrap_or_else(|| theme.color_token("ring"));
    let text_color = style
        .text_color
        .as_ref()
        .map(|c| c.resolve(theme))
        .or_else(|| {
            editor_text_field_color(
                theme,
                EditorTokenKeys::TEXT_FIELD_FG,
                "component.text_field.fg",
            )
        })
        .or_else(|| theme.color_by_key("component.input.fg"))
        .unwrap_or_else(|| theme.color_token("foreground"));
    let text_px = editor_text_field_metric(
        theme,
        EditorTokenKeys::TEXT_FIELD_TEXT_PX,
        "component.text_field.text_px",
    )
    .or_else(|| theme.metric_by_key("component.input.text_px"))
    .unwrap_or_else(|| size.control_text_px(theme));
    let selection_color = editor_text_field_color(
        theme,
        EditorTokenKeys::TEXT_FIELD_SELECTION,
        "component.text_field.selection",
    )
    .or_else(|| theme.color_by_key("component.input.selection"))
    .unwrap_or_else(|| theme.color_token("selection.background"));

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

pub(crate) fn resolve_editor_text_field_frame_chrome(
    theme: &Theme,
    size: Size,
    refinement: &ChromeRefinement,
) -> ResolvedEditorFrameChrome {
    let resolved = resolve_editor_text_field_input_chrome(theme, size, refinement);
    ResolvedEditorFrameChrome {
        padding: resolved.padding,
        radius: resolved.radius,
        border_width: resolved.border_width,
        bg: sanitize_editor_surface_bg(theme, resolved.background),
        border: resolved.border_color,
        border_focus: resolved.border_color_focused,
        fg: resolved.text_color,
        text_px: resolved.text_px,
    }
}

pub(crate) fn joined_text_input_style(mut chrome: TextInputStyle) -> TextInputStyle {
    chrome.padding = Edges::all(Px(0.0));
    chrome.border = Edges::all(Px(0.0));
    chrome.corner_radii = Corners::all(Px(0.0));
    chrome.background = Color {
        a: 0.0,
        ..chrome.background
    };
    chrome.border_color = Color {
        a: 0.0,
        ..chrome.border_color
    };
    chrome.border_color_focused = chrome.border_color;
    chrome.focus_ring = None;
    chrome
}

pub(crate) fn joined_text_area_style(mut chrome: TextAreaStyle) -> TextAreaStyle {
    chrome.padding_x = Px(0.0);
    chrome.padding_y = Px(0.0);
    chrome.border = Edges::all(Px(0.0));
    chrome.corner_radii = Corners::all(Px(0.0));
    chrome.background = Color {
        a: 0.0,
        ..chrome.background
    };
    chrome.border_color = Color {
        a: 0.0,
        ..chrome.border_color
    };
    chrome.focus_ring = None;
    chrome
}

pub(crate) fn resolve_editor_text_field_style(
    theme: &Theme,
    size: Size,
    refinement: &ChromeRefinement,
) -> (TextInputStyle, TextStyle) {
    let resolved = resolve_editor_text_field_input_chrome(theme, size, refinement);

    let mut chrome = TextInputStyle::from_theme(theme.snapshot());
    chrome.padding = resolved.padding;
    chrome.corner_radii = Corners::all(resolved.radius);
    chrome.border = Edges::all(resolved.border_width);
    chrome.background = sanitize_editor_surface_bg(theme, resolved.background);
    chrome.border_color = resolved.border_color;
    chrome.border_color_focused = resolved.border_color_focused;
    chrome.text_color = resolved.text_color;
    chrome.caret_color = resolved.text_color;
    chrome.selection_color = resolved.selection_color;

    let font_line_height = theme
        .metric_by_key("font.line_height")
        .unwrap_or_else(|| theme.metric_token("font.line_height"));
    let text_style = typography::as_control_text(TextStyle {
        size: resolved.text_px,
        line_height: Some(font_line_height),
        ..Default::default()
    });

    (chrome, text_style)
}

pub(crate) fn resolve_editor_text_area_field_style(
    theme: &Theme,
    size: Size,
    refinement: &ChromeRefinement,
) -> (TextAreaStyle, TextStyle) {
    let resolved = resolve_editor_text_field_input_chrome(theme, size, refinement);
    let ring_color = theme
        .color_by_key("ring")
        .unwrap_or_else(|| theme.color_token("primary"));

    let font_line_height = theme
        .metric_by_key("font.line_height")
        .unwrap_or_else(|| theme.metric_token("font.line_height"));
    let text_style = typography::as_content_text(TextStyle {
        size: resolved.text_px,
        line_height: Some(font_line_height),
        ..Default::default()
    });

    let chrome = TextAreaStyle {
        padding_x: resolved.padding.left,
        padding_y: resolved.padding.top,
        background: sanitize_editor_surface_bg(theme, resolved.background),
        border: Edges::all(resolved.border_width),
        border_color: resolved.border_color,
        border_color_focused: resolved.border_color_focused,
        focus_ring: Some(RingStyle {
            placement: RingPlacement::Outset,
            width: Px(2.0),
            offset: Px(2.0),
            color: ring_color,
            offset_color: None,
            corner_radii: Corners::all(resolved.radius),
        }),
        corner_radii: Corners::all(resolved.radius),
        text_color: resolved.text_color,
        placeholder_color: editor_muted_foreground(theme),
        selection_color: resolved.selection_color,
        caret_color: resolved.text_color,
        preedit_bg_color: Color {
            a: 0.22,
            ..resolved.selection_color
        },
        preedit_underline_color: theme.color_token("primary"),
    };

    (chrome, text_style)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::{TextLineHeightPolicy, TextVerticalPlacement};
    use fret_ui::ThemeConfig;

    #[test]
    fn editor_text_field_style_uses_control_intent_defaults() {
        let app = App::new();
        let theme = Theme::global(&app);
        let (_chrome, style) =
            resolve_editor_text_field_style(theme, Size::Small, &ChromeRefinement::default());

        assert!(style.line_height.is_some());
        assert_eq!(
            style.line_height_policy,
            TextLineHeightPolicy::FixedFromStyle
        );
        assert_eq!(
            style.vertical_placement,
            TextVerticalPlacement::BoundsAsLineBox
        );
    }

    #[test]
    fn editor_text_area_style_uses_content_intent_defaults() {
        let app = App::new();
        let theme = Theme::global(&app);
        let (_chrome, style) =
            resolve_editor_text_area_field_style(theme, Size::Small, &ChromeRefinement::default());

        assert!(style.line_height.is_some());
        assert_eq!(style.line_height_policy, TextLineHeightPolicy::ExpandToFit);
        assert_eq!(
            style.vertical_placement,
            TextVerticalPlacement::CenterMetricsBox
        );
    }

    #[test]
    fn editor_text_field_style_prefers_editor_tokens_over_legacy_component_tokens() {
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| {
            let mut cfg = ThemeConfig::default();
            cfg.colors.insert(
                EditorTokenKeys::TEXT_FIELD_BG.to_string(),
                "#141b24".to_string(),
            );
            cfg.colors
                .insert("component.text_field.bg".to_string(), "#ffffff".to_string());
            theme.apply_config_patch(&cfg);
        });

        let theme = Theme::global(&app);
        let (chrome, _style) =
            resolve_editor_text_field_style(theme, Size::Small, &ChromeRefinement::default());

        assert_eq!(chrome.background, Color::from_srgb_hex_rgb(0x14_1b_24));
    }

    #[test]
    fn editor_text_field_style_keeps_legacy_component_text_field_fallback() {
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| {
            let mut cfg = ThemeConfig::default();
            cfg.colors
                .insert("component.text_field.bg".to_string(), "#141b24".to_string());
            cfg.metrics
                .insert("component.text_field.min_height".to_string(), 29.0);
            theme.apply_config_patch(&cfg);
        });

        let theme = Theme::global(&app);
        let (chrome, _style) =
            resolve_editor_text_field_style(theme, Size::Small, &ChromeRefinement::default());

        assert_eq!(chrome.background, Color::from_srgb_hex_rgb(0x14_1b_24));
        assert_eq!(chrome.padding.top, Size::Small.input_py(theme));
        assert_eq!(chrome.border, Edges::all(Px(1.0)));
    }
}
