use fret_core::{Color, Px};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::OnceLock};

use crate::UiHost;

fn default_color_tokens(colors: ThemeColors) -> HashMap<String, Color> {
    HashMap::from([
        (
            "color.surface.background".to_string(),
            colors.surface_background,
        ),
        (
            "color.panel.background".to_string(),
            colors.panel_background,
        ),
        ("color.panel.border".to_string(), colors.panel_border),
        ("color.text.primary".to_string(), colors.text_primary),
        ("color.text.muted".to_string(), colors.text_muted),
        ("color.text.disabled".to_string(), colors.text_disabled),
        ("color.accent".to_string(), colors.accent),
        (
            "color.selection.background".to_string(),
            colors.selection_background,
        ),
        (
            "color.hover.background".to_string(),
            colors.hover_background,
        ),
        ("color.focus.ring".to_string(), colors.focus_ring),
        ("color.menu.background".to_string(), colors.menu_background),
        ("color.menu.border".to_string(), colors.menu_border),
        ("color.menu.item.hover".to_string(), colors.menu_item_hover),
        (
            "color.menu.item.selected".to_string(),
            colors.menu_item_selected,
        ),
        ("color.list.background".to_string(), colors.list_background),
        ("color.list.border".to_string(), colors.list_border),
        ("color.list.row.hover".to_string(), colors.list_row_hover),
        (
            "color.list.row.selected".to_string(),
            colors.list_row_selected,
        ),
        ("color.scrollbar.track".to_string(), colors.scrollbar_track),
        ("color.scrollbar.thumb".to_string(), colors.scrollbar_thumb),
        (
            "color.scrollbar.thumb.hover".to_string(),
            colors.scrollbar_thumb_hover,
        ),
        (
            "color.viewport.selection.fill".to_string(),
            colors.viewport_selection_fill,
        ),
        (
            "color.viewport.selection.stroke".to_string(),
            colors.viewport_selection_stroke,
        ),
        ("color.viewport.marker".to_string(), colors.viewport_marker),
        (
            "color.viewport.drag_line.pan".to_string(),
            colors.viewport_drag_line_pan,
        ),
        (
            "color.viewport.drag_line.orbit".to_string(),
            colors.viewport_drag_line_orbit,
        ),
        (
            "color.viewport.gizmo.x".to_string(),
            colors.viewport_gizmo_x,
        ),
        (
            "color.viewport.gizmo.y".to_string(),
            colors.viewport_gizmo_y,
        ),
        (
            "color.viewport.gizmo.handle.background".to_string(),
            colors.viewport_gizmo_handle_background,
        ),
        (
            "color.viewport.gizmo.handle.border".to_string(),
            colors.viewport_gizmo_handle_border,
        ),
        (
            "color.viewport.rotate_gizmo".to_string(),
            colors.viewport_rotate_gizmo,
        ),
    ])
}

fn default_metric_tokens(metrics: ThemeMetrics) -> HashMap<String, Px> {
    HashMap::from([
        ("metric.radius.sm".to_string(), metrics.radius_sm),
        ("metric.radius.md".to_string(), metrics.radius_md),
        ("metric.radius.lg".to_string(), metrics.radius_lg),
        ("metric.padding.sm".to_string(), metrics.padding_sm),
        ("metric.padding.md".to_string(), metrics.padding_md),
        (
            "metric.scrollbar.width".to_string(),
            metrics.scrollbar_width,
        ),
        ("metric.font.size".to_string(), metrics.font_size),
        ("metric.font.mono_size".to_string(), metrics.mono_font_size),
        (
            "metric.font.line_height".to_string(),
            metrics.font_line_height,
        ),
        (
            "metric.font.mono_line_height".to_string(),
            metrics.mono_font_line_height,
        ),
    ])
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeConfig {
    pub name: String,
    pub author: Option<String>,
    pub url: Option<String>,
    pub colors: HashMap<String, String>,
    pub metrics: HashMap<String, f32>,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            author: None,
            url: None,
            colors: HashMap::new(),
            metrics: HashMap::new(),
        }
    }
}

