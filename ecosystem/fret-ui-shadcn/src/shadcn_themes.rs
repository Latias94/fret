//! Built-in shadcn/ui theme presets.
//!
//! The upstream shadcn/ui project ships theme definitions as CSS variable sets (HSL/OKLCH).
//! Fret's runtime theme system is token-based (see `fret_ui::ThemeConfig`), so we convert those
//! CSS variables into `ThemeConfig` maps and rely on `Theme::apply_config` to parse/alias them.

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
            // Match shadcn's default border-radius recipe:
            // lg = var(--radius), md = var(--radius) - 2px, sm = var(--radius) - 4px.
            metrics.insert("metric.radius.lg".to_string(), px);
            metrics.insert("metric.radius.md".to_string(), (px - 2.0).max(0.0));
            metrics.insert("metric.radius.sm".to_string(), (px - 4.0).max(0.0));
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
