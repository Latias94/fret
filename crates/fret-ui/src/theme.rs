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
        self.extra_colors.get(key).copied()
    }

    pub fn metric_by_key(&self, key: &str) -> Option<Px> {
        self.extra_metrics.get(key).copied()
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
                    if let Some(c) = parse_hex_srgb_to_linear(v) {
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

        apply_metric!("metric.radius.sm", self.metrics.radius_sm);
        apply_metric!("metric.radius.md", self.metrics.radius_md);
        apply_metric!("metric.radius.lg", self.metrics.radius_lg);
        apply_metric!("metric.padding.sm", self.metrics.padding_sm);
        apply_metric!("metric.padding.md", self.metrics.padding_md);
        apply_metric!("metric.scrollbar.width", self.metrics.scrollbar_width);

        for (k, v) in &cfg.colors {
            if next_colors.contains_key(k) {
                continue;
            }
            if let Some(c) = parse_hex_srgb_to_linear(v) {
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

    fn srgb_channel_to_linear(u: u8) -> f32 {
        let c = u as f32 / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }

    Some(Color {
        r: srgb_channel_to_linear(r),
        g: srgb_channel_to_linear(g),
        b: srgb_channel_to_linear(b),
        a: a as f32 / 255.0,
    })
}