impl ThemeConfig {
    pub fn from_slice(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ThemeMetrics {
    pub radius_sm: Px,
    pub radius_md: Px,
    pub radius_lg: Px,
    pub padding_sm: Px,
    pub padding_md: Px,
    pub scrollbar_width: Px,
    pub font_size: Px,
    pub mono_font_size: Px,
    pub font_line_height: Px,
    pub mono_font_line_height: Px,
}

#[derive(Debug, Clone, Copy)]
pub struct ThemeColors {
    pub surface_background: Color,
    pub panel_background: Color,
    pub panel_border: Color,

    pub text_primary: Color,
    pub text_muted: Color,
    pub text_disabled: Color,

    pub accent: Color,
    pub selection_background: Color,
    pub hover_background: Color,
    pub focus_ring: Color,

    pub menu_background: Color,
    pub menu_border: Color,
    pub menu_item_hover: Color,
    pub menu_item_selected: Color,

    pub list_background: Color,
    pub list_border: Color,
    pub list_row_hover: Color,
    pub list_row_selected: Color,

    pub scrollbar_track: Color,
    pub scrollbar_thumb: Color,
    pub scrollbar_thumb_hover: Color,

    pub viewport_selection_fill: Color,
    pub viewport_selection_stroke: Color,
    pub viewport_marker: Color,
    pub viewport_drag_line_pan: Color,
    pub viewport_drag_line_orbit: Color,
    pub viewport_gizmo_x: Color,
    pub viewport_gizmo_y: Color,
    pub viewport_gizmo_handle_background: Color,
    pub viewport_gizmo_handle_border: Color,
    pub viewport_rotate_gizmo: Color,
}

#[derive(Debug, Clone, Copy)]
pub struct ThemeSnapshot {
    pub colors: ThemeColors,
    pub metrics: ThemeMetrics,
    pub revision: u64,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub author: Option<String>,
    pub url: Option<String>,
    pub colors: ThemeColors,
    pub metrics: ThemeMetrics,
    extra_colors: HashMap<String, Color>,
    extra_metrics: HashMap<String, Px>,
    revision: u64,
}

impl Theme {
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn color_by_key(&self, key: &str) -> Option<Color> {
        self.extra_colors
            .get(key)
            .copied()
            .or_else(|| self.fallback_color_alias(key))
    }

    pub fn metric_by_key(&self, key: &str) -> Option<Px> {
        self.extra_metrics
            .get(key)
            .copied()
            .or_else(|| self.fallback_metric_alias(key))
    }

    /// Fallback aliases for gpui-component / shadcn-style semantic keys.
    ///
    /// This provides a compatibility bridge without forcing theme files or widgets to adopt a new
    /// schema in one refactor pass.
    fn fallback_color_alias(&self, key: &str) -> Option<Color> {
        match key {
            // Core shadcn-like semantic palette.
            "background" => Some(self.colors.surface_background),
            "foreground" => Some(self.colors.text_primary),
            "border" => Some(self.colors.panel_border),
            "ring" => Some(self.colors.focus_ring),
            "ring-offset-background" | "ring_offset_background" => {
                Some(self.colors.surface_background)
            }

            // Surfaces.
            "card" | "card.background" => Some(self.colors.panel_background),
            "card.foreground" | "card-foreground" => Some(self.colors.text_primary),
            "card.border" => Some(self.colors.panel_border),

            // Common state semantics.
            "selection.background" => Some(self.colors.selection_background),
            "muted" | "muted.background" => Some(self.colors.panel_background),
            "muted.foreground" | "muted-foreground" => Some(self.colors.text_muted),
            "accent" | "accent.background" => Some(self.colors.hover_background),
            "accent.foreground" | "accent-foreground" => Some(self.colors.text_primary),

            // Primary/secondary/destructive semantic palette (best-effort fallbacks).
            "primary" | "primary.background" => Some(self.colors.accent),
            "primary.foreground" | "primary-foreground" => Some(self.colors.text_primary),
            "secondary" | "secondary.background" => Some(self.colors.panel_background),
            "secondary.foreground" | "secondary-foreground" => Some(self.colors.text_primary),
            "destructive" | "destructive.background" => Some(self.colors.viewport_gizmo_x),
            "destructive.foreground" | "destructive-foreground" => Some(self.colors.text_primary),

            // Popovers/menus map well onto the existing menu surface tokens.
            "popover" | "popover.background" => Some(self.colors.menu_background),
            "popover.foreground" | "popover-foreground" => Some(self.colors.text_primary),
            "popover.border" => Some(self.colors.menu_border),

            // List semantics used heavily by gpui-component.
            "list.background" => Some(self.colors.list_background),
            "list.hover.background" => Some(self.colors.list_row_hover),
            "list.active.background" => Some(self.colors.list_row_selected),
            "list.active.border" => Some(self.colors.accent),

            // Inputs.
            "input" | "input.border" => Some(self.colors.panel_border),
            "input.background" => Some(self.colors.panel_background),
            "input.foreground" => Some(self.colors.text_primary),
            "caret" => Some(self.colors.text_primary),

            // Scrollbars.
            "scrollbar.background" => Some(self.colors.scrollbar_track),
            "scrollbar.thumb.background" => Some(self.colors.scrollbar_thumb),
            "scrollbar.thumb.hover.background" => Some(self.colors.scrollbar_thumb_hover),

            // Shadows/elevation. These are intentionally best-effort fallbacks until the theme
            // schema grows first-class shadow tokens.
            "shadow" | "shadow.color" => Some(Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }),

            _ => None,
        }
    }

    fn fallback_metric_alias(&self, key: &str) -> Option<Px> {
        match key {
            // gpui-component uses `radius` / `radius.lg` as generic theme knobs.
            "radius" => Some(self.metrics.radius_sm),
            "radius.lg" => Some(self.metrics.radius_md),
            "font.size" => Some(self.metrics.font_size),
            "mono_font.size" => Some(self.metrics.mono_font_size),
            "font.line_height" => Some(self.metrics.font_line_height),
            "mono_font.line_height" => Some(self.metrics.mono_font_line_height),
            _ => None,
        }
    }

    pub fn snapshot(&self) -> ThemeSnapshot {
        ThemeSnapshot {
            colors: self.colors,
            metrics: self.metrics,
            revision: self.revision,
        }
    }

    pub fn global<H: UiHost>(app: &H) -> &Theme {
        if let Some(theme) = app.global::<Theme>() {
            theme
        } else {
            default_theme()
        }
    }

    pub fn global_mut<H: UiHost>(app: &mut H) -> &mut Theme {
        if app.global::<Theme>().is_none() {
            app.set_global(default_theme().clone());
        }
        app.global_mut::<Theme>().expect("theme global exists")
    }

    pub fn with_global_mut<H: UiHost, R>(app: &mut H, f: impl FnOnce(&mut Theme) -> R) -> R {
        app.with_global_mut(|| default_theme().clone(), |theme, _app| f(theme))
    }

    pub fn apply_config(&mut self, cfg: &ThemeConfig) {
        self.name = cfg.name.clone();
        self.author = cfg.author.clone();
        self.url = cfg.url.clone();

        let mut changed = false;

        let mut next_colors = default_color_tokens(self.colors);
        let mut next_metrics = default_metric_tokens(self.metrics);

        macro_rules! apply_color {
            ($key:literal, $field:expr) => {
                if let Some(v) = cfg.colors.get($key) {
                    if let Some(c) = parse_color_to_linear(v) {
                        next_colors.insert($key.to_string(), c);
                        if $field != c {
                            $field = c;
                            changed = true;
                        }
                    }
                }
            };
        }

        macro_rules! apply_metric {
            ($key:literal, $field:expr) => {
                if let Some(v) = cfg.metrics.get($key).copied() {
                    let px = Px(v);
                    next_metrics.insert($key.to_string(), px);
                    if $field != px {
                        $field = px;
                        changed = true;
                    }
                }
            };
        }

        apply_color!("color.surface.background", self.colors.surface_background);
        apply_color!("color.panel.background", self.colors.panel_background);
        apply_color!("color.panel.border", self.colors.panel_border);

        apply_color!("color.text.primary", self.colors.text_primary);
        apply_color!("color.text.muted", self.colors.text_muted);
        apply_color!("color.text.disabled", self.colors.text_disabled);

        apply_color!("color.accent", self.colors.accent);
        apply_color!(
            "color.selection.background",
            self.colors.selection_background
        );
        apply_color!("color.hover.background", self.colors.hover_background);
        apply_color!("color.focus.ring", self.colors.focus_ring);

        apply_color!("color.menu.background", self.colors.menu_background);
        apply_color!("color.menu.border", self.colors.menu_border);
        apply_color!("color.menu.item.hover", self.colors.menu_item_hover);
        apply_color!("color.menu.item.selected", self.colors.menu_item_selected);

        apply_color!("color.list.background", self.colors.list_background);
        apply_color!("color.list.border", self.colors.list_border);
        apply_color!("color.list.row.hover", self.colors.list_row_hover);
        apply_color!("color.list.row.selected", self.colors.list_row_selected);

        apply_color!("color.scrollbar.track", self.colors.scrollbar_track);
        apply_color!("color.scrollbar.thumb", self.colors.scrollbar_thumb);
        apply_color!(
            "color.scrollbar.thumb.hover",
            self.colors.scrollbar_thumb_hover
        );

        apply_color!(
            "color.viewport.selection.fill",
            self.colors.viewport_selection_fill
        );
        apply_color!(
            "color.viewport.selection.stroke",
            self.colors.viewport_selection_stroke
        );
        apply_color!("color.viewport.marker", self.colors.viewport_marker);
        apply_color!(
            "color.viewport.drag_line.pan",
            self.colors.viewport_drag_line_pan
        );
        apply_color!(
            "color.viewport.drag_line.orbit",
            self.colors.viewport_drag_line_orbit
        );
        apply_color!("color.viewport.gizmo.x", self.colors.viewport_gizmo_x);
        apply_color!("color.viewport.gizmo.y", self.colors.viewport_gizmo_y);
        apply_color!(
            "color.viewport.gizmo.handle.background",
            self.colors.viewport_gizmo_handle_background
        );
        apply_color!(
            "color.viewport.gizmo.handle.border",
            self.colors.viewport_gizmo_handle_border
        );
        apply_color!(
            "color.viewport.rotate_gizmo",
            self.colors.viewport_rotate_gizmo
        );

        // shadcn/gpui-component compatibility: if a theme only provides semantic keys (e.g.
        // `background`, `foreground`, `border`, `ring`, ...), backfill the typed baseline tokens.
        // This avoids subtle drift where legacy/runtime widgets read `theme.colors.*` while
        // component-layer code reads `theme.color_by_key(...)`.
        macro_rules! backfill_color_from_alias {
            ($canonical:literal, $field:expr, [$($alias:literal),+ $(,)?]) => {
                if !cfg.colors.contains_key($canonical) {
                    for alias in [$($alias),+] {
                        if let Some(v) = cfg.colors.get(alias)
                            && let Some(c) = parse_color_to_linear(v)
                        {
                            next_colors.insert($canonical.to_string(), c);
                            if $field != c {
                                $field = c;
                                changed = true;
                            }
                            break;
                        }
                    }
                }
            };
        }

        backfill_color_from_alias!(
            "color.surface.background",
            self.colors.surface_background,
            ["background"]
        );
        backfill_color_from_alias!(
            "color.text.primary",
            self.colors.text_primary,
            ["foreground"]
        );
        backfill_color_from_alias!("color.panel.border", self.colors.panel_border, ["border"]);
        backfill_color_from_alias!("color.focus.ring", self.colors.focus_ring, ["ring"]);
        backfill_color_from_alias!(
            "color.panel.background",
            self.colors.panel_background,
            ["card", "card.background", "popover", "popover.background"]
        );
        backfill_color_from_alias!(
            "color.menu.background",
            self.colors.menu_background,
            ["popover", "popover.background"]
        );
        backfill_color_from_alias!(
            "color.menu.border",
            self.colors.menu_border,
            ["popover.border"]
        );
        backfill_color_from_alias!(
            "color.accent",
            self.colors.accent,
            [
                "primary",
                "primary.background",
                "accent",
                "accent.background"
            ]
        );

        apply_metric!("metric.radius.sm", self.metrics.radius_sm);
        apply_metric!("metric.radius.md", self.metrics.radius_md);
        apply_metric!("metric.radius.lg", self.metrics.radius_lg);
        apply_metric!("metric.padding.sm", self.metrics.padding_sm);
        apply_metric!("metric.padding.md", self.metrics.padding_md);
        apply_metric!("metric.scrollbar.width", self.metrics.scrollbar_width);
        apply_metric!("metric.font.size", self.metrics.font_size);
        apply_metric!("metric.font.mono_size", self.metrics.mono_font_size);
        apply_metric!("metric.font.line_height", self.metrics.font_line_height);
        apply_metric!(
            "metric.font.mono_line_height",
            self.metrics.mono_font_line_height
        );

        // gpui-component compatibility: accept `font.size` / `mono_font.size` when the canonical
        // `metric.font.*` keys are not present.
        if !cfg.metrics.contains_key("metric.font.size")
            && let Some(v) = cfg.metrics.get("font.size").copied()
        {
            let px = Px(v);
            next_metrics.insert("metric.font.size".to_string(), px);
            next_metrics.insert("font.size".to_string(), px);
            if self.metrics.font_size != px {
                self.metrics.font_size = px;
                changed = true;
            }
        }
        if !cfg.metrics.contains_key("metric.font.mono_size")
            && let Some(v) = cfg.metrics.get("mono_font.size").copied()
        {
            let px = Px(v);
            next_metrics.insert("metric.font.mono_size".to_string(), px);
            next_metrics.insert("mono_font.size".to_string(), px);
            if self.metrics.mono_font_size != px {
                self.metrics.mono_font_size = px;
                changed = true;
            }
        }
        if !cfg.metrics.contains_key("metric.font.line_height")
            && let Some(v) = cfg.metrics.get("font.line_height").copied()
        {
            let px = Px(v);
            next_metrics.insert("metric.font.line_height".to_string(), px);
            next_metrics.insert("font.line_height".to_string(), px);
            if self.metrics.font_line_height != px {
                self.metrics.font_line_height = px;
                changed = true;
            }
        }
        if !cfg.metrics.contains_key("metric.font.mono_line_height")
            && let Some(v) = cfg.metrics.get("mono_font.line_height").copied()
        {
            let px = Px(v);
            next_metrics.insert("metric.font.mono_line_height".to_string(), px);
            next_metrics.insert("mono_font.line_height".to_string(), px);
            if self.metrics.mono_font_line_height != px {
                self.metrics.mono_font_line_height = px;
                changed = true;
            }
        }

        for (k, v) in &cfg.colors {
            if next_colors.contains_key(k) {
                continue;
            }
            if let Some(c) = parse_color_to_linear(v) {
                next_colors.insert(k.clone(), c);
            }
        }

        for (k, v) in &cfg.metrics {
            if next_metrics.contains_key(k) {
                continue;
            }
            next_metrics.insert(k.clone(), Px(*v));
        }

        if self.extra_colors != next_colors {
            self.extra_colors = next_colors;
            changed = true;
        }
        if self.extra_metrics != next_metrics {
            self.extra_metrics = next_metrics;
            changed = true;
        }

        if changed {
            self.revision = self.revision.saturating_add(1);
        }
    }
}

fn default_theme() -> &'static Theme {
    static DEFAULT: OnceLock<Theme> = OnceLock::new();
    DEFAULT.get_or_init(|| {
        let metrics = ThemeMetrics {
            radius_sm: Px(6.0),
            radius_md: Px(8.0),
            radius_lg: Px(10.0),
            padding_sm: Px(8.0),
            padding_md: Px(10.0),
            scrollbar_width: Px(10.0),
            font_size: Px(13.0),
            mono_font_size: Px(13.0),
            font_line_height: Px(16.0),
            mono_font_line_height: Px(16.0),
        };
        let colors = ThemeColors {
            surface_background: parse_hex_srgb_to_linear("#24272E").unwrap(),
            panel_background: parse_hex_srgb_to_linear("#2B3038").unwrap(),
            panel_border: parse_hex_srgb_to_linear("#3A424D").unwrap(),
            text_primary: parse_hex_srgb_to_linear("#D7DEE9").unwrap(),
            text_muted: parse_hex_srgb_to_linear("#AAB3C2").unwrap(),
            text_disabled: parse_hex_srgb_to_linear("#7D8798").unwrap(),
            accent: parse_hex_srgb_to_linear("#3D8BFF").unwrap(),
            selection_background: parse_hex_srgb_to_linear("#3D8BFF66").unwrap(),
            hover_background: parse_hex_srgb_to_linear("#363C46").unwrap(),
            focus_ring: parse_hex_srgb_to_linear("#3D8BFFCC").unwrap(),
            menu_background: parse_hex_srgb_to_linear("#2B3038").unwrap(),
            menu_border: parse_hex_srgb_to_linear("#3A424D").unwrap(),
            menu_item_hover: parse_hex_srgb_to_linear("#363C46").unwrap(),
            menu_item_selected: parse_hex_srgb_to_linear("#3D8BFF66").unwrap(),
            list_background: parse_hex_srgb_to_linear("#2B3038").unwrap(),
            list_border: parse_hex_srgb_to_linear("#3A424D").unwrap(),
            list_row_hover: parse_hex_srgb_to_linear("#363C46").unwrap(),
            list_row_selected: parse_hex_srgb_to_linear("#3D8BFF66").unwrap(),
            scrollbar_track: parse_hex_srgb_to_linear("#1C1F25").unwrap(),
            scrollbar_thumb: parse_hex_srgb_to_linear("#4C5666").unwrap(),
            scrollbar_thumb_hover: parse_hex_srgb_to_linear("#5A687D").unwrap(),

            viewport_selection_fill: parse_hex_srgb_to_linear("#3D8BFF29").unwrap(),
            viewport_selection_stroke: parse_hex_srgb_to_linear("#3D8BFFCC").unwrap(),
            viewport_marker: parse_hex_srgb_to_linear("#3D8BFFFF").unwrap(),
            viewport_drag_line_pan: parse_hex_srgb_to_linear("#33E684D9").unwrap(),
            viewport_drag_line_orbit: parse_hex_srgb_to_linear("#FFC44AD9").unwrap(),
            viewport_gizmo_x: parse_hex_srgb_to_linear("#E74C3CFF").unwrap(),
            viewport_gizmo_y: parse_hex_srgb_to_linear("#2ECC71FF").unwrap(),
            viewport_gizmo_handle_background: parse_hex_srgb_to_linear("#1E2229FF").unwrap(),
            viewport_gizmo_handle_border: parse_hex_srgb_to_linear("#D7DEE9FF").unwrap(),
            viewport_rotate_gizmo: parse_hex_srgb_to_linear("#FFC44AFF").unwrap(),
        };

        Theme {
            name: "Fret Default (Dark)".to_string(),
            author: Some("Fret".to_string()),
            url: None,
            revision: 1,
            metrics,
            colors,
            extra_colors: default_color_tokens(colors),
            extra_metrics: default_metric_tokens(metrics),
        }
    })
}

