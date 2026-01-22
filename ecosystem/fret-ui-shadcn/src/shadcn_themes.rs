//! Built-in shadcn/ui theme presets.
//!
//! The upstream shadcn/ui project ships theme definitions as CSS variable sets (HSL/OKLCH).
//! Fret's runtime theme system is token-based (see `fret_ui::ThemeConfig`), so we convert those
//! CSS variables into `ThemeConfig` maps and rely on `Theme::apply_config` to parse/alias them.

use std::collections::HashMap;

use fret_ui::{Theme, ThemeConfig, UiHost};
use fret_ui_kit::theme_tokens;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadcnColorScheme {
    Light,
    Dark,
}

impl ShadcnColorScheme {
    pub fn as_str(self) -> &'static str {
        match self {
            ShadcnColorScheme::Light => "light",
            ShadcnColorScheme::Dark => "dark",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadcnBaseColor {
    Neutral,
    Zinc,
    Slate,
    Stone,
    Gray,
}

impl ShadcnBaseColor {
    pub const ALL: &'static [ShadcnBaseColor] = &[
        ShadcnBaseColor::Neutral,
        ShadcnBaseColor::Zinc,
        ShadcnBaseColor::Slate,
        ShadcnBaseColor::Stone,
        ShadcnBaseColor::Gray,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            ShadcnBaseColor::Neutral => "neutral",
            ShadcnBaseColor::Zinc => "zinc",
            ShadcnBaseColor::Slate => "slate",
            ShadcnBaseColor::Stone => "stone",
            ShadcnBaseColor::Gray => "gray",
        }
    }
}

#[derive(Debug, Deserialize)]
struct ShadcnRegistryTheme {
    #[serde(rename = "cssVars")]
    css_vars: ShadcnCssVars,
}

#[derive(Debug, Deserialize)]
struct ShadcnCssVars {
    light: HashMap<String, String>,
    dark: HashMap<String, String>,
}

