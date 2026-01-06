//! Built-in shadcn/ui theme presets.
//!
//! The upstream shadcn/ui project ships theme definitions as CSS variable sets (HSL/OKLCH).
//! Fret's runtime theme system is token-based (see `fret_ui::ThemeConfig`), so we convert those
//! CSS variables into `ThemeConfig` maps and rely on `Theme::apply_config` to parse them.

use std::collections::HashMap;

use fret_ui::{Theme, ThemeConfig, UiHost};
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

    let mut metrics: HashMap<String, f32> = HashMap::new();
    if let Some(radius) = colors.remove("radius") {
        if let Some(px) = parse_css_length_px(&radius) {
            metrics.insert("radius".to_string(), px);
        }
    }

    // new-york-v4 component defaults are expressed as Tailwind classes in the upstream registry
    // (e.g. `h-9`, `px-3`, `gap-2`, `focus-visible:ring-[3px]`). Our component library consumes
    // theme metrics, so we seed those defaults here to reduce per-component drift.
    //
    // Note: keep this small and scoped to ergonomic, high-signal tokens; component-specific
    // deviations should live in the component implementations and audit doc.
    metrics.entry("fret.padding.sm".to_string()).or_insert(8.0);
    metrics.entry("fret.padding.md".to_string()).or_insert(10.0);
    metrics.entry("font.size".to_string()).or_insert(14.0);
    metrics
        .entry("font.line_height".to_string())
        .or_insert(20.0);

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

    if let Some(ring) = colors.get("ring").cloned() {
        if let Some(ring_50) = with_oklch_alpha(&ring, 0.5) {
            colors.insert("ring/50".to_string(), ring_50);
        }
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

    ThemeConfig {
        name: format!("shadcn/new-york-v4/{}/{}", base.as_str(), scheme.as_str()),
        author: Some("shadcn/ui".to_string()),
        url: Some("https://ui.shadcn.com".to_string()),
        colors,
        metrics,
    }
}

/// Apply a shadcn preset into the global `Theme`.
pub fn apply_shadcn_new_york_v4<H: UiHost>(
    app: &mut H,
    base: ShadcnBaseColor,
    scheme: ShadcnColorScheme,
) {
    let cfg = shadcn_new_york_v4_config(base, scheme);
    Theme::global_mut(app).apply_config(&cfg);
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

    // `oklch(L C H)` -> `oklch(L C H / XX%)`
    // `oklch(L C H / YY%)` -> `oklch(L C H / XX%)`
    let inner = inner.split('/').next()?.trim();
    let pct = (alpha * 100.0).round().clamp(0.0, 100.0) as u32;
    Some(format!("oklch({inner} / {pct}%)"))
}