fn parse_color_to_linear(s: &str) -> Option<Color> {
    parse_hex_srgb_to_linear(s)
        .or_else(|| parse_hsl_tokens_to_linear(s))
        .or_else(|| parse_oklch_to_linear(s))
}

fn parse_hex_srgb_to_linear(s: &str) -> Option<Color> {
    let s = s.trim();
    let hex = s.strip_prefix('#').unwrap_or(s);
    let (r, g, b, a) = match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            (r, g, b, 255)
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            (r, g, b, a)
        }
        _ => return None,
    };

    Some(Color {
        r: srgb_channel_to_linear(r),
        g: srgb_channel_to_linear(g),
        b: srgb_channel_to_linear(b),
        a: a as f32 / 255.0,
    })
}

fn srgb_channel_to_linear(u: u8) -> f32 {
    let c = u as f32 / 255.0;
    srgb_f32_to_linear(c)
}

fn srgb_f32_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn parse_hsl_tokens_to_linear(s: &str) -> Option<Color> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    let inner = s
        .strip_prefix("hsl(")
        .and_then(|rest| rest.strip_suffix(')'))
        .unwrap_or(s);

    // shadcn v3 theme tokens use `H S% L%` (space-separated) or the same inside `hsl(...)`.
    let parts: Vec<&str> = inner
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|p| !p.is_empty())
        .collect();
    if parts.len() != 3 {
        return None;
    }

    let h_deg: f32 = parts[0].parse().ok()?;
    let s_pct: f32 = parts[1].trim_end_matches('%').parse().ok()?;
    let l_pct: f32 = parts[2].trim_end_matches('%').parse().ok()?;

    let h = (h_deg % 360.0 + 360.0) % 360.0 / 360.0;
    let s = (s_pct / 100.0).clamp(0.0, 1.0);
    let l = (l_pct / 100.0).clamp(0.0, 1.0);

    let (r_srgb, g_srgb, b_srgb) = hsl_to_srgb(h, s, l);
    Some(Color {
        r: srgb_f32_to_linear(r_srgb.clamp(0.0, 1.0)),
        g: srgb_f32_to_linear(g_srgb.clamp(0.0, 1.0)),
        b: srgb_f32_to_linear(b_srgb.clamp(0.0, 1.0)),
        a: 1.0,
    })
}