/// Load a shadcn v4 "new-york-v4" theme preset.
///
/// Theme source: `repo-ref/ui/apps/v4/public/r/styles/new-york-v4/theme-*.json` (vendored).
pub fn shadcn_new_york_v4_config(base: ShadcnBaseColor, scheme: ShadcnColorScheme) -> ThemeConfig {
    let raw = match base {
        ShadcnBaseColor::Neutral => {
            include_str!("../assets/shadcn/themes/new-york-v4/theme-neutral.json")
        }
        ShadcnBaseColor::Zinc => {
            include_str!("../assets/shadcn/themes/new-york-v4/theme-zinc.json")
        }
        ShadcnBaseColor::Slate => {
            include_str!("../assets/shadcn/themes/new-york-v4/theme-slate.json")
        }
        ShadcnBaseColor::Stone => {
            include_str!("../assets/shadcn/themes/new-york-v4/theme-stone.json")
        }
        ShadcnBaseColor::Gray => {
            include_str!("../assets/shadcn/themes/new-york-v4/theme-gray.json")
        }
    };

    let theme: ShadcnRegistryTheme =
        serde_json::from_str(raw).expect("vendored shadcn theme JSON is valid");

    let mut colors = match scheme {
        ShadcnColorScheme::Light => theme.css_vars.light,
        ShadcnColorScheme::Dark => theme.css_vars.dark,
    };

    // The upstream shadcn registry v4 theme JSONs focus on base palette tokens.
    // Some `*-foreground` variables are still relied upon by component recipes (e.g. Button,
    // Badge). Seed missing ones to match the upstream CSS defaults.
    colors
        .entry("destructive-foreground".to_string())
        .or_insert_with(|| match scheme {
            // Source: `repo-ref/ui/apps/v4/styles/globals.css`.
            ShadcnColorScheme::Light => "oklch(0.97 0.01 17)".to_string(),
            ShadcnColorScheme::Dark => "oklch(0.58 0.22 27)".to_string(),
        });

    let mut metrics: HashMap<String, f32> = HashMap::new();
    if let Some(radius) = colors.remove("radius") {
        if let Some(px) = parse_css_length_px(&radius) {
            // Match shadcn's default border-radius recipe:
            // lg = var(--radius), md = var(--radius) - 2px, sm = var(--radius) - 4px.
            metrics.insert("metric.radius.lg".to_string(), px);
            metrics.insert("metric.radius.md".to_string(), (px - 2.0).max(0.0));
            metrics.insert("metric.radius.sm".to_string(), (px - 4.0).max(0.0));
        }
    }

    // new-york-v4 component defaults are expressed as Tailwind classes in the upstream registry
    // (e.g. `h-9`, `px-3`, `gap-2`, `focus-visible:ring-[3px]`). Our component library consumes
    // theme metrics, so we seed those defaults here to reduce per-component drift.
    //
    // Note: keep this small and scoped to ergonomic, high-signal tokens; component-specific
    // deviations should live in the component implementations and audit doc.
    metrics
        .entry("metric.padding.sm".to_string())
        .or_insert(8.0);
    metrics
        .entry("metric.padding.md".to_string())
        .or_insert(10.0);

    // Component-specific overrides in the upstream registry.
    // - Checkbox uses `rounded-[4px]` (not `rounded-sm`, which would be `radius - 4px`).
    metrics
        .entry("component.checkbox.radius".to_string())
        .or_insert(4.0);
    metrics
        .entry("metric.font.size".to_string())
        .or_insert(14.0);
    metrics
        .entry("metric.font.line_height".to_string())
        .or_insert(20.0);

    // Default typography scales used across shadcn recipes (via fret-ui-kit helpers).
    // These are also accessed directly by some components (e.g. Calendar) via `metric_required`.
    metrics
        .entry(theme_tokens::metric::COMPONENT_TEXT_SM_PX.to_string())
        .or_insert(14.0);
    metrics
        .entry(theme_tokens::metric::COMPONENT_TEXT_SM_LINE_HEIGHT.to_string())
        .or_insert(20.0);
    metrics
        .entry(theme_tokens::metric::COMPONENT_TEXT_BASE_PX.to_string())
        .or_insert(15.0);
    metrics
        .entry(theme_tokens::metric::COMPONENT_TEXT_BASE_LINE_HEIGHT.to_string())
        .or_insert(20.0);
    metrics
        .entry(theme_tokens::metric::COMPONENT_TEXT_PROSE_PX.to_string())
        .or_insert(16.0);
    metrics
        .entry(theme_tokens::metric::COMPONENT_TEXT_PROSE_LINE_HEIGHT.to_string())
        .or_insert(24.0);

    metrics
        .entry("component.ring.width".to_string())
        .or_insert(3.0);
    metrics
        .entry("component.ring.offset".to_string())
        .or_insert(0.0);

    metrics
        .entry("component.size.md.input.h".to_string())
        .or_insert(36.0);
    metrics
        .entry("component.size.md.input.px".to_string())
        .or_insert(12.0);
    metrics
        .entry("component.size.md.input.py".to_string())
        .or_insert(4.0);

    metrics
        .entry("component.size.md.button.h".to_string())
        .or_insert(36.0);
    metrics
        .entry("component.size.sm.button.h".to_string())
        .or_insert(32.0);
    metrics
        .entry("component.size.lg.button.h".to_string())
        .or_insert(40.0);
    metrics
        .entry("component.size.md.icon_button.size".to_string())
        .or_insert(36.0);
    metrics
        .entry("component.size.sm.icon_button.size".to_string())
        .or_insert(32.0);
    metrics
        .entry("component.size.lg.icon_button.size".to_string())
        .or_insert(40.0);

    // Legacy generic size tokens used by some components/tests.
    // Prefer `component.size.*` tokens in new code.
    metrics.entry("metric.size.sm".to_string()).or_insert(32.0);
    metrics.entry("metric.size.md".to_string()).or_insert(36.0);
    metrics.entry("metric.size.lg".to_string()).or_insert(40.0);

    // new-york-v4 `Slider` defaults:
    // - Track uses `h-1.5` (6px) via `data-[orientation=horizontal]:h-1.5`.
    // - Thumb uses `size-4` (16px).
    metrics
        .entry("component.slider.track_height".to_string())
        .or_insert(6.0);
    metrics
        .entry("component.slider.thumb_size".to_string())
        .or_insert(16.0);

    // new-york-v4 `Badge` defaults:
    // - `text-xs` (12px) with Tailwind default leading (16px).
    metrics
        .entry("component.badge.text_px".to_string())
        .or_insert(12.0);
    metrics
        .entry("component.badge.line_height".to_string())
        .or_insert(16.0);

    // new-york-v4 `Label` defaults:
    // - `text-sm` (14px) and `leading-none` (line-height = font-size).
    metrics
        .entry("component.label.text_px".to_string())
        .or_insert(14.0);
    metrics
        .entry("component.label.line_height".to_string())
        .or_insert(14.0);

    // new-york-v4 `Field` defaults:
    // - `FieldGroup` uses `gap-7` (28px).
    // - `FieldLabel` uses `text-sm` with `leading-snug` (14px * 1.375 = 19.25px).
    // - `FieldDescription` uses `text-sm` with `leading-normal` (14px * 1.5 = 21px).
    metrics
        .entry("component.field.group_gap".to_string())
        .or_insert(28.0);
    metrics
        .entry("component.field.label_px".to_string())
        .or_insert(14.0);
    metrics
        .entry("component.field.label_line_height".to_string())
        .or_insert(19.25);
    metrics
        .entry("component.field.description_px".to_string())
        .or_insert(14.0);
    metrics
        .entry("component.field.description_line_height".to_string())
        .or_insert(21.0);

    // Tooltip defaults in the upstream registry:
    // - `sideOffset={4}`
    // - Arrow uses `h-2 w-2` (8px)
    metrics
        .entry("component.tooltip.side_offset".to_string())
        .or_insert(4.0);
    metrics
        .entry("component.tooltip.arrow_size".to_string())
        .or_insert(8.0);

    if let Some(ring) = colors.get("ring").cloned() {
        if let Some(ring_50) = with_oklch_alpha(&ring, 0.5) {
            colors.insert("ring/50".to_string(), ring_50);
        }
    }
    if let Some(destructive) = colors.get("destructive").cloned() {
        if let Some(v) = with_oklch_alpha(&destructive, 0.1) {
            colors.insert("destructive/10".to_string(), v);
        }
        if let Some(v) = with_oklch_alpha(&destructive, 0.2) {
            colors.insert("destructive/20".to_string(), v);
        }
        if let Some(v) = with_oklch_alpha(&destructive, 0.4) {
            colors.insert("destructive/40".to_string(), v);
        }
    }

    // new-york-v4 `ScrollArea` uses `bg-border` for the thumb.
    if let Some(border) = colors.get("border").cloned() {
        colors.insert("scrollbar.thumb.background".to_string(), border.clone());
        colors.insert("scrollbar.thumb.hover.background".to_string(), border);
        colors
            .entry("scrollbar.background".to_string())
            .or_insert("#00000000".to_string());
    }
    match scheme {
        ShadcnColorScheme::Light => {
            // `bg-transparent` for inputs in light mode.
            colors.insert("component.input.bg".to_string(), "#00000000".to_string());
        }
        ShadcnColorScheme::Dark => {
            // `dark:bg-input/30` in the upstream Input component.
            if let Some(input) = colors.get("input").cloned() {
                colors.insert(
                    "component.input.bg".to_string(),
                    with_oklch_alpha(&input, 0.3).unwrap_or(input),
                );
            }
        }
    }

    seed_syntax_colors(&mut colors);

    ThemeConfig {
        name: format!("shadcn/new-york-v4/{}/{}", base.as_str(), scheme.as_str()),
        author: Some("shadcn/ui".to_string()),
        url: Some("https://ui.shadcn.com".to_string()),
        colors,
        metrics,
        ..Default::default()
    }
}

