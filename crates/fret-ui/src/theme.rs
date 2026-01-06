use fret_core::{Color, Px};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::OnceLock};

use crate::UiHost;
use crate::{ThemeColorKey, ThemeMetricKey};

fn with_alpha(color: Color, alpha: f32) -> Color {
    Color {
        a: alpha.clamp(0.0, 1.0),
        ..color
    }
}

fn default_color_tokens(colors: ThemeColors) -> HashMap<String, Color> {
    let mut out = HashMap::new();

    // shadcn/new-york core semantic palette (canonical names).
    out.insert("background".to_string(), colors.surface_background);
    out.insert("foreground".to_string(), colors.text_primary);
    out.insert("border".to_string(), colors.panel_border);
    out.insert("input".to_string(), colors.panel_border);
    out.insert("ring".to_string(), colors.focus_ring);
    out.insert(
        "ring-offset-background".to_string(),
        colors.surface_background,
    );

    out.insert("card".to_string(), colors.panel_background);
    out.insert("card-foreground".to_string(), colors.text_primary);

    out.insert("popover".to_string(), colors.menu_background);
    out.insert("popover-foreground".to_string(), colors.text_primary);

    out.insert("muted".to_string(), colors.panel_background);
    out.insert("muted-foreground".to_string(), colors.text_muted);

    out.insert("accent".to_string(), colors.hover_background);
    out.insert("accent-foreground".to_string(), colors.text_primary);

    out.insert("primary".to_string(), colors.accent);
    out.insert("primary-foreground".to_string(), colors.text_primary);

    out.insert("secondary".to_string(), colors.panel_background);
    out.insert("secondary-foreground".to_string(), colors.text_primary);

    out.insert("destructive".to_string(), colors.viewport_gizmo_x);
    out.insert("destructive-foreground".to_string(), colors.text_primary);

    // shadcn/new-york extension tokens used by the upstream site (optional but stable).
    out.insert("selection".to_string(), colors.selection_background);
    out.insert("selection-foreground".to_string(), colors.text_primary);

    // Fret-specific tokens (namespaced).
    out.insert("fret.text.disabled".to_string(), colors.text_disabled);

    out.insert("fret.menu.background".to_string(), colors.menu_background);
    out.insert("fret.menu.border".to_string(), colors.menu_border);
    out.insert("fret.menu.item.hover".to_string(), colors.menu_item_hover);
    out.insert(
        "fret.menu.item.selected".to_string(),
        colors.menu_item_selected,
    );

    out.insert("fret.list.background".to_string(), colors.list_background);
    out.insert("fret.list.border".to_string(), colors.list_border);
    out.insert("fret.list.row.hover".to_string(), colors.list_row_hover);
    out.insert(
        "fret.list.row.selected".to_string(),
        colors.list_row_selected,
    );

    out.insert("fret.scrollbar.track".to_string(), colors.scrollbar_track);
    out.insert("fret.scrollbar.thumb".to_string(), colors.scrollbar_thumb);
    out.insert(
        "fret.scrollbar.thumb.hover".to_string(),
        colors.scrollbar_thumb_hover,
    );

    out.insert(
        "fret.viewport.selection.fill".to_string(),
        colors.viewport_selection_fill,
    );
    out.insert(
        "fret.viewport.selection.stroke".to_string(),
        colors.viewport_selection_stroke,
    );
    out.insert("fret.viewport.marker".to_string(), colors.viewport_marker);
    out.insert(
        "fret.viewport.drag_line.pan".to_string(),
        colors.viewport_drag_line_pan,
    );
    out.insert(
        "fret.viewport.drag_line.orbit".to_string(),
        colors.viewport_drag_line_orbit,
    );
    out.insert("fret.viewport.gizmo.x".to_string(), colors.viewport_gizmo_x);
    out.insert("fret.viewport.gizmo.y".to_string(), colors.viewport_gizmo_y);
    out.insert(
        "fret.viewport.gizmo.handle.background".to_string(),
        colors.viewport_gizmo_handle_background,
    );
    out.insert(
        "fret.viewport.gizmo.handle.border".to_string(),
        colors.viewport_gizmo_handle_border,
    );
    out.insert(
        "fret.viewport.rotate_gizmo".to_string(),
        colors.viewport_rotate_gizmo,
    );

    out
}