fn hsl_to_srgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    if s == 0.0 {
        return (l, l, l);
    }

    fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
        if t < 0.0 {
            t += 1.0;
        }
        if t > 1.0 {
            t -= 1.0;
        }
        if t < 1.0 / 6.0 {
            return p + (q - p) * 6.0 * t;
        }
        if t < 1.0 / 2.0 {
            return q;
        }
        if t < 2.0 / 3.0 {
            return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
        }
        p
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    (
        hue_to_rgb(p, q, h + 1.0 / 3.0),
        hue_to_rgb(p, q, h),
        hue_to_rgb(p, q, h - 1.0 / 3.0),
    )
}

fn parse_oklch_to_linear(s: &str) -> Option<Color> {
    let s = s.trim();
    let inner = s.strip_prefix("oklch(")?.strip_suffix(')')?.trim();

    // Accept `oklch(L C H / A)` where A can be `0..1` or `NN%`.
    let (main, alpha_part) = if let Some((l, r)) = inner.split_once('/') {
        (l.trim(), Some(r.trim()))
    } else {
        (inner, None)
    };

    let parts: Vec<&str> = main
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|p| !p.is_empty())
        .collect();
    if parts.len() != 3 {
        return None;
    }

    let l: f32 = parts[0].parse().ok()?;
    let c: f32 = parts[1].parse().ok()?;
    let h_deg: f32 = parts[2].parse().ok()?;

    let alpha = if let Some(a) = alpha_part {
        if let Some(pct) = a.trim_end_matches('%').parse::<f32>().ok()
            && a.trim_end().ends_with('%')
        {
            (pct / 100.0).clamp(0.0, 1.0)
        } else {
            a.parse::<f32>().ok()?.clamp(0.0, 1.0)
        }
    } else {
        1.0
    };

    // OKLCH -> OKLab
    let h_rad = h_deg.to_radians();
    let a = c * h_rad.cos();
    let b = c * h_rad.sin();

    // OKLab -> linear sRGB (Björn Ottosson's reference implementation)
    let l_ = l + 0.396_337_777_4 * a + 0.215_803_757_3 * b;
    let m_ = l - 0.105_561_345_8 * a - 0.063_854_172_8 * b;
    let s_ = l - 0.089_484_177_5 * a - 1.291_485_548_0 * b;

    let l3 = l_ * l_ * l_;
    let m3 = m_ * m_ * m_;
    let s3 = s_ * s_ * s_;

    let r_lin = 4.076_741_662_1 * l3 - 3.307_711_591_3 * m3 + 0.230_969_929_2 * s3;
    let g_lin = -1.268_438_004_6 * l3 + 2.609_757_401_1 * m3 - 0.341_319_396_5 * s3;
    let b_lin = -0.004_196_086_3 * l3 - 0.703_418_614_7 * m3 + 1.707_614_701_0 * s3;

    Some(Color {
        r: r_lin.clamp(0.0, 1.0),
        g: g_lin.clamp(0.0, 1.0),
        b: b_lin.clamp(0.0, 1.0),
        a: alpha,
    })
}