fn seed_syntax_colors(colors: &mut HashMap<String, String>) {
    fn pick(colors: &HashMap<String, String>, keys: &[&str]) -> Option<String> {
        for &key in keys {
            if let Some(value) = colors.get(key) {
                return Some(value.clone());
            }
        }
        None
    }

    fn insert_if_missing(colors: &mut HashMap<String, String>, key: &str, value: Option<String>) {
        if colors.contains_key(key) {
            return;
        }
        if let Some(value) = value {
            colors.insert(key.to_string(), value);
        }
    }

    // These keys are consumed by `fret-code-view` for tree-sitter highlight tags (ADR 0100).
    // We derive a small palette from the base shadcn `chart-*` tokens.
    insert_if_missing(
        colors,
        "color.syntax.comment",
        pick(colors, &["muted-foreground"]),
    );
    insert_if_missing(
        colors,
        "color.syntax.keyword",
        pick(colors, &["chart-3", "primary"]),
    );
    insert_if_missing(
        colors,
        "color.syntax.function",
        pick(colors, &["chart-1", "primary"]),
    );
    insert_if_missing(
        colors,
        "color.syntax.type",
        pick(colors, &["chart-4", "accent-foreground", "foreground"]),
    );
    insert_if_missing(
        colors,
        "color.syntax.string",
        pick(colors, &["chart-2", "foreground"]),
    );
    insert_if_missing(
        colors,
        "color.syntax.constant",
        pick(colors, &["chart-5", "primary"]),
    );
    insert_if_missing(
        colors,
        "color.syntax.number",
        pick(colors, &["chart-5", "primary"]),
    );
    insert_if_missing(
        colors,
        "color.syntax.operator",
        pick(colors, &["muted-foreground", "foreground"]),
    );
    insert_if_missing(
        colors,
        "color.syntax.punctuation",
        pick(colors, &["muted-foreground", "foreground"]),
    );
    insert_if_missing(
        colors,
        "color.syntax.variable",
        pick(colors, &["foreground"]),
    );
}