fn default_metric_tokens(metrics: ThemeMetrics) -> HashMap<String, Px> {
    let mut out = HashMap::new();

    // shadcn/new-york core semantic metric(s).
    //
    // Matches the upstream contract: sm = radius - 4px, md = radius - 2px, lg = radius.
    out.insert("radius".to_string(), metrics.radius_lg);

    // Fret typography baseline (framework-level; not part of shadcn's variable set).
    out.insert("font.size".to_string(), metrics.font_size);
    out.insert("mono_font.size".to_string(), metrics.mono_font_size);
    out.insert("font.line_height".to_string(), metrics.font_line_height);
    out.insert(
        "mono_font.line_height".to_string(),
        metrics.mono_font_line_height,
    );

    // Fret layout baseline (namespaced).
    out.insert("fret.padding.sm".to_string(), metrics.padding_sm);
    out.insert("fret.padding.md".to_string(), metrics.padding_md);
    out.insert("fret.scrollbar.width".to_string(), metrics.scrollbar_width);

    out
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

    pub fn color(&self, key: ThemeColorKey) -> Color {
        self.color_by_key(key.canonical_name())
            .unwrap_or_else(|| panic!("missing core theme color key {}", key.canonical_name()))
    }

    pub fn metric(&self, key: ThemeMetricKey) -> Px {
        self.metric_by_key(key.canonical_name())
            .unwrap_or_else(|| panic!("missing core theme metric key {}", key.canonical_name()))
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

    pub fn with_global_mut<H: UiHost, R>(app: &mut H, f: impl FnOnce(&mut Theme) -> R) -> R {
        app.with_global_mut(|| default_theme().clone(), |theme, _app| f(theme))
    }

    pub fn apply_config(&mut self, cfg: &ThemeConfig) {
        self.name = cfg.name.clone();
        self.author = cfg.author.clone();
        self.url = cfg.url.clone();

        assert_no_legacy_theme_keys(cfg);

        let mut changed = false;

        let mut next_colors = default_color_tokens(self.colors);
        let mut next_metrics = default_metric_tokens(self.metrics);

        macro_rules! apply_semantic_color {
            ($key:literal, $set:expr) => {
                if let Some(v) = cfg.colors.get($key) {
                    if let Some(c) = parse_color_to_linear(v) {
                        next_colors.insert($key.to_string(), c);
                        $set(c);
                    }
                }
            };
        }

        macro_rules! apply_semantic_metric {
            ($key:literal, $set:expr) => {
                if let Some(v) = cfg.metrics.get($key).copied() {
                    let px = Px(v);
                    next_metrics.insert($key.to_string(), px);
                    $set(px);
                }
            };
        }

        // Apply shadcn semantic keys first; typed tokens are derived from them for consistency.
        apply_semantic_color!("background", |c| {
            if self.colors.surface_background != c {
                self.colors.surface_background = c;
                changed = true;
            }
        });
        apply_semantic_color!("foreground", |c| {
            if self.colors.text_primary != c {
                self.colors.text_primary = c;
                changed = true;
            }
        });
        apply_semantic_color!("border", |c| {
            if self.colors.panel_border != c {
                self.colors.panel_border = c;
                changed = true;
            }
            if self.colors.menu_border != c {
                self.colors.menu_border = c;
                changed = true;
            }
            if self.colors.list_border != c {
                self.colors.list_border = c;
                changed = true;
            }
        });
        apply_semantic_color!("input", |c| {
            if !cfg.colors.contains_key("border") {
                if self.colors.panel_border != c {
                    self.colors.panel_border = c;
                    changed = true;
                }
                if self.colors.menu_border != c {
                    self.colors.menu_border = c;
                    changed = true;
                }
                if self.colors.list_border != c {
                    self.colors.list_border = c;
                    changed = true;
                }
            }
        });
        apply_semantic_color!("ring", |c| {
            if self.colors.focus_ring != c {
                self.colors.focus_ring = c;
                changed = true;
            }
        });
        apply_semantic_color!("card", |c| {
            if self.colors.panel_background != c {
                self.colors.panel_background = c;
                changed = true;
            }
            if !cfg.colors.contains_key("fret.list.background") && self.colors.list_background != c
            {
                self.colors.list_background = c;
                changed = true;
            }
        });
        apply_semantic_color!("popover", |c| {
            if self.colors.menu_background != c {
                self.colors.menu_background = c;
                changed = true;
            }
        });
        apply_semantic_color!("muted-foreground", |c| {
            if self.colors.text_muted != c {
                self.colors.text_muted = c;
                changed = true;
            }
        });
        apply_semantic_color!("accent", |c| {
            if self.colors.hover_background != c {
                self.colors.hover_background = c;
                changed = true;
            }
            if !cfg.colors.contains_key("fret.menu.item.hover") && self.colors.menu_item_hover != c
            {
                self.colors.menu_item_hover = c;
                changed = true;
            }
            if !cfg.colors.contains_key("fret.list.row.hover") && self.colors.list_row_hover != c {
                self.colors.list_row_hover = c;
                changed = true;
            }
        });
        apply_semantic_color!("primary", |c| {
            if self.colors.accent != c {
                self.colors.accent = c;
                changed = true;
            }
            if !cfg.colors.contains_key("selection") {
                let selection = with_alpha(c, 0.4);
                if self.colors.selection_background != selection {
                    self.colors.selection_background = selection;
                    changed = true;
                }
            }
        });

        apply_semantic_metric!("radius", |px| {
            if self.metrics.radius_lg != px {
                self.metrics.radius_lg = px;
                changed = true;
            }

            let md = Px((px.0 - 2.0).max(0.0));
            let sm = Px((px.0 - 4.0).max(0.0));
            if self.metrics.radius_md != md {
                self.metrics.radius_md = md;
                changed = true;
            }
            if self.metrics.radius_sm != sm {
                self.metrics.radius_sm = sm;
                changed = true;
            }
        });
        apply_semantic_metric!("font.size", |px| {
            if self.metrics.font_size != px {
                self.metrics.font_size = px;
                changed = true;
            }
        });
        apply_semantic_metric!("mono_font.size", |px| {
            if self.metrics.mono_font_size != px {
                self.metrics.mono_font_size = px;
                changed = true;
            }
        });
        apply_semantic_metric!("font.line_height", |px| {
            if self.metrics.font_line_height != px {
                self.metrics.font_line_height = px;
                changed = true;
            }
        });
        apply_semantic_metric!("mono_font.line_height", |px| {
            if self.metrics.mono_font_line_height != px {
                self.metrics.mono_font_line_height = px;
                changed = true;
            }
        });

        apply_semantic_color!("selection", |c| {
            if self.colors.selection_background != c {
                self.colors.selection_background = c;
                changed = true;
            }
        });

        if !cfg.colors.contains_key("fret.menu.item.selected")
            && self.colors.menu_item_selected != self.colors.selection_background
        {
            self.colors.menu_item_selected = self.colors.selection_background;
            changed = true;
        }
        if !cfg.colors.contains_key("fret.list.row.selected")
            && self.colors.list_row_selected != self.colors.selection_background
        {
            self.colors.list_row_selected = self.colors.selection_background;
            changed = true;
        }

        apply_semantic_color!("fret.text.disabled", |c| {
            if self.colors.text_disabled != c {
                self.colors.text_disabled = c;
                changed = true;
            }
        });

        apply_semantic_color!("fret.menu.background", |c| {
            if self.colors.menu_background != c {
                self.colors.menu_background = c;
                changed = true;
            }
        });
        apply_semantic_color!("fret.menu.border", |c| {
            if self.colors.menu_border != c {
                self.colors.menu_border = c;
                changed = true;
            }
        });
        apply_semantic_color!("fret.menu.item.hover", |c| {
            if self.colors.menu_item_hover != c {
                self.colors.menu_item_hover = c;
                changed = true;
            }
        });
        apply_semantic_color!("fret.menu.item.selected", |c| {
            if self.colors.menu_item_selected != c {
                self.colors.menu_item_selected = c;
                changed = true;
            }
        });

        apply_semantic_color!("fret.list.background", |c| {
            if self.colors.list_background != c {
                self.colors.list_background = c;
                changed = true;
            }
        });
        apply_semantic_color!("fret.list.border", |c| {
            if self.colors.list_border != c {
                self.colors.list_border = c;
                changed = true;
            }
        });
        apply_semantic_color!("fret.list.row.hover", |c| {
            if self.colors.list_row_hover != c {
                self.colors.list_row_hover = c;
                changed = true;
            }
        });
        apply_semantic_color!("fret.list.row.selected", |c| {
            if self.colors.list_row_selected != c {
                self.colors.list_row_selected = c;
                changed = true;
            }
        });

        apply_semantic_color!("fret.scrollbar.track", |c| {
            if self.colors.scrollbar_track != c {
                self.colors.scrollbar_track = c;
                changed = true;
            }
        });
        apply_semantic_color!("fret.scrollbar.thumb", |c| {
            if self.colors.scrollbar_thumb != c {
                self.colors.scrollbar_thumb = c;
                changed = true;
            }
        });
        apply_semantic_color!("fret.scrollbar.thumb.hover", |c| {
            if self.colors.scrollbar_thumb_hover != c {
                self.colors.scrollbar_thumb_hover = c;
                changed = true;
            }
        });

        apply_semantic_color!("fret.viewport.selection.fill", |c| {
            if self.colors.viewport_selection_fill != c {
                self.colors.viewport_selection_fill = c;
                changed = true;
            }
        });
        apply_semantic_color!("fret.viewport.selection.stroke", |c| {
            if self.colors.viewport_selection_stroke != c {
                self.colors.viewport_selection_stroke = c;
                changed = true;
            }
        });
        apply_semantic_color!("fret.viewport.marker", |c| {
            if self.colors.viewport_marker != c {
                self.colors.viewport_marker = c;
                changed = true;
            }
        });
        apply_semantic_color!("fret.viewport.drag_line.pan", |c| {
            if self.colors.viewport_drag_line_pan != c {
                self.colors.viewport_drag_line_pan = c;
                changed = true;
            }
        });
        apply_semantic_color!("fret.viewport.drag_line.orbit", |c| {
            if self.colors.viewport_drag_line_orbit != c {
                self.colors.viewport_drag_line_orbit = c;
                changed = true;
            }
        });
        apply_semantic_color!("fret.viewport.gizmo.x", |c| {
            if self.colors.viewport_gizmo_x != c {
                self.colors.viewport_gizmo_x = c;
                changed = true;
            }
        });
        apply_semantic_color!("fret.viewport.gizmo.y", |c| {
            if self.colors.viewport_gizmo_y != c {
                self.colors.viewport_gizmo_y = c;
                changed = true;
            }
        });
        apply_semantic_color!("fret.viewport.gizmo.handle.background", |c| {
            if self.colors.viewport_gizmo_handle_background != c {
                self.colors.viewport_gizmo_handle_background = c;
                changed = true;
            }
        });
        apply_semantic_color!("fret.viewport.gizmo.handle.border", |c| {
            if self.colors.viewport_gizmo_handle_border != c {
                self.colors.viewport_gizmo_handle_border = c;
                changed = true;
            }
        });
        apply_semantic_color!("fret.viewport.rotate_gizmo", |c| {
            if self.colors.viewport_rotate_gizmo != c {
                self.colors.viewport_rotate_gizmo = c;
                changed = true;
            }
        });

        apply_semantic_metric!("fret.padding.sm", |px| {
            if self.metrics.padding_sm != px {
                self.metrics.padding_sm = px;
                changed = true;
            }
        });
        apply_semantic_metric!("fret.padding.md", |px| {
            if self.metrics.padding_md != px {
                self.metrics.padding_md = px;
                changed = true;
            }
        });
        apply_semantic_metric!("fret.scrollbar.width", |px| {
            if self.metrics.scrollbar_width != px {
                self.metrics.scrollbar_width = px;
                changed = true;
            }
        });

        next_colors = default_color_tokens(self.colors);
        next_metrics = default_metric_tokens(self.metrics);

        for (k, v) in &cfg.colors {
            if let Some(c) = parse_color_to_linear(v) {
                next_colors.insert(k.clone(), c);
            }
        }

        for (k, v) in &cfg.metrics {
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

fn assert_no_legacy_theme_keys(cfg: &ThemeConfig) {
    for key in cfg.colors.keys() {
        if key.starts_with("color.") {
            panic!(
                "legacy theme color key {key} is not supported; use semantic keys (e.g. background) or namespaced keys (e.g. fret.*)"
            );
        }

        let is_semantic_alias = matches!(
            key.split_once('.').map(|(head, _)| head),
            Some(
                "background"
                    | "foreground"
                    | "border"
                    | "input"
                    | "ring"
                    | "card"
                    | "popover"
                    | "primary"
                    | "secondary"
                    | "muted"
                    | "accent"
                    | "destructive"
            )
        );
        if is_semantic_alias {
            panic!(
                "legacy theme alias key {key} is not supported; use canonical shadcn names (e.g. card-foreground) instead"
            );
        }
    }

    for key in cfg.metrics.keys() {
        if key.starts_with("metric.") {
            panic!(
                "legacy theme metric key {key} is not supported; use semantic keys (e.g. radius) or namespaced keys (e.g. fret.*)"
            );
        }
        if matches!(key.as_str(), "radius.sm" | "radius.md" | "radius.lg") {
            panic!("legacy radius key {key} is not supported; use radius only");
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
    use crate::{ThemeColorKey, ThemeMetricKey};
    use std::collections::HashMap;

    #[test]
    fn shadcn_semantic_palette_keys_exist_on_default_theme() {
        let host = crate::test_host::TestHost::default();
        let theme = Theme::global(&host);

        for key in ThemeColorKey::ALL {
            let name = key.canonical_name();
            assert!(theme.color_by_key(name).is_some(), "missing {name}");
        }
        for key in ThemeMetricKey::ALL {
            let name = key.canonical_name();
            assert!(theme.metric_by_key(name).is_some(), "missing {name}");
        }

        assert!(theme.color_by_key("selection").is_some());
        assert!(theme.color_by_key("fret.menu.background").is_some());
        assert!(theme.metric_by_key("fret.padding.sm").is_some());

        for legacy in [
            "input.border",
            "card.background",
            "popover.background",
            "color.surface.background",
        ] {
            assert!(theme.color_by_key(legacy).is_none(), "legacy key {legacy}");
        }
        for legacy in [
            "metric.radius.sm",
            "metric.padding.sm",
            "radius.sm",
            "radius.lg",
        ] {
            assert!(theme.metric_by_key(legacy).is_none(), "legacy key {legacy}");
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
        colors.insert("muted-foreground".to_string(), "#00ffff".to_string());
        let cfg = ThemeConfig {
            name: "Semantic Only".to_string(),
            colors,
            ..Default::default()
        };

        // 仅提供语义键；typed baseline 仍应改变。
        theme.apply_config(&cfg);

        let bg = theme.color_by_key("background").expect("background");
        let fg = theme.color_by_key("foreground").expect("foreground");
        let border = theme.color_by_key("border").expect("border");
        let ring = theme.color_by_key("ring").expect("ring");
        let primary = theme.color_by_key("primary").expect("primary");
        let muted_fg = theme
            .color_by_key("muted-foreground")
            .expect("muted-foreground");

        assert_eq!(theme.colors.surface_background, bg);
        assert_eq!(theme.colors.text_primary, fg);
        assert_eq!(theme.colors.panel_border, border);
        assert_eq!(theme.colors.focus_ring, ring);
        assert_eq!(theme.colors.accent, primary);
        assert_eq!(theme.colors.text_muted, muted_fg);
    }

    #[test]
    fn semantic_keys_backfill_panel_border_from_input_when_border_missing() {
        let mut theme = Theme::global(&crate::test_host::TestHost::default()).clone();

        let mut colors = HashMap::new();
        colors.insert("background".to_string(), "#000000".to_string());
        colors.insert("foreground".to_string(), "#ffffff".to_string());
        colors.insert("input".to_string(), "#ff0000".to_string());
        let cfg = ThemeConfig {
            name: "Semantic Input Border".to_string(),
            colors,
            ..Default::default()
        };

        theme.apply_config(&cfg);

        let input = theme.color_by_key("input").expect("input");
        assert_eq!(theme.colors.panel_border, input);
        assert_eq!(theme.colors.menu_border, input);
        assert_eq!(theme.colors.list_border, input);
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

    #[test]
    fn typed_theme_keys_resolve_via_semantic_palette() {
        let host = crate::test_host::TestHost::default();
        let theme = Theme::global(&host);

        assert_eq!(
            theme.color(ThemeColorKey::PopoverForeground),
            theme
                .color_by_key("popover-foreground")
                .expect("popover-foreground")
        );
        assert_eq!(
            theme.metric(ThemeMetricKey::Radius),
            theme.metric_by_key("radius").expect("radius")
        );
    }

    #[test]
    #[should_panic]
    fn apply_config_rejects_legacy_color_keys() {
        let mut theme = Theme::global(&crate::test_host::TestHost::default()).clone();
        theme.apply_config(&ThemeConfig {
            name: "Legacy Color".to_string(),
            colors: HashMap::from([(
                "color.surface.background".to_string(),
                "#000000".to_string(),
            )]),
            ..ThemeConfig::default()
        });
    }

    #[test]
    #[should_panic]
    fn apply_config_rejects_legacy_metric_keys() {
        let mut theme = Theme::global(&crate::test_host::TestHost::default()).clone();
        theme.apply_config(&ThemeConfig {
            name: "Legacy Metric".to_string(),
            metrics: HashMap::from([("metric.padding.sm".to_string(), 12.0)]),
            ..ThemeConfig::default()
        });
    }
}