#[cfg(test)]
mod tests {
    use super::Theme;
    use super::ThemeConfig;
    use super::parse_color_to_linear;
    use std::collections::HashMap;

    #[test]
    fn shadcn_semantic_palette_aliases_exist_on_default_theme() {
        let host = crate::test_host::TestHost::default();
        let theme = Theme::global(&host);

        for key in [
            "background",
            "foreground",
            "border",
            "ring",
            "ring-offset-background",
            "card",
            "card.background",
            "card-foreground",
            "card.foreground",
            "primary",
            "primary.background",
            "primary-foreground",
            "primary.foreground",
            "secondary",
            "secondary.background",
            "secondary-foreground",
            "secondary.foreground",
            "destructive",
            "destructive.background",
            "destructive-foreground",
            "destructive.foreground",
            "muted",
            "input.background",
            "input",
            "input.foreground",
            "accent",
            "popover.background",
            "popover.foreground",
        ] {
            assert!(theme.color_by_key(key).is_some(), "missing alias {key}");
        }
    }

    #[test]
    fn semantic_keys_backfill_typed_baseline_colors_when_missing() {
        let mut theme = Theme::global(&crate::test_host::TestHost::default()).clone();

        let mut colors = HashMap::new();
        colors.insert("background".to_string(), "#000000".to_string());
        colors.insert("foreground".to_string(), "#ffffff".to_string());
        colors.insert("border".to_string(), "#ff0000".to_string());
        colors.insert("ring".to_string(), "#00ff00".to_string());
        colors.insert("primary".to_string(), "#0000ff".to_string());
        let cfg = ThemeConfig {
            name: "Semantic Only".to_string(),
            colors,
            ..Default::default()
        };

        // No `color.*` keys are provided; typed fields should still change.
        theme.apply_config(&cfg);

        let bg = theme.color_by_key("background").expect("background");
        let fg = theme.color_by_key("foreground").expect("foreground");
        let border = theme.color_by_key("border").expect("border");
        let ring = theme.color_by_key("ring").expect("ring");
        let primary = theme.color_by_key("primary").expect("primary");

        assert_eq!(theme.colors.surface_background, bg);
        assert_eq!(theme.colors.text_primary, fg);
        assert_eq!(theme.colors.panel_border, border);
        assert_eq!(theme.colors.focus_ring, ring);
        assert_eq!(theme.colors.accent, primary);
    }

    #[test]
    fn parse_color_supports_shadcn_hsl_tokens() {
        let white = parse_color_to_linear("0 0% 100%").expect("hsl tokens");
        assert!((white.r - 1.0).abs() < 1e-6);
        assert!((white.g - 1.0).abs() < 1e-6);
        assert!((white.b - 1.0).abs() < 1e-6);
        assert!((white.a - 1.0).abs() < 1e-6);
    }

    #[test]
    fn parse_color_supports_shadcn_oklch_tokens_with_alpha() {
        let c = parse_color_to_linear("oklch(1 0 0 / 10%)").expect("oklch");
        assert!((c.r - 1.0).abs() < 1e-6);
        assert!((c.g - 1.0).abs() < 1e-6);
        assert!((c.b - 1.0).abs() < 1e-6);
        assert!((c.a - 0.1).abs() < 1e-6);
    }
}