/// Apply a shadcn preset into the global `Theme`.
pub fn apply_shadcn_new_york_v4<H: UiHost>(
    app: &mut H,
    base: ShadcnBaseColor,
    scheme: ShadcnColorScheme,
) {
    let cfg = shadcn_new_york_v4_config(base, scheme);
    Theme::with_global_mut(app, |theme| theme.apply_config(&cfg));
}

fn parse_css_length_px(s: &str) -> Option<f32> {
    let s = s.trim();
    if let Some(rem) = s.strip_suffix("rem") {
        let v: f32 = rem.trim().parse().ok()?;
        return Some(v * 16.0);
    }
    if let Some(px) = s.strip_suffix("px") {
        let v: f32 = px.trim().parse().ok()?;
        return Some(v);
    }
    None
}

fn with_oklch_alpha(raw: &str, alpha: f32) -> Option<String> {
    let alpha = alpha.clamp(0.0, 1.0);
    let raw = raw.trim();
    let inner = raw.strip_prefix("oklch(")?.strip_suffix(')')?.trim();

    let (main, base_alpha) = if let Some((main, alpha_part)) = inner.split_once('/') {
        let alpha_part = alpha_part.trim();
        let base_alpha = if alpha_part.ends_with('%') {
            let pct: f32 = alpha_part.trim_end_matches('%').trim().parse().ok()?;
            (pct / 100.0).clamp(0.0, 1.0)
        } else {
            alpha_part.parse::<f32>().ok()?.clamp(0.0, 1.0)
        };
        (main.trim(), base_alpha)
    } else {
        (inner, 1.0)
    };

    // Tailwind-style opacity modifiers (e.g. `bg-input/30`) should multiply alpha when the base
    // token already includes one (shadcn v4 dark themes often do).
    let out_alpha = (base_alpha * alpha).clamp(0.0, 1.0);
    let pct = ((out_alpha * 1000.0).round() / 10.0).clamp(0.0, 100.0);
    if (pct.fract() - 0.0).abs() < f32::EPSILON {
        Some(format!("oklch({main} / {}%)", pct as u32))
    } else {
        Some(format!("oklch({main} / {pct:.1}%)"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_ui::Theme;

    #[test]
    fn new_york_v4_seeds_component_text_metrics() {
        for &base in ShadcnBaseColor::ALL {
            for scheme in [ShadcnColorScheme::Light, ShadcnColorScheme::Dark] {
                let cfg = shadcn_new_york_v4_config(base, scheme);
                assert!(
                    cfg.metrics
                        .contains_key(theme_tokens::metric::COMPONENT_TEXT_SM_PX)
                );
                assert!(
                    cfg.metrics
                        .contains_key(theme_tokens::metric::COMPONENT_TEXT_SM_LINE_HEIGHT)
                );
                assert!(
                    cfg.metrics
                        .contains_key(theme_tokens::metric::COMPONENT_TEXT_BASE_PX)
                );
                assert!(
                    cfg.metrics
                        .contains_key(theme_tokens::metric::COMPONENT_TEXT_BASE_LINE_HEIGHT)
                );
            }
        }
    }

    #[test]
    fn new_york_v4_seeds_control_sizing_metrics() {
        let cfg = shadcn_new_york_v4_config(ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);
        assert_eq!(
            cfg.metrics.get("component.size.md.button.h").copied(),
            Some(36.0)
        );
        assert_eq!(
            cfg.metrics.get("component.size.sm.button.h").copied(),
            Some(32.0)
        );
        assert_eq!(
            cfg.metrics.get("component.size.lg.button.h").copied(),
            Some(40.0)
        );
        assert_eq!(
            cfg.metrics
                .get("component.size.md.icon_button.size")
                .copied(),
            Some(36.0)
        );
        assert_eq!(
            cfg.metrics
                .get("component.size.sm.icon_button.size")
                .copied(),
            Some(32.0)
        );
        assert_eq!(
            cfg.metrics
                .get("component.size.lg.icon_button.size")
                .copied(),
            Some(40.0)
        );

        let mut app = fret_app::App::new();
        apply_shadcn_new_york_v4(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        let theme = Theme::global(&app);
        assert_eq!(
            theme.metric_by_key("component.size.md.icon_button.size"),
            Some(fret_core::Px(36.0))
        );
        assert_eq!(
            theme.metric_by_key("component.size.sm.button.h"),
            Some(fret_core::Px(32.0))
        );
    }
}
