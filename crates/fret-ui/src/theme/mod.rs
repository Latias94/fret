pub mod keys;
pub(crate) mod registry;

use fret_core::{Color, Corners, Px, TextStyle};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};

use crate::UiHost;
use crate::theme_registry::{ThemeTokenKind, canonicalize_token_key};
use crate::{ThemeColorKey, ThemeMetricKey};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CubicBezier {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

fn default_color_tokens(colors: ThemeColors) -> HashMap<String, Color> {
    let mut out = HashMap::from([
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
    ]);

    // Viewport 3D tooling extensions (not yet part of the typed `ThemeColors` baseline).
    // These are used by engine-pass gizmos and are theme-overridable via JSON theme configs.
    out.insert(
        "color.viewport.gizmo.z".to_string(),
        Color {
            r: 0.2,
            g: 0.5,
            b: 1.0,
            a: 1.0,
        },
    );
    out.insert(
        "color.viewport.gizmo.hover".to_string(),
        colors.viewport_rotate_gizmo,
    );
    out.insert(
        "color.viewport.view_gizmo.face".to_string(),
        Color {
            r: 0.22,
            g: 0.22,
            b: 0.24,
            a: 0.35,
        },
    );
    out.insert(
        "color.viewport.view_gizmo.edge".to_string(),
        Color {
            r: 0.95,
            g: 0.95,
            b: 0.98,
            a: 0.90,
        },
    );

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
    out.insert("popover.border".to_string(), colors.menu_border);

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

    // Common non-core semantic keys used by recipes (kept for the migration window).
    out.insert(
        "selection.background".to_string(),
        colors.selection_background,
    );
    out.insert("input.background".to_string(), colors.panel_background);
    out.insert("input.foreground".to_string(), colors.text_primary);
    out.insert("caret".to_string(), colors.text_primary);
    out.insert("scrollbar.background".to_string(), colors.scrollbar_track);
    out.insert(
        "scrollbar.thumb.background".to_string(),
        colors.scrollbar_thumb,
    );
    out.insert(
        "scrollbar.thumb.hover.background".to_string(),
        colors.scrollbar_thumb_hover,
    );

    out
}

fn default_metric_tokens(metrics: ThemeMetrics) -> HashMap<String, Px> {
    let mut out = HashMap::from([
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
    ]);

    // shadcn/new-york core semantic metrics (canonical names).
    out.insert("radius".to_string(), metrics.radius_sm);
    out.insert("radius.lg".to_string(), metrics.radius_md);
    out.insert("font.size".to_string(), metrics.font_size);
    out.insert("mono_font.size".to_string(), metrics.mono_font_size);
    out.insert("font.line_height".to_string(), metrics.font_line_height);
    out.insert(
        "mono_font.line_height".to_string(),
        metrics.mono_font_line_height,
    );

    // Typography defaults used by shadcn/ui-kit helpers.
    //
    // These keys are intentionally treated as "optional overrides" by higher-level components,
    // but some call sites use `metric_required` directly. Seed reasonable fallbacks here so
    // custom themes don't crash when they omit them.
    out.insert("component.text.sm_px".to_string(), metrics.font_size);
    out.insert(
        "component.text.sm_line_height".to_string(),
        metrics.font_line_height,
    );
    out.insert(
        "component.text.base_px".to_string(),
        Px(metrics.font_size.0 + 1.0),
    );
    out.insert(
        "component.text.base_line_height".to_string(),
        metrics.font_line_height,
    );

    // Legacy generic size tokens used by some shadcn ports/tests.
    // Prefer `component.size.*` tokens in new code.
    out.insert("metric.size.sm".to_string(), Px(32.0));
    out.insert("metric.size.md".to_string(), Px(36.0));
    out.insert("metric.size.lg".to_string(), Px(40.0));

    // `fret-markdown` canonical metrics.
    //
    // Keep this value derived from baseline mono font metrics so it tracks theme typography.
    // This is intended as a "reasonable default" (roughly 16 lines) rather than a hard rule.
    let code_block_max_height =
        Px((metrics.mono_font_line_height.0 * 16.0).max(metrics.mono_font_size.0 * 18.0));
    out.insert(
        "fret.markdown.code_block.max_height".to_string(),
        code_block_max_height,
    );

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
    pub corners: HashMap<String, Corners>,
    pub numbers: HashMap<String, f32>,
    pub durations_ms: HashMap<String, u32>,
    pub easings: HashMap<String, CubicBezier>,
    pub text_styles: HashMap<String, TextStyle>,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            author: None,
            url: None,
            colors: HashMap::new(),
            metrics: HashMap::new(),
            corners: HashMap::new(),
            numbers: HashMap::new(),
            durations_ms: HashMap::new(),
            easings: HashMap::new(),
            text_styles: HashMap::new(),
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

impl ThemeSnapshot {
    pub fn color_by_key(&self, key: &str) -> Option<Color> {
        let key = canonicalize_token_key(ThemeTokenKind::Color, key);
        let c = match key {
            // Baseline dotted keys (ADR 0050). These are useful for app/editor overlays and for
            // gradual migration away from typed theme field reads.
            "color.surface.background" => self.colors.surface_background,
            "color.panel.background" => self.colors.panel_background,
            "color.panel.border" => self.colors.panel_border,
            "color.text.primary" => self.colors.text_primary,
            "color.text.muted" => self.colors.text_muted,
            "color.text.disabled" => self.colors.text_disabled,
            "color.accent" => self.colors.accent,
            "color.selection.background" => self.colors.selection_background,
            "color.hover.background" => self.colors.hover_background,
            "color.focus.ring" => self.colors.focus_ring,
            "color.menu.background" => self.colors.menu_background,
            "color.menu.border" => self.colors.menu_border,
            "color.menu.item.hover" => self.colors.menu_item_hover,
            "color.menu.item.selected" => self.colors.menu_item_selected,
            "color.list.background" => self.colors.list_background,
            "color.list.border" => self.colors.list_border,
            "color.list.row.hover" => self.colors.list_row_hover,
            "color.list.row.selected" => self.colors.list_row_selected,
            "color.scrollbar.track" => self.colors.scrollbar_track,
            "color.scrollbar.thumb" => self.colors.scrollbar_thumb,
            "color.scrollbar.thumb.hover" => self.colors.scrollbar_thumb_hover,
            "color.viewport.selection.fill" => self.colors.viewport_selection_fill,
            "color.viewport.selection.stroke" => self.colors.viewport_selection_stroke,
            "color.viewport.marker" => self.colors.viewport_marker,
            "color.viewport.drag_line.pan" => self.colors.viewport_drag_line_pan,
            "color.viewport.drag_line.orbit" => self.colors.viewport_drag_line_orbit,
            "color.viewport.gizmo.x" => self.colors.viewport_gizmo_x,
            "color.viewport.gizmo.y" => self.colors.viewport_gizmo_y,
            "color.viewport.gizmo.handle.background" => {
                self.colors.viewport_gizmo_handle_background
            }
            "color.viewport.gizmo.handle.border" => self.colors.viewport_gizmo_handle_border,
            "color.viewport.rotate_gizmo" => self.colors.viewport_rotate_gizmo,

            "background" => self.colors.surface_background,
            "foreground" => self.colors.text_primary,

            "card" => self.colors.panel_background,
            "card-foreground" => self.colors.text_primary,

            "popover" => self.colors.menu_background,
            "popover-foreground" => self.colors.text_primary,
            "popover.border" => self.colors.menu_border,

            "border" => self.colors.panel_border,
            "input" => self.colors.panel_border,
            "ring" => self.colors.focus_ring,
            "ring-offset-background" => self.colors.surface_background,

            "muted" => self.colors.panel_background,
            "muted-foreground" => self.colors.text_muted,

            "accent" => self.colors.hover_background,
            "accent-foreground" => self.colors.text_primary,

            "primary" => self.colors.accent,
            "primary-foreground" => self.colors.text_primary,

            "secondary" => self.colors.panel_background,
            "secondary-foreground" => self.colors.text_primary,

            "destructive" => self.colors.viewport_gizmo_x,
            "destructive-foreground" => self.colors.text_primary,

            "selection.background" => self.colors.selection_background,

            "scrollbar.background" => self.colors.scrollbar_track,
            "scrollbar.thumb.background" => self.colors.scrollbar_thumb,
            "scrollbar.thumb.hover.background" => self.colors.scrollbar_thumb_hover,

            "fret.menu.background" => self.colors.menu_background,
            "fret.menu.border" => self.colors.menu_border,
            "fret.menu.item.hover" => self.colors.menu_item_hover,
            "fret.menu.item.selected" => self.colors.menu_item_selected,

            "fret.list.background" => self.colors.list_background,
            "fret.list.border" => self.colors.list_border,
            "fret.list.row.hover" => self.colors.list_row_hover,
            "fret.list.row.selected" => self.colors.list_row_selected,

            _ => return None,
        };
        Some(c)
    }

    pub fn color_required(&self, key: &str) -> Color {
        self.color_by_key(key)
            .unwrap_or_else(|| panic!("missing theme color token {key}"))
    }

    pub fn metric_by_key(&self, key: &str) -> Option<Px> {
        let key = canonicalize_token_key(ThemeTokenKind::Metric, key);
        let px = match key {
            "metric.radius.sm" => self.metrics.radius_sm,
            "metric.radius.md" => self.metrics.radius_md,
            "metric.radius.lg" => self.metrics.radius_lg,
            "metric.padding.sm" => self.metrics.padding_sm,
            "metric.padding.md" => self.metrics.padding_md,
            "metric.size.sm" => Px(32.0),
            "metric.size.md" => Px(36.0),
            "metric.size.lg" => Px(40.0),
            "metric.scrollbar.width" => self.metrics.scrollbar_width,
            "metric.font.size" => self.metrics.font_size,
            "metric.font.line_height" => self.metrics.font_line_height,
            "metric.font.mono_size" => self.metrics.mono_font_size,
            "metric.font.mono_line_height" => self.metrics.mono_font_line_height,

            "radius" => self.metrics.radius_sm,
            "radius.lg" => self.metrics.radius_md,

            "font.size" => self.metrics.font_size,
            "font.line_height" => self.metrics.font_line_height,
            "mono_font.size" => self.metrics.mono_font_size,
            "mono_font.line_height" => self.metrics.mono_font_line_height,

            _ => return None,
        };
        Some(px)
    }

    pub fn metric_required(&self, key: &str) -> Px {
        self.metric_by_key(key)
            .unwrap_or_else(|| panic!("missing theme metric token {key}"))
    }
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
    extra_corners: HashMap<String, Corners>,
    extra_numbers: HashMap<String, f32>,
    extra_durations_ms: HashMap<String, u32>,
    extra_easings: HashMap<String, CubicBezier>,
    extra_text_styles: HashMap<String, TextStyle>,
    configured_colors: HashSet<String>,
    configured_metrics: HashSet<String>,
    configured_corners: HashSet<String>,
    configured_numbers: HashSet<String>,
    configured_durations_ms: HashSet<String>,
    configured_easings: HashSet<String>,
    configured_text_styles: HashSet<String>,
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
        let key = canonicalize_token_key(ThemeTokenKind::Color, key);
        self.extra_colors
            .get(key)
            .copied()
            .or_else(|| self.snapshot().color_by_key(key))
    }

    pub fn color_required(&self, key: &str) -> Color {
        self.color_by_key(key)
            .unwrap_or_else(|| panic!("missing theme color token {key}"))
    }

    pub fn metric_by_key(&self, key: &str) -> Option<Px> {
        let key = canonicalize_token_key(ThemeTokenKind::Metric, key);
        self.extra_metrics
            .get(key)
            .copied()
            .or_else(|| self.snapshot().metric_by_key(key))
    }

    pub fn metric_required(&self, key: &str) -> Px {
        self.metric_by_key(key)
            .unwrap_or_else(|| panic!("missing theme metric token {key}"))
    }

    pub fn corners_by_key(&self, key: &str) -> Option<Corners> {
        let key = canonicalize_token_key(ThemeTokenKind::Corners, key);
        self.extra_corners
            .get(key)
            .copied()
            .or_else(|| self.metric_by_key(key).map(Corners::all))
    }

    pub fn corners_required(&self, key: &str) -> Corners {
        self.corners_by_key(key)
            .unwrap_or_else(|| panic!("missing theme corners token {key}"))
    }

    pub fn number_by_key(&self, key: &str) -> Option<f32> {
        let key = canonicalize_token_key(ThemeTokenKind::Number, key);
        self.extra_numbers.get(key).copied()
    }

    pub fn number_required(&self, key: &str) -> f32 {
        self.number_by_key(key)
            .unwrap_or_else(|| panic!("missing theme number token {key}"))
    }

    pub fn duration_ms_by_key(&self, key: &str) -> Option<u32> {
        let key = canonicalize_token_key(ThemeTokenKind::DurationMs, key);
        self.extra_durations_ms.get(key).copied()
    }

    pub fn duration_ms_required(&self, key: &str) -> u32 {
        self.duration_ms_by_key(key)
            .unwrap_or_else(|| panic!("missing theme duration_ms token {key}"))
    }

    pub fn easing_by_key(&self, key: &str) -> Option<CubicBezier> {
        let key = canonicalize_token_key(ThemeTokenKind::Easing, key);
        self.extra_easings.get(key).copied()
    }

    pub fn easing_required(&self, key: &str) -> CubicBezier {
        self.easing_by_key(key)
            .unwrap_or_else(|| panic!("missing theme easing token {key}"))
    }

    pub fn text_style_by_key(&self, key: &str) -> Option<TextStyle> {
        let key = canonicalize_token_key(ThemeTokenKind::TextStyle, key);
        self.extra_text_styles.get(key).cloned()
    }

    pub fn text_style_required(&self, key: &str) -> TextStyle {
        self.text_style_by_key(key)
            .unwrap_or_else(|| panic!("missing theme text_style token {key}"))
    }

    pub fn color_key_configured(&self, key: &str) -> bool {
        self.configured_colors.contains(key.trim())
    }

    pub fn metric_key_configured(&self, key: &str) -> bool {
        self.configured_metrics.contains(key.trim())
    }

    pub fn corners_key_configured(&self, key: &str) -> bool {
        self.configured_corners.contains(key.trim())
    }

    pub fn number_key_configured(&self, key: &str) -> bool {
        self.configured_numbers.contains(key.trim())
    }

    pub fn duration_ms_key_configured(&self, key: &str) -> bool {
        self.configured_durations_ms.contains(key.trim())
    }

    pub fn easing_key_configured(&self, key: &str) -> bool {
        self.configured_easings.contains(key.trim())
    }

    pub fn text_style_key_configured(&self, key: &str) -> bool {
        self.configured_text_styles.contains(key.trim())
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
        let mut next_numbers = HashMap::new();
        let mut next_durations_ms = HashMap::new();
        let mut next_easings = HashMap::new();
        let mut next_text_styles = HashMap::new();
        let mut next_corners = HashMap::new();

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
            if !cfg.colors.contains_key("fret.list.background")
                && !cfg.colors.contains_key("color.list.background")
                && self.colors.list_background != c
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
            if !cfg.colors.contains_key("fret.menu.item.hover")
                && !cfg.colors.contains_key("color.menu.item.hover")
                && self.colors.menu_item_hover != c
            {
                self.colors.menu_item_hover = c;
                changed = true;
            }
            if !cfg.colors.contains_key("fret.list.row.hover")
                && !cfg.colors.contains_key("color.list.row.hover")
                && self.colors.list_row_hover != c
            {
                self.colors.list_row_hover = c;
                changed = true;
            }
        });
        apply_semantic_color!("primary", |c| {
            if self.colors.accent != c {
                self.colors.accent = c;
                changed = true;
            }
            if !cfg.colors.contains_key("selection")
                && !cfg.colors.contains_key("selection.background")
                && !cfg.colors.contains_key("color.selection.background")
            {
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

        // Ensure the semantic keys remain present and mirror the resolved typed baseline.
        next_colors.insert("background".to_string(), self.colors.surface_background);
        next_colors.insert("foreground".to_string(), self.colors.text_primary);
        next_colors.insert("border".to_string(), self.colors.panel_border);
        next_colors.insert("input".to_string(), self.colors.panel_border);
        next_colors.insert("ring".to_string(), self.colors.focus_ring);
        next_colors.insert(
            "ring-offset-background".to_string(),
            self.colors.surface_background,
        );
        next_colors.insert("card".to_string(), self.colors.panel_background);
        next_colors.insert("card-foreground".to_string(), self.colors.text_primary);
        next_colors.insert("popover".to_string(), self.colors.menu_background);
        next_colors.insert("popover-foreground".to_string(), self.colors.text_primary);
        next_colors.insert("muted".to_string(), self.colors.panel_background);
        next_colors.insert("muted-foreground".to_string(), self.colors.text_muted);
        next_colors.insert("accent".to_string(), self.colors.hover_background);
        next_colors.insert("accent-foreground".to_string(), self.colors.text_primary);
        next_colors.insert("primary".to_string(), self.colors.accent);
        next_colors.insert("primary-foreground".to_string(), self.colors.text_primary);
        next_colors.insert("secondary".to_string(), self.colors.panel_background);
        next_colors.insert("secondary-foreground".to_string(), self.colors.text_primary);
        next_colors.insert("destructive".to_string(), self.colors.viewport_gizmo_x);
        next_colors.insert(
            "destructive-foreground".to_string(),
            self.colors.text_primary,
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

        // Ensure the semantic metric keys remain present and mirror the resolved typed baseline.
        next_metrics.insert("radius".to_string(), self.metrics.radius_lg);
        next_metrics.insert("radius.sm".to_string(), self.metrics.radius_sm);
        next_metrics.insert("radius.md".to_string(), self.metrics.radius_md);
        next_metrics.insert("radius.lg".to_string(), self.metrics.radius_lg);
        next_metrics.insert("font.size".to_string(), self.metrics.font_size);
        next_metrics.insert("mono_font.size".to_string(), self.metrics.mono_font_size);
        next_metrics.insert(
            "font.line_height".to_string(),
            self.metrics.font_line_height,
        );
        next_metrics.insert(
            "mono_font.line_height".to_string(),
            self.metrics.mono_font_line_height,
        );

        for (k, v) in &cfg.colors {
            if let Some(c) = parse_color_to_linear(v) {
                next_colors.insert(k.clone(), c);
            }
        }

        for (k, v) in &cfg.metrics {
            next_metrics.insert(k.clone(), Px(*v));
        }

        for (k, v) in &cfg.numbers {
            next_numbers.insert(k.clone(), *v);
        }

        for (k, v) in &cfg.durations_ms {
            next_durations_ms.insert(k.clone(), *v);
        }

        for (k, v) in &cfg.easings {
            next_easings.insert(k.clone(), *v);
        }

        for (k, v) in &cfg.text_styles {
            next_text_styles.insert(k.clone(), v.clone());
        }

        for (k, v) in &cfg.corners {
            next_corners.insert(k.clone(), *v);
        }

        let next_configured_colors: HashSet<String> = cfg.colors.keys().cloned().collect();
        if self.configured_colors != next_configured_colors {
            self.configured_colors = next_configured_colors;
            changed = true;
        }
        let next_configured_metrics: HashSet<String> = cfg.metrics.keys().cloned().collect();
        if self.configured_metrics != next_configured_metrics {
            self.configured_metrics = next_configured_metrics;
            changed = true;
        }
        let next_configured_corners: HashSet<String> = cfg.corners.keys().cloned().collect();
        if self.configured_corners != next_configured_corners {
            self.configured_corners = next_configured_corners;
            changed = true;
        }

        if self.extra_colors != next_colors {
            self.extra_colors = next_colors;
            changed = true;
        }
        if self.extra_metrics != next_metrics {
            self.extra_metrics = next_metrics;
            changed = true;
        }
        if self.extra_corners != next_corners {
            self.extra_corners = next_corners;
            changed = true;
        }

        let next_configured_numbers: HashSet<String> = cfg.numbers.keys().cloned().collect();
        if self.configured_numbers != next_configured_numbers {
            self.configured_numbers = next_configured_numbers;
            changed = true;
        }

        let next_configured_durations_ms: HashSet<String> =
            cfg.durations_ms.keys().cloned().collect();
        if self.configured_durations_ms != next_configured_durations_ms {
            self.configured_durations_ms = next_configured_durations_ms;
            changed = true;
        }

        let next_configured_easings: HashSet<String> = cfg.easings.keys().cloned().collect();
        if self.configured_easings != next_configured_easings {
            self.configured_easings = next_configured_easings;
            changed = true;
        }

        let next_configured_text_styles: HashSet<String> =
            cfg.text_styles.keys().cloned().collect();
        if self.configured_text_styles != next_configured_text_styles {
            self.configured_text_styles = next_configured_text_styles;
            changed = true;
        }

        if self.extra_numbers != next_numbers {
            self.extra_numbers = next_numbers;
            changed = true;
        }
        if self.extra_durations_ms != next_durations_ms {
            self.extra_durations_ms = next_durations_ms;
            changed = true;
        }
        if self.extra_easings != next_easings {
            self.extra_easings = next_easings;
            changed = true;
        }
        if self.extra_text_styles != next_text_styles {
            self.extra_text_styles = next_text_styles;
            changed = true;
        }

        if changed {
            self.revision = self.revision.saturating_add(1);
        }
    }

    /// Merge additional tokens from a `ThemeConfig` into the current theme without resetting the
    /// baseline theme (colors/metrics) or the configured-key tracking sets.
    ///
    /// This is intended for ecosystem-driven design system presets (e.g. Material 3) that need to
    /// inject extra token kinds (motion/state/typography) on top of an existing theme preset
    /// (e.g. a shadcn color scheme in the gallery app).
    pub fn extend_tokens_from_config(&mut self, cfg: &ThemeConfig) {
        let mut changed = false;

        for (k, v) in &cfg.colors {
            let key = k.trim();
            if key.is_empty() {
                continue;
            }
            if let Some(c) = parse_color_to_linear(v) {
                match self.extra_colors.get(key).copied() {
                    Some(prev) if prev == c => {}
                    _ => {
                        self.extra_colors.insert(key.to_string(), c);
                        changed = true;
                    }
                }
            }
        }

        for (k, v) in &cfg.metrics {
            let key = k.trim();
            if key.is_empty() {
                continue;
            }
            let px = Px(*v);
            match self.extra_metrics.get(key).copied() {
                Some(prev) if prev == px => {}
                _ => {
                    self.extra_metrics.insert(key.to_string(), px);
                    changed = true;
                }
            }
        }

        for (k, v) in &cfg.corners {
            let key = k.trim();
            if key.is_empty() {
                continue;
            }
            match self.extra_corners.get(key).copied() {
                Some(prev) if prev == *v => {}
                _ => {
                    self.extra_corners.insert(key.to_string(), *v);
                    changed = true;
                }
            }
        }

        for (k, v) in &cfg.numbers {
            let key = k.trim();
            if key.is_empty() {
                continue;
            }
            match self.extra_numbers.get(key).copied() {
                Some(prev) if (prev - *v).abs() < 1e-6 => {}
                _ => {
                    self.extra_numbers.insert(key.to_string(), *v);
                    changed = true;
                }
            }
        }

        for (k, v) in &cfg.durations_ms {
            let key = k.trim();
            if key.is_empty() {
                continue;
            }
            match self.extra_durations_ms.get(key).copied() {
                Some(prev) if prev == *v => {}
                _ => {
                    self.extra_durations_ms.insert(key.to_string(), *v);
                    changed = true;
                }
            }
        }

        for (k, v) in &cfg.easings {
            let key = k.trim();
            if key.is_empty() {
                continue;
            }
            match self.extra_easings.get(key).copied() {
                Some(prev) if prev == *v => {}
                _ => {
                    self.extra_easings.insert(key.to_string(), *v);
                    changed = true;
                }
            }
        }

        for (k, v) in &cfg.text_styles {
            let key = k.trim();
            if key.is_empty() {
                continue;
            }
            match self.extra_text_styles.get(key) {
                Some(prev) if prev == v => {}
                _ => {
                    self.extra_text_styles.insert(key.to_string(), v.clone());
                    changed = true;
                }
            }
        }

        if changed {
            self.revision = self.revision.saturating_add(1);
        }
    }

    /// Apply a `ThemeConfig` as a patch layered on top of the current theme.
    ///
    /// Unlike [`Self::apply_config`], this does **not** reset any existing token tables. This is
    /// the intended API for app-level overrides (e.g. "compact editor" metric tweaks) layered on
    /// top of an ecosystem preset (e.g. shadcn New York).
    ///
    /// This updates the `configured_*` tracking sets by adding the keys present in `cfg` and
    /// increments the theme revision when the effective token tables change.
    pub fn apply_config_patch(&mut self, cfg: &ThemeConfig) {
        assert_no_legacy_theme_keys(cfg);

        let mut changed = false;

        fn key_trim(k: &str) -> Option<&str> {
            let k = k.trim();
            (!k.is_empty()).then_some(k)
        }

        for (k, v) in &cfg.colors {
            let Some(key) = key_trim(k) else {
                continue;
            };
            self.configured_colors.insert(key.to_string());
            let Some(c) = parse_color_to_linear(v) else {
                continue;
            };
            match self.extra_colors.get(key).copied() {
                Some(prev) if prev == c => {}
                _ => {
                    self.extra_colors.insert(key.to_string(), c);
                    changed = true;
                }
            }
        }

        for (k, v) in &cfg.metrics {
            let Some(key) = key_trim(k) else {
                continue;
            };
            self.configured_metrics.insert(key.to_string());
            let px = Px(*v);
            match self.extra_metrics.get(key).copied() {
                Some(prev) if prev == px => {}
                _ => {
                    self.extra_metrics.insert(key.to_string(), px);
                    changed = true;
                }
            }
        }

        for (k, v) in &cfg.corners {
            let Some(key) = key_trim(k) else {
                continue;
            };
            self.configured_corners.insert(key.to_string());
            match self.extra_corners.get(key).copied() {
                Some(prev) if prev == *v => {}
                _ => {
                    self.extra_corners.insert(key.to_string(), *v);
                    changed = true;
                }
            }
        }

        for (k, v) in &cfg.numbers {
            let Some(key) = key_trim(k) else {
                continue;
            };
            self.configured_numbers.insert(key.to_string());
            match self.extra_numbers.get(key).copied() {
                Some(prev) if (prev - *v).abs() < 1e-6 => {}
                _ => {
                    self.extra_numbers.insert(key.to_string(), *v);
                    changed = true;
                }
            }
        }

        for (k, v) in &cfg.durations_ms {
            let Some(key) = key_trim(k) else {
                continue;
            };
            self.configured_durations_ms.insert(key.to_string());
            match self.extra_durations_ms.get(key).copied() {
                Some(prev) if prev == *v => {}
                _ => {
                    self.extra_durations_ms.insert(key.to_string(), *v);
                    changed = true;
                }
            }
        }

        for (k, v) in &cfg.easings {
            let Some(key) = key_trim(k) else {
                continue;
            };
            self.configured_easings.insert(key.to_string());
            match self.extra_easings.get(key).copied() {
                Some(prev) if prev == *v => {}
                _ => {
                    self.extra_easings.insert(key.to_string(), *v);
                    changed = true;
                }
            }
        }

        for (k, v) in &cfg.text_styles {
            let Some(key) = key_trim(k) else {
                continue;
            };
            self.configured_text_styles.insert(key.to_string());
            match self.extra_text_styles.get(key) {
                Some(prev) if prev == v => {}
                _ => {
                    self.extra_text_styles.insert(key.to_string(), v.clone());
                    changed = true;
                }
            }
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
            extra_corners: HashMap::new(),
            extra_numbers: HashMap::new(),
            extra_durations_ms: HashMap::new(),
            extra_easings: HashMap::new(),
            extra_text_styles: HashMap::new(),
            configured_colors: HashSet::new(),
            configured_metrics: HashSet::new(),
            configured_corners: HashSet::new(),
            configured_numbers: HashSet::new(),
            configured_durations_ms: HashSet::new(),
            configured_easings: HashSet::new(),
            configured_text_styles: HashSet::new(),
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

#[allow(clippy::excessive_precision)]
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

fn with_alpha(mut color: Color, alpha: f32) -> Color {
    color.a = alpha;
    color
}

fn assert_no_legacy_theme_keys(_cfg: &ThemeConfig) {
    // TODO: enforce/diagnose legacy keys once the theme config migration settles.
}

#[cfg(test)]
mod tests {
    use super::parse_color_to_linear;
    use super::{CubicBezier, Theme, ThemeConfig};
    use crate::{ThemeColorKey, ThemeMetricKey};
    use fret_core::{Corners, FontId, FontWeight, Px, TextSlant, TextStyle};
    use std::collections::HashMap;

    #[test]
    fn shadcn_semantic_palette_aliases_exist_on_default_theme() {
        let host = crate::test_host::TestHost::default();
        let theme = Theme::global(&host);

        for key in [
            "background",
            "foreground",
            "border",
            "input",
            "input.border",
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
            "muted-foreground",
            "input.background",
            "input.foreground",
            "accent",
            "accent-foreground",
            "popover.background",
            "popover.foreground",
            "popover-foreground",
            "popover.border",
        ] {
            assert!(theme.color_by_key(key).is_some(), "missing alias {key}");
        }
    }

    #[test]
    fn shadcn_legacy_size_metrics_exist_on_default_theme() {
        let host = crate::test_host::TestHost::default();
        let theme = Theme::global(&host);

        for key in ["metric.size.sm", "metric.size.md", "metric.size.lg"] {
            assert!(theme.metric_by_key(key).is_some(), "missing metric {key}");
        }
    }

    #[test]
    fn shadcn_legacy_size_metrics_exist_on_default_snapshot() {
        let host = crate::test_host::TestHost::default();
        let theme = Theme::global(&host);
        let snap = theme.snapshot();

        for key in ["metric.size.sm", "metric.size.md", "metric.size.lg"] {
            assert!(
                snap.metric_by_key(key).is_some(),
                "missing snapshot metric {key}"
            );
        }
    }

    #[test]
    fn shadcn_component_text_metrics_exist_on_default_theme() {
        let host = crate::test_host::TestHost::default();
        let theme = Theme::global(&host);

        for key in [
            "component.text.sm_px",
            "component.text.sm_line_height",
            "component.text.base_px",
            "component.text.base_line_height",
        ] {
            assert!(theme.metric_by_key(key).is_some(), "missing metric {key}");
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

        // No `color.*` keys are provided; typed fields should still change.
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
    fn theme_config_v2_parses_additional_token_kinds() {
        let cfg = ThemeConfig::from_slice(
            br#"{
  "name": "md3",
  "numbers": { "md.sys.state.hover.state-layer-opacity": 0.08 },
  "durations_ms": { "md.sys.motion.duration.short3": 150 },
  "easings": { "md.sys.motion.easing.emphasized.accelerate": { "x1": 0.3, "y1": 0.0, "x2": 0.8, "y2": 0.15 } },
  "corners": { "md.sys.shape.corner.extra-small.top": { "top_left": 4, "top_right": 4, "bottom_right": 0, "bottom_left": 0 } },
  "text_styles": {
    "md.sys.typescale.body-medium": { "font": "ui", "size": 14, "weight": 400, "slant": "normal" }
  }
}"#,
        )
        .expect("valid theme config");

        assert_eq!(cfg.name, "md3");
        assert_eq!(
            cfg.numbers
                .get("md.sys.state.hover.state-layer-opacity")
                .copied(),
            Some(0.08)
        );
        assert_eq!(
            cfg.durations_ms
                .get("md.sys.motion.duration.short3")
                .copied(),
            Some(150)
        );
        assert_eq!(
            cfg.easings
                .get("md.sys.motion.easing.emphasized.accelerate")
                .copied(),
            Some(CubicBezier {
                x1: 0.3,
                y1: 0.0,
                x2: 0.8,
                y2: 0.15
            })
        );
        assert_eq!(
            cfg.corners
                .get("md.sys.shape.corner.extra-small.top")
                .copied(),
            Some(Corners {
                top_left: Px(4.0),
                top_right: Px(4.0),
                bottom_right: Px(0.0),
                bottom_left: Px(0.0),
            })
        );
        assert!(cfg.text_styles.contains_key("md.sys.typescale.body-medium"));
    }

    #[test]
    fn theme_apply_config_updates_extended_token_maps_and_revision() {
        let mut theme = Theme::global(&crate::test_host::TestHost::default()).clone();
        let before = theme.revision();

        theme.apply_config(&ThemeConfig {
            name: "md3".to_string(),
            corners: HashMap::from([(
                "c".to_string(),
                Corners {
                    top_left: Px(1.0),
                    top_right: Px(2.0),
                    bottom_right: Px(3.0),
                    bottom_left: Px(4.0),
                },
            )]),
            numbers: HashMap::from([("n".to_string(), 1.25)]),
            durations_ms: HashMap::from([("d".to_string(), 120)]),
            easings: HashMap::from([(
                "e".to_string(),
                CubicBezier {
                    x1: 0.2,
                    y1: 0.0,
                    x2: 0.0,
                    y2: 1.0,
                },
            )]),
            text_styles: HashMap::from([(
                "t".to_string(),
                TextStyle {
                    font: FontId::ui(),
                    size: Px(14.0),
                    weight: FontWeight::NORMAL,
                    slant: TextSlant::Normal,
                    line_height: Some(Px(20.0)),
                    letter_spacing_em: None,
                },
            )]),
            ..ThemeConfig::default()
        });

        assert_eq!(
            theme.corners_by_key("c"),
            Some(Corners {
                top_left: Px(1.0),
                top_right: Px(2.0),
                bottom_right: Px(3.0),
                bottom_left: Px(4.0),
            })
        );
        assert_eq!(theme.number_by_key("n"), Some(1.25));
        assert_eq!(theme.duration_ms_by_key("d"), Some(120));
        assert_eq!(
            theme.easing_by_key("e"),
            Some(CubicBezier {
                x1: 0.2,
                y1: 0.0,
                x2: 0.0,
                y2: 1.0
            })
        );
        assert!(theme.text_style_by_key("t").is_some());

        assert!(theme.corners_key_configured("c"));
        assert!(theme.number_key_configured("n"));
        assert!(theme.duration_ms_key_configured("d"));
        assert!(theme.easing_key_configured("e"));
        assert!(theme.text_style_key_configured("t"));

        assert!(theme.revision() > before);
    }

    #[test]
    fn extend_tokens_from_config_preserves_configured_sets() {
        let mut theme = Theme::global(&crate::test_host::TestHost::default()).clone();

        theme.apply_config(&ThemeConfig {
            name: "Base".to_string(),
            metrics: HashMap::from([("metric.radius.sm".to_string(), 11.0)]),
            corners: HashMap::from([(
                "base.corners".to_string(),
                Corners {
                    top_left: Px(1.0),
                    top_right: Px(1.0),
                    bottom_right: Px(1.0),
                    bottom_left: Px(1.0),
                },
            )]),
            ..ThemeConfig::default()
        });
        assert!(theme.metric_key_configured("metric.radius.sm"));
        assert!(theme.corners_key_configured("base.corners"));

        let before = theme.revision();
        theme.extend_tokens_from_config(&ThemeConfig {
            name: "Extras".to_string(),
            metrics: HashMap::from([("md.sys.shape.corner.full".to_string(), 9999.0)]),
            corners: HashMap::from([(
                "md.sys.shape.corner.extra-small.top".to_string(),
                Corners {
                    top_left: Px(4.0),
                    top_right: Px(4.0),
                    bottom_right: Px(0.0),
                    bottom_left: Px(0.0),
                },
            )]),
            numbers: HashMap::from([("md.sys.state.hover.state-layer-opacity".to_string(), 0.08)]),
            ..ThemeConfig::default()
        });

        assert!(theme.metric_key_configured("metric.radius.sm"));
        assert!(theme.corners_key_configured("base.corners"));
        assert_eq!(
            theme.metric_by_key("md.sys.shape.corner.full"),
            Some(Px(9999.0))
        );
        assert!(
            theme
                .corners_by_key("md.sys.shape.corner.extra-small.top")
                .is_some()
        );
        assert_eq!(
            theme.number_by_key("md.sys.state.hover.state-layer-opacity"),
            Some(0.08)
        );
        assert!(theme.revision() > before);
    }

    #[test]
    fn apply_config_patch_preserves_existing_extra_colors() {
        let mut theme = Theme::global(&crate::test_host::TestHost::default()).clone();

        theme.extend_tokens_from_config(&ThemeConfig {
            name: "Base".to_string(),
            colors: HashMap::from([("primary".to_string(), "#ff0000".to_string())]),
            ..ThemeConfig::default()
        });
        assert_eq!(
            theme.color_by_key("primary"),
            Some(fret_core::Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            })
        );

        theme.apply_config_patch(&ThemeConfig {
            name: "Patch".to_string(),
            metrics: HashMap::from([("metric.padding.sm".to_string(), 7.0)]),
            ..ThemeConfig::default()
        });

        assert_eq!(
            theme.color_by_key("primary"),
            Some(fret_core::Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }),
            "expected shadcn-style palette tokens to remain intact after metric patches"
        );
        assert_eq!(theme.metric_by_key("metric.padding.sm"), Some(Px(7.0)));
    }
}
