pub mod keys;
pub(crate) mod registry;

use fret_core::{Color, Corners, Px, TextStyle};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex, OnceLock},
};

use crate::UiHost;
use crate::theme_registry::{ThemeTokenKind, canonicalize_token_key};
use crate::{ThemeColorKey, ThemeMetricKey};

const FALLBACK_COLOR: Color = Color {
    r: 1.0,
    g: 0.0,
    b: 1.0,
    a: 1.0,
};

fn warn_invalid_default_theme_color_once(name: &str, value: &str) -> bool {
    static SEEN: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

    let seen = SEEN.get_or_init(|| Mutex::new(HashSet::new()));
    let mut seen = match seen.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let k = format!("{name}:{value}");
    if !seen.insert(k) {
        return false;
    }

    tracing::warn!(
        color_name = name,
        color_value = value,
        "invalid default theme color; using fallback"
    );
    true
}

fn parse_default_theme_hex_color(name: &str, value: &str) -> Color {
    match parse_hex_srgb_to_linear(value) {
        Some(color) => color,
        None => {
            if strict_theme_enabled() {
                panic!("invalid default theme color {name}: {value}");
            }
            warn_invalid_default_theme_color_once(name, value);
            FALLBACK_COLOR
        }
    }
}

fn fallback_easing() -> CubicBezier {
    CubicBezier {
        x1: 0.0,
        y1: 0.0,
        x2: 1.0,
        y2: 1.0,
    }
}

#[cfg(not(test))]
fn strict_theme_enabled() -> bool {
    static STRICT: OnceLock<bool> = OnceLock::new();
    *STRICT.get_or_init(fret_runtime::strict_runtime::strict_runtime_enabled_from_env)
}

#[cfg(test)]
thread_local! {
    static STRICT_THEME_OVERRIDE: std::cell::Cell<Option<bool>> =
        const { std::cell::Cell::new(None) };
}

#[cfg(test)]
fn strict_theme_enabled() -> bool {
    STRICT_THEME_OVERRIDE
        .with(|cell| cell.get())
        .unwrap_or_else(fret_runtime::strict_runtime::strict_runtime_enabled_from_env)
}

#[cfg(test)]
struct StrictThemeGuard(Option<bool>);

#[cfg(test)]
fn strict_theme_for_tests(value: bool) -> StrictThemeGuard {
    let prev = STRICT_THEME_OVERRIDE.with(|cell| {
        let prev = cell.get();
        cell.set(Some(value));
        prev
    });
    StrictThemeGuard(prev)
}

#[cfg(test)]
impl Drop for StrictThemeGuard {
    fn drop(&mut self) {
        STRICT_THEME_OVERRIDE.with(|cell| cell.set(self.0));
    }
}

fn warn_missing_theme_token_once(kind: ThemeTokenKind, key: &str) -> bool {
    static SEEN: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

    let canonical = canonicalize_token_key(kind, key);
    if canonical.is_empty() {
        return false;
    }

    let seen = SEEN.get_or_init(|| Mutex::new(HashSet::new()));
    let mut seen = match seen.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let k = format!("{kind:?}:{canonical}");
    if !seen.insert(k) {
        return false;
    }

    tracing::warn!(
        token_kind = ?kind,
        token_key = canonical,
        "missing theme token; using fallback"
    );
    true
}

fn fallback_color_by_key(key: &str) -> Color {
    default_theme().color_by_key(key).unwrap_or(FALLBACK_COLOR)
}

fn fallback_metric_by_key(key: &str) -> Px {
    default_theme().metric_by_key(key).unwrap_or(Px(0.0))
}

fn fallback_corners_by_key(key: &str) -> Corners {
    default_theme()
        .corners_by_key(key)
        .unwrap_or_else(|| Corners::all(Px(0.0)))
}

fn fallback_number_by_key(key: &str) -> f32 {
    default_theme().number_by_key(key).unwrap_or(0.0)
}

fn fallback_duration_ms_by_key(key: &str) -> u32 {
    default_theme().duration_ms_by_key(key).unwrap_or(0)
}

fn fallback_easing_by_key(key: &str) -> CubicBezier {
    default_theme()
        .easing_by_key(key)
        .unwrap_or_else(fallback_easing)
}

fn fallback_text_style_by_key(key: &str) -> TextStyle {
    default_theme().text_style_by_key(key).unwrap_or_default()
}

fn canonicalize_config_map<V: Clone>(
    kind: ThemeTokenKind,
    map: &HashMap<String, V>,
) -> HashMap<String, V> {
    let mut out: HashMap<String, V> = HashMap::new();

    // Keep results deterministic even when the input is a `HashMap` and multiple aliases map to
    // the same canonical key (config error, but we should behave consistently).
    let mut keys: Vec<&String> = map.keys().collect();
    keys.sort_by(|a, b| a.trim().cmp(b.trim()).then_with(|| a.cmp(b)));

    // First pass: keys already in canonical form win over aliases.
    for k in keys.iter().copied() {
        let trimmed = k.trim();
        if trimmed.is_empty() {
            continue;
        }
        let canon = canonicalize_token_key(kind, trimmed);
        if canon == trimmed {
            // `k` might include whitespace, but `canon` is derived from `trimmed`, so read the
            // original value and store under the canonical key.
            if let Some(v) = map.get(k) {
                out.insert(canon.to_string(), v.clone());
            }
        }
    }

    // Second pass: fill missing canonical keys from aliases.
    for k in keys.iter().copied() {
        let trimmed = k.trim();
        if trimmed.is_empty() {
            continue;
        }
        let canon = canonicalize_token_key(kind, trimmed);
        if canon == trimmed {
            continue;
        }
        if let Some(v) = map.get(k) {
            out.entry(canon.to_string()).or_insert_with(|| v.clone());
        }
    }

    out
}

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
            "color.selection.inactive.background".to_string(),
            colors.selection_inactive_background,
        ),
        (
            "color.selection.window_inactive.background".to_string(),
            colors.selection_window_inactive_background,
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

    // shadcn/new-york v4 extended palette.
    //
    // These are optional in upstream theme presets, but shadcn recipes and our chart ports expect
    // the keys to exist. Keep defaults stable (theme-overridable via JSON configs).
    out.insert(
        "chart-1".to_string(),
        parse_default_theme_hex_color("chart-1", "#93C5FD"),
    );
    out.insert(
        "chart-2".to_string(),
        parse_default_theme_hex_color("chart-2", "#3B82F6"),
    );
    out.insert(
        "chart-3".to_string(),
        parse_default_theme_hex_color("chart-3", "#2563EB"),
    );
    out.insert(
        "chart-4".to_string(),
        parse_default_theme_hex_color("chart-4", "#1D4ED8"),
    );
    out.insert(
        "chart-5".to_string(),
        parse_default_theme_hex_color("chart-5", "#1E40AF"),
    );

    out.insert("sidebar".to_string(), colors.panel_background);
    out.insert("sidebar-foreground".to_string(), colors.text_primary);
    out.insert("sidebar-primary".to_string(), colors.accent);
    out.insert(
        "sidebar-primary-foreground".to_string(),
        colors.text_primary,
    );
    out.insert("sidebar-accent".to_string(), colors.hover_background);
    out.insert("sidebar-accent-foreground".to_string(), colors.text_primary);
    out.insert("sidebar-border".to_string(), colors.panel_border);
    out.insert("sidebar-ring".to_string(), colors.focus_ring);

    // Common non-core semantic keys used by recipes (kept for the migration window).
    out.insert(
        "selection.background".to_string(),
        colors.selection_background,
    );
    out.insert(
        "selection.inactive.background".to_string(),
        colors.selection_inactive_background,
    );
    out.insert(
        "selection.window_inactive.background".to_string(),
        colors.selection_window_inactive_background,
    );
    out.insert("input.background".to_string(), colors.panel_background);
    out.insert("input.foreground".to_string(), colors.text_primary);
    out.insert("caret".to_string(), colors.text_primary);
    out.insert("scrollbar.background".to_string(), colors.scrollbar_track);
    // Historic/compat key used by some UI kit ports.
    out.insert(
        "scrollbar.track.background".to_string(),
        colors.scrollbar_track,
    );
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
        "component.text.xs_px".to_string(),
        Px((metrics.font_size.0 - 1.0).max(1.0)),
    );
    out.insert(
        "component.text.xs_line_height".to_string(),
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
    out.insert(
        "component.text.prose_px".to_string(),
        Px(metrics.font_size.0 + 3.0),
    );
    out.insert(
        "component.text.prose_line_height".to_string(),
        Px((metrics.font_line_height.0 + 8.0).max(metrics.font_size.0 + 10.0)),
    );

    // Common spacing and sizing tokens used by ecosystem widgets.
    out.insert("metric.gap.sm".to_string(), metrics.padding_sm);
    out.insert("component.size.sm.icon_button.size".to_string(), Px(32.0));
    out.insert("component.size.md.icon_button.size".to_string(), Px(36.0));
    out.insert("component.size.lg.icon_button.size".to_string(), Px(40.0));

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
    pub selection_inactive_background: Color,
    pub selection_window_inactive_background: Color,
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

#[derive(Debug, Clone)]
pub struct ThemeSnapshot {
    pub colors: ThemeColors,
    pub metrics: ThemeMetrics,
    pub revision: u64,
    color_tokens: Arc<HashMap<String, Color>>,
    metric_tokens: Arc<HashMap<String, Px>>,
}

impl ThemeSnapshot {
    pub fn from_baseline(colors: ThemeColors, metrics: ThemeMetrics, revision: u64) -> Self {
        Self {
            colors,
            metrics,
            revision,
            color_tokens: Arc::new(default_color_tokens(colors)),
            metric_tokens: Arc::new(default_metric_tokens(metrics)),
        }
    }

    pub fn color_by_key(&self, key: &str) -> Option<Color> {
        let key = canonicalize_token_key(ThemeTokenKind::Color, key);
        self.color_tokens.get(key).copied()
    }

    pub fn color_required(&self, key: &str) -> Color {
        if let Some(v) = self.color_by_key(key) {
            return v;
        }

        if strict_theme_enabled() {
            panic!("missing theme color token {key}");
        }
        warn_missing_theme_token_once(ThemeTokenKind::Color, key);
        fallback_color_by_key(key)
    }

    /// Non-panicking theme token access with diagnostics + fallback behavior.
    pub fn color_token(&self, key: &str) -> Color {
        self.color_required(key)
    }

    pub fn metric_by_key(&self, key: &str) -> Option<Px> {
        let key = canonicalize_token_key(ThemeTokenKind::Metric, key);
        self.metric_tokens.get(key).copied()
    }

    pub fn metric_required(&self, key: &str) -> Px {
        if let Some(v) = self.metric_by_key(key) {
            return v;
        }

        if strict_theme_enabled() {
            panic!("missing theme metric token {key}");
        }
        warn_missing_theme_token_once(ThemeTokenKind::Metric, key);
        fallback_metric_by_key(key)
    }

    /// Non-panicking theme token access with diagnostics + fallback behavior.
    pub fn metric_token(&self, key: &str) -> Px {
        self.metric_required(key)
    }
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub author: Option<String>,
    pub url: Option<String>,
    pub colors: ThemeColors,
    pub metrics: ThemeMetrics,
    extra_colors: Arc<HashMap<String, Color>>,
    extra_metrics: Arc<HashMap<String, Px>>,
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
        let name = key.canonical_name();
        if let Some(v) = self.color_by_key(name) {
            return v;
        }

        if strict_theme_enabled() {
            panic!("missing core theme color key {}", name);
        }
        warn_missing_theme_token_once(ThemeTokenKind::Color, name);
        fallback_color_by_key(name)
    }

    pub fn metric(&self, key: ThemeMetricKey) -> Px {
        let name = key.canonical_name();
        if let Some(v) = self.metric_by_key(name) {
            return v;
        }

        if strict_theme_enabled() {
            panic!("missing core theme metric key {}", name);
        }
        warn_missing_theme_token_once(ThemeTokenKind::Metric, name);
        fallback_metric_by_key(name)
    }

    pub fn color_by_key(&self, key: &str) -> Option<Color> {
        let key = canonicalize_token_key(ThemeTokenKind::Color, key);
        self.extra_colors.get(key).copied()
    }

    pub fn color_required(&self, key: &str) -> Color {
        if let Some(v) = self.color_by_key(key) {
            return v;
        }

        if strict_theme_enabled() {
            panic!("missing theme color token {key}");
        }
        warn_missing_theme_token_once(ThemeTokenKind::Color, key);
        fallback_color_by_key(key)
    }

    /// Non-panicking theme token access with diagnostics + fallback behavior.
    pub fn color_token(&self, key: &str) -> Color {
        self.color_required(key)
    }

    pub fn metric_by_key(&self, key: &str) -> Option<Px> {
        let key = canonicalize_token_key(ThemeTokenKind::Metric, key);
        self.extra_metrics.get(key).copied()
    }

    pub fn metric_required(&self, key: &str) -> Px {
        if let Some(v) = self.metric_by_key(key) {
            return v;
        }

        if strict_theme_enabled() {
            panic!("missing theme metric token {key}");
        }
        warn_missing_theme_token_once(ThemeTokenKind::Metric, key);
        fallback_metric_by_key(key)
    }

    /// Non-panicking theme token access with diagnostics + fallback behavior.
    pub fn metric_token(&self, key: &str) -> Px {
        self.metric_required(key)
    }

    pub fn corners_by_key(&self, key: &str) -> Option<Corners> {
        let key = canonicalize_token_key(ThemeTokenKind::Corners, key);
        self.extra_corners
            .get(key)
            .copied()
            .or_else(|| self.metric_by_key(key).map(Corners::all))
    }

    pub fn corners_required(&self, key: &str) -> Corners {
        if let Some(v) = self.corners_by_key(key) {
            return v;
        }

        if strict_theme_enabled() {
            panic!("missing theme corners token {key}");
        }
        warn_missing_theme_token_once(ThemeTokenKind::Corners, key);
        fallback_corners_by_key(key)
    }

    /// Non-panicking theme token access with diagnostics + fallback behavior.
    pub fn corners_token(&self, key: &str) -> Corners {
        self.corners_required(key)
    }

    pub fn number_by_key(&self, key: &str) -> Option<f32> {
        let key = canonicalize_token_key(ThemeTokenKind::Number, key);
        self.extra_numbers.get(key).copied()
    }

    pub fn number_required(&self, key: &str) -> f32 {
        if let Some(v) = self.number_by_key(key) {
            return v;
        }

        if strict_theme_enabled() {
            panic!("missing theme number token {key}");
        }
        warn_missing_theme_token_once(ThemeTokenKind::Number, key);
        fallback_number_by_key(key)
    }

    /// Non-panicking theme token access with diagnostics + fallback behavior.
    pub fn number_token(&self, key: &str) -> f32 {
        self.number_required(key)
    }

    pub fn duration_ms_by_key(&self, key: &str) -> Option<u32> {
        let key = canonicalize_token_key(ThemeTokenKind::DurationMs, key);
        self.extra_durations_ms.get(key).copied()
    }

    pub fn duration_ms_required(&self, key: &str) -> u32 {
        if let Some(v) = self.duration_ms_by_key(key) {
            return v;
        }

        if strict_theme_enabled() {
            panic!("missing theme duration_ms token {key}");
        }
        warn_missing_theme_token_once(ThemeTokenKind::DurationMs, key);
        fallback_duration_ms_by_key(key)
    }

    /// Non-panicking theme token access with diagnostics + fallback behavior.
    pub fn duration_ms_token(&self, key: &str) -> u32 {
        self.duration_ms_required(key)
    }

    pub fn easing_by_key(&self, key: &str) -> Option<CubicBezier> {
        let key = canonicalize_token_key(ThemeTokenKind::Easing, key);
        self.extra_easings.get(key).copied()
    }

    pub fn easing_required(&self, key: &str) -> CubicBezier {
        if let Some(v) = self.easing_by_key(key) {
            return v;
        }

        if strict_theme_enabled() {
            panic!("missing theme easing token {key}");
        }
        warn_missing_theme_token_once(ThemeTokenKind::Easing, key);
        fallback_easing_by_key(key)
    }

    /// Non-panicking theme token access with diagnostics + fallback behavior.
    pub fn easing_token(&self, key: &str) -> CubicBezier {
        self.easing_required(key)
    }

    pub fn text_style_by_key(&self, key: &str) -> Option<TextStyle> {
        let key = canonicalize_token_key(ThemeTokenKind::TextStyle, key);
        self.extra_text_styles.get(key).cloned()
    }

    pub fn text_style_required(&self, key: &str) -> TextStyle {
        if let Some(v) = self.text_style_by_key(key) {
            return v;
        }

        if strict_theme_enabled() {
            panic!("missing theme text_style token {key}");
        }
        warn_missing_theme_token_once(ThemeTokenKind::TextStyle, key);
        fallback_text_style_by_key(key)
    }

    /// Non-panicking theme token access with diagnostics + fallback behavior.
    pub fn text_style_token(&self, key: &str) -> TextStyle {
        self.text_style_required(key)
    }

    pub fn color_key_configured(&self, key: &str) -> bool {
        let key = canonicalize_token_key(ThemeTokenKind::Color, key);
        self.configured_colors.contains(key)
    }

    pub fn metric_key_configured(&self, key: &str) -> bool {
        let key = canonicalize_token_key(ThemeTokenKind::Metric, key);
        self.configured_metrics.contains(key)
    }

    pub fn corners_key_configured(&self, key: &str) -> bool {
        let key = canonicalize_token_key(ThemeTokenKind::Corners, key);
        self.configured_corners.contains(key)
    }

    pub fn number_key_configured(&self, key: &str) -> bool {
        let key = canonicalize_token_key(ThemeTokenKind::Number, key);
        self.configured_numbers.contains(key)
    }

    pub fn duration_ms_key_configured(&self, key: &str) -> bool {
        let key = canonicalize_token_key(ThemeTokenKind::DurationMs, key);
        self.configured_durations_ms.contains(key)
    }

    pub fn easing_key_configured(&self, key: &str) -> bool {
        let key = canonicalize_token_key(ThemeTokenKind::Easing, key);
        self.configured_easings.contains(key)
    }

    pub fn text_style_key_configured(&self, key: &str) -> bool {
        let key = canonicalize_token_key(ThemeTokenKind::TextStyle, key);
        self.configured_text_styles.contains(key)
    }

    pub fn snapshot(&self) -> ThemeSnapshot {
        ThemeSnapshot {
            colors: self.colors,
            metrics: self.metrics,
            revision: self.revision,
            color_tokens: self.extra_colors.clone(),
            metric_tokens: self.extra_metrics.clone(),
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

        let cfg_colors = canonicalize_config_map(ThemeTokenKind::Color, &cfg.colors);
        let cfg_metrics = canonicalize_config_map(ThemeTokenKind::Metric, &cfg.metrics);
        let cfg_corners = canonicalize_config_map(ThemeTokenKind::Corners, &cfg.corners);
        let cfg_numbers = canonicalize_config_map(ThemeTokenKind::Number, &cfg.numbers);
        let cfg_durations_ms =
            canonicalize_config_map(ThemeTokenKind::DurationMs, &cfg.durations_ms);
        let cfg_easings = canonicalize_config_map(ThemeTokenKind::Easing, &cfg.easings);
        let cfg_text_styles = canonicalize_config_map(ThemeTokenKind::TextStyle, &cfg.text_styles);

        let mut changed = false;

        let mut next_numbers = HashMap::new();
        let mut next_durations_ms = HashMap::new();
        let mut next_easings = HashMap::new();
        let mut next_text_styles = HashMap::new();
        let mut next_corners = HashMap::new();

        macro_rules! apply_semantic_color {
            ($key:literal, $set:expr) => {
                if let Some(v) = cfg_colors.get($key) {
                    if let Some(c) = parse_color_to_linear(v) {
                        $set(c);
                    }
                }
            };
        }

        macro_rules! apply_metric {
            ($key:literal, $field:expr) => {
                if let Some(v) = cfg_metrics.get($key).copied() {
                    let px = Px(v);
                    if $field != px {
                        $field = px;
                        changed = true;
                    }
                }
            };
        }

        macro_rules! apply_semantic_metric {
            ($key:literal, $set:expr) => {
                if let Some(v) = cfg_metrics.get($key).copied() {
                    let px = Px(v);
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
            if !cfg_colors.contains_key("border") {
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
            if !cfg_colors.contains_key("fret.list.background")
                && !cfg_colors.contains_key("color.list.background")
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
            if !cfg_colors.contains_key("fret.menu.item.hover")
                && !cfg_colors.contains_key("color.menu.item.hover")
                && self.colors.menu_item_hover != c
            {
                self.colors.menu_item_hover = c;
                changed = true;
            }
            if !cfg_colors.contains_key("fret.list.row.hover")
                && !cfg_colors.contains_key("color.list.row.hover")
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
            if !cfg_colors.contains_key("selection")
                && !cfg_colors.contains_key("selection.background")
                && !cfg_colors.contains_key("color.selection.background")
            {
                let selection = with_alpha(c, 0.4);
                if self.colors.selection_background != selection {
                    self.colors.selection_background = selection;
                    changed = true;
                }
            }
            if !cfg_colors.contains_key("selection.inactive.background")
                && !cfg_colors.contains_key("color.selection.inactive.background")
            {
                let inactive = with_alpha(c, 0.24);
                if self.colors.selection_inactive_background != inactive {
                    self.colors.selection_inactive_background = inactive;
                    changed = true;
                }
            }
            if !cfg_colors.contains_key("selection.window_inactive.background")
                && !cfg_colors.contains_key("color.selection.window_inactive.background")
            {
                let inactive = with_alpha(c, 0.16);
                if self.colors.selection_window_inactive_background != inactive {
                    self.colors.selection_window_inactive_background = inactive;
                    changed = true;
                }
            }
        });

        macro_rules! apply_baseline_color {
            ($key:literal, $field:expr) => {
                if let Some(v) = cfg_colors.get($key) {
                    if let Some(c) = parse_color_to_linear(v) {
                        if $field != c {
                            $field = c;
                            changed = true;
                        }
                    }
                }
            };
        }

        // Apply baseline dotted keys (ADR 0050) after semantic keys so explicit baseline tokens win.
        apply_baseline_color!("color.surface.background", self.colors.surface_background);
        apply_baseline_color!("color.panel.background", self.colors.panel_background);
        apply_baseline_color!("color.panel.border", self.colors.panel_border);
        apply_baseline_color!("color.text.primary", self.colors.text_primary);
        apply_baseline_color!("color.text.muted", self.colors.text_muted);
        apply_baseline_color!("color.text.disabled", self.colors.text_disabled);
        apply_baseline_color!("color.accent", self.colors.accent);
        if !cfg_colors.contains_key("color.selection.background") {
            apply_baseline_color!("selection.background", self.colors.selection_background);
        }
        if !cfg_colors.contains_key("color.selection.inactive.background") {
            apply_baseline_color!(
                "selection.inactive.background",
                self.colors.selection_inactive_background
            );
        }
        if !cfg_colors.contains_key("color.selection.window_inactive.background") {
            apply_baseline_color!(
                "selection.window_inactive.background",
                self.colors.selection_window_inactive_background
            );
        }
        apply_baseline_color!(
            "color.selection.background",
            self.colors.selection_background
        );
        apply_baseline_color!(
            "color.selection.inactive.background",
            self.colors.selection_inactive_background
        );
        apply_baseline_color!(
            "color.selection.window_inactive.background",
            self.colors.selection_window_inactive_background
        );
        apply_baseline_color!("color.hover.background", self.colors.hover_background);
        apply_baseline_color!("color.focus.ring", self.colors.focus_ring);
        apply_baseline_color!("color.menu.background", self.colors.menu_background);
        apply_baseline_color!("color.menu.border", self.colors.menu_border);
        apply_baseline_color!("color.menu.item.hover", self.colors.menu_item_hover);
        apply_baseline_color!("color.menu.item.selected", self.colors.menu_item_selected);
        apply_baseline_color!("color.list.background", self.colors.list_background);
        apply_baseline_color!("color.list.border", self.colors.list_border);
        apply_baseline_color!("color.list.row.hover", self.colors.list_row_hover);
        apply_baseline_color!("color.list.row.selected", self.colors.list_row_selected);
        apply_baseline_color!("color.scrollbar.track", self.colors.scrollbar_track);
        apply_baseline_color!("color.scrollbar.thumb", self.colors.scrollbar_thumb);
        apply_baseline_color!(
            "color.scrollbar.thumb.hover",
            self.colors.scrollbar_thumb_hover
        );
        apply_baseline_color!(
            "color.viewport.selection.fill",
            self.colors.viewport_selection_fill
        );
        apply_baseline_color!(
            "color.viewport.selection.stroke",
            self.colors.viewport_selection_stroke
        );
        apply_baseline_color!("color.viewport.marker", self.colors.viewport_marker);
        apply_baseline_color!(
            "color.viewport.drag_line.pan",
            self.colors.viewport_drag_line_pan
        );
        apply_baseline_color!(
            "color.viewport.drag_line.orbit",
            self.colors.viewport_drag_line_orbit
        );
        apply_baseline_color!("color.viewport.gizmo.x", self.colors.viewport_gizmo_x);
        apply_baseline_color!("color.viewport.gizmo.y", self.colors.viewport_gizmo_y);
        apply_baseline_color!(
            "color.viewport.gizmo.handle.background",
            self.colors.viewport_gizmo_handle_background
        );
        apply_baseline_color!(
            "color.viewport.gizmo.handle.border",
            self.colors.viewport_gizmo_handle_border
        );
        apply_baseline_color!(
            "color.viewport.rotate_gizmo",
            self.colors.viewport_rotate_gizmo
        );

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
        if !cfg_metrics.contains_key("metric.font.size")
            && let Some(v) = cfg_metrics.get("font.size").copied()
        {
            let px = Px(v);
            if self.metrics.font_size != px {
                self.metrics.font_size = px;
                changed = true;
            }
        }
        if !cfg_metrics.contains_key("metric.font.mono_size")
            && let Some(v) = cfg_metrics.get("mono_font.size").copied()
        {
            let px = Px(v);
            if self.metrics.mono_font_size != px {
                self.metrics.mono_font_size = px;
                changed = true;
            }
        }
        if !cfg_metrics.contains_key("metric.font.line_height")
            && let Some(v) = cfg_metrics.get("font.line_height").copied()
        {
            let px = Px(v);
            if self.metrics.font_line_height != px {
                self.metrics.font_line_height = px;
                changed = true;
            }
        }
        if !cfg_metrics.contains_key("metric.font.mono_line_height")
            && let Some(v) = cfg_metrics.get("mono_font.line_height").copied()
        {
            let px = Px(v);
            if self.metrics.mono_font_line_height != px {
                self.metrics.mono_font_line_height = px;
                changed = true;
            }
        }

        let mut next_colors = default_color_tokens(self.colors);
        let mut next_metrics = default_metric_tokens(self.metrics);

        for (k, v) in &cfg_colors {
            if let Some(c) = parse_color_to_linear(v) {
                next_colors.insert(k.clone(), c);
            }
        }

        for (k, v) in &cfg_metrics {
            next_metrics.insert(k.clone(), Px(*v));
        }

        for (k, v) in &cfg_numbers {
            next_numbers.insert(k.clone(), *v);
        }

        for (k, v) in &cfg_durations_ms {
            next_durations_ms.insert(k.clone(), *v);
        }

        for (k, v) in &cfg_easings {
            next_easings.insert(k.clone(), *v);
        }

        for (k, v) in &cfg_text_styles {
            next_text_styles.insert(k.clone(), v.clone());
        }

        for (k, v) in &cfg_corners {
            next_corners.insert(k.clone(), *v);
        }

        // Ensure baseline + semantic keys mirror the resolved typed baseline, even if the config
        // provided overlapping aliases.
        next_colors.insert(
            "color.surface.background".to_string(),
            self.colors.surface_background,
        );
        next_colors.insert(
            "color.panel.background".to_string(),
            self.colors.panel_background,
        );
        next_colors.insert("color.panel.border".to_string(), self.colors.panel_border);
        next_colors.insert("color.text.primary".to_string(), self.colors.text_primary);
        next_colors.insert("color.text.muted".to_string(), self.colors.text_muted);
        next_colors.insert("color.text.disabled".to_string(), self.colors.text_disabled);
        next_colors.insert("color.accent".to_string(), self.colors.accent);
        next_colors.insert(
            "color.selection.background".to_string(),
            self.colors.selection_background,
        );
        next_colors.insert(
            "color.selection.inactive.background".to_string(),
            self.colors.selection_inactive_background,
        );
        next_colors.insert(
            "color.selection.window_inactive.background".to_string(),
            self.colors.selection_window_inactive_background,
        );
        next_colors.insert(
            "color.hover.background".to_string(),
            self.colors.hover_background,
        );
        next_colors.insert("color.focus.ring".to_string(), self.colors.focus_ring);
        next_colors.insert(
            "color.menu.background".to_string(),
            self.colors.menu_background,
        );
        next_colors.insert("color.menu.border".to_string(), self.colors.menu_border);
        next_colors.insert(
            "color.menu.item.hover".to_string(),
            self.colors.menu_item_hover,
        );
        next_colors.insert(
            "color.menu.item.selected".to_string(),
            self.colors.menu_item_selected,
        );
        next_colors.insert(
            "color.list.background".to_string(),
            self.colors.list_background,
        );
        next_colors.insert("color.list.border".to_string(), self.colors.list_border);
        next_colors.insert(
            "color.list.row.hover".to_string(),
            self.colors.list_row_hover,
        );
        next_colors.insert(
            "color.list.row.selected".to_string(),
            self.colors.list_row_selected,
        );
        next_colors.insert(
            "color.scrollbar.track".to_string(),
            self.colors.scrollbar_track,
        );
        next_colors.insert(
            "color.scrollbar.thumb".to_string(),
            self.colors.scrollbar_thumb,
        );
        next_colors.insert(
            "color.scrollbar.thumb.hover".to_string(),
            self.colors.scrollbar_thumb_hover,
        );
        next_colors.insert(
            "color.viewport.selection.fill".to_string(),
            self.colors.viewport_selection_fill,
        );
        next_colors.insert(
            "color.viewport.selection.stroke".to_string(),
            self.colors.viewport_selection_stroke,
        );
        next_colors.insert(
            "color.viewport.marker".to_string(),
            self.colors.viewport_marker,
        );
        next_colors.insert(
            "color.viewport.drag_line.pan".to_string(),
            self.colors.viewport_drag_line_pan,
        );
        next_colors.insert(
            "color.viewport.drag_line.orbit".to_string(),
            self.colors.viewport_drag_line_orbit,
        );
        next_colors.insert(
            "color.viewport.gizmo.x".to_string(),
            self.colors.viewport_gizmo_x,
        );
        next_colors.insert(
            "color.viewport.gizmo.y".to_string(),
            self.colors.viewport_gizmo_y,
        );
        next_colors.insert(
            "color.viewport.gizmo.handle.background".to_string(),
            self.colors.viewport_gizmo_handle_background,
        );
        next_colors.insert(
            "color.viewport.gizmo.handle.border".to_string(),
            self.colors.viewport_gizmo_handle_border,
        );
        next_colors.insert(
            "color.viewport.rotate_gizmo".to_string(),
            self.colors.viewport_rotate_gizmo,
        );

        // Keep shadcn semantic aliases coherent with the resolved typed baseline fields.
        //
        // Important: do not overwrite config-provided tokens that do *not* map onto typed baseline
        // fields (e.g. `*-foreground`). Those are part of the shadcn token surface and must
        // remain author-controlled.
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
        next_colors.insert("popover".to_string(), self.colors.menu_background);
        next_colors.insert("muted-foreground".to_string(), self.colors.text_muted);
        next_colors.insert("accent".to_string(), self.colors.hover_background);
        next_colors.insert("primary".to_string(), self.colors.accent);

        next_metrics.insert("metric.radius.sm".to_string(), self.metrics.radius_sm);
        next_metrics.insert("metric.radius.md".to_string(), self.metrics.radius_md);
        next_metrics.insert("metric.radius.lg".to_string(), self.metrics.radius_lg);
        next_metrics.insert("metric.padding.sm".to_string(), self.metrics.padding_sm);
        next_metrics.insert("metric.padding.md".to_string(), self.metrics.padding_md);
        next_metrics.insert(
            "metric.scrollbar.width".to_string(),
            self.metrics.scrollbar_width,
        );
        next_metrics.insert("metric.font.size".to_string(), self.metrics.font_size);
        next_metrics.insert(
            "metric.font.mono_size".to_string(),
            self.metrics.mono_font_size,
        );
        next_metrics.insert(
            "metric.font.line_height".to_string(),
            self.metrics.font_line_height,
        );
        next_metrics.insert(
            "metric.font.mono_line_height".to_string(),
            self.metrics.mono_font_line_height,
        );

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

        let next_configured_colors: HashSet<String> = cfg_colors.keys().cloned().collect();
        if self.configured_colors != next_configured_colors {
            self.configured_colors = next_configured_colors;
            changed = true;
        }
        let next_configured_metrics: HashSet<String> = cfg_metrics.keys().cloned().collect();
        if self.configured_metrics != next_configured_metrics {
            self.configured_metrics = next_configured_metrics;
            changed = true;
        }
        let next_configured_corners: HashSet<String> = cfg_corners.keys().cloned().collect();
        if self.configured_corners != next_configured_corners {
            self.configured_corners = next_configured_corners;
            changed = true;
        }

        let next_colors = Arc::new(next_colors);
        let next_metrics = Arc::new(next_metrics);

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

        let next_configured_numbers: HashSet<String> = cfg_numbers.keys().cloned().collect();
        if self.configured_numbers != next_configured_numbers {
            self.configured_numbers = next_configured_numbers;
            changed = true;
        }

        let next_configured_durations_ms: HashSet<String> =
            cfg_durations_ms.keys().cloned().collect();
        if self.configured_durations_ms != next_configured_durations_ms {
            self.configured_durations_ms = next_configured_durations_ms;
            changed = true;
        }

        let next_configured_easings: HashSet<String> = cfg_easings.keys().cloned().collect();
        if self.configured_easings != next_configured_easings {
            self.configured_easings = next_configured_easings;
            changed = true;
        }

        let next_configured_text_styles: HashSet<String> =
            cfg_text_styles.keys().cloned().collect();
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

        let colors = canonicalize_config_map(ThemeTokenKind::Color, &cfg.colors);
        for (key, v) in &colors {
            if let Some(c) = parse_color_to_linear(v) {
                match self.extra_colors.get(key.as_str()).copied() {
                    Some(prev) if prev == c => {}
                    _ => {
                        Arc::make_mut(&mut self.extra_colors).insert(key.to_string(), c);
                        changed = true;
                    }
                }
            }
        }

        let metrics = canonicalize_config_map(ThemeTokenKind::Metric, &cfg.metrics);
        for (key, v) in &metrics {
            let px = Px(*v);
            match self.extra_metrics.get(key.as_str()).copied() {
                Some(prev) if prev == px => {}
                _ => {
                    Arc::make_mut(&mut self.extra_metrics).insert(key.to_string(), px);
                    changed = true;
                }
            }
        }

        let corners = canonicalize_config_map(ThemeTokenKind::Corners, &cfg.corners);
        for (key, v) in &corners {
            match self.extra_corners.get(key.as_str()).copied() {
                Some(prev) if prev == *v => {}
                _ => {
                    self.extra_corners.insert(key.to_string(), *v);
                    changed = true;
                }
            }
        }

        let numbers = canonicalize_config_map(ThemeTokenKind::Number, &cfg.numbers);
        for (key, v) in &numbers {
            match self.extra_numbers.get(key.as_str()).copied() {
                Some(prev) if (prev - *v).abs() < 1e-6 => {}
                _ => {
                    self.extra_numbers.insert(key.to_string(), *v);
                    changed = true;
                }
            }
        }

        let durations_ms = canonicalize_config_map(ThemeTokenKind::DurationMs, &cfg.durations_ms);
        for (key, v) in &durations_ms {
            match self.extra_durations_ms.get(key.as_str()).copied() {
                Some(prev) if prev == *v => {}
                _ => {
                    self.extra_durations_ms.insert(key.to_string(), *v);
                    changed = true;
                }
            }
        }

        let easings = canonicalize_config_map(ThemeTokenKind::Easing, &cfg.easings);
        for (key, v) in &easings {
            match self.extra_easings.get(key.as_str()).copied() {
                Some(prev) if prev == *v => {}
                _ => {
                    self.extra_easings.insert(key.to_string(), *v);
                    changed = true;
                }
            }
        }

        let text_styles = canonicalize_config_map(ThemeTokenKind::TextStyle, &cfg.text_styles);
        for (key, v) in &text_styles {
            match self.extra_text_styles.get(key.as_str()) {
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

        let colors = canonicalize_config_map(ThemeTokenKind::Color, &cfg.colors);
        for (key, v) in &colors {
            self.configured_colors.insert(key.to_string());
            let Some(c) = parse_color_to_linear(v) else {
                continue;
            };
            match self.extra_colors.get(key.as_str()).copied() {
                Some(prev) if prev == c => {}
                _ => {
                    Arc::make_mut(&mut self.extra_colors).insert(key.to_string(), c);
                    changed = true;
                }
            }
        }

        let metrics = canonicalize_config_map(ThemeTokenKind::Metric, &cfg.metrics);
        for (key, v) in &metrics {
            self.configured_metrics.insert(key.to_string());
            let px = Px(*v);
            match self.extra_metrics.get(key.as_str()).copied() {
                Some(prev) if prev == px => {}
                _ => {
                    Arc::make_mut(&mut self.extra_metrics).insert(key.to_string(), px);
                    changed = true;
                }
            }
        }

        let corners = canonicalize_config_map(ThemeTokenKind::Corners, &cfg.corners);
        for (key, v) in &corners {
            self.configured_corners.insert(key.to_string());
            match self.extra_corners.get(key.as_str()).copied() {
                Some(prev) if prev == *v => {}
                _ => {
                    self.extra_corners.insert(key.to_string(), *v);
                    changed = true;
                }
            }
        }

        let numbers = canonicalize_config_map(ThemeTokenKind::Number, &cfg.numbers);
        for (key, v) in &numbers {
            self.configured_numbers.insert(key.to_string());
            match self.extra_numbers.get(key.as_str()).copied() {
                Some(prev) if (prev - *v).abs() < 1e-6 => {}
                _ => {
                    self.extra_numbers.insert(key.to_string(), *v);
                    changed = true;
                }
            }
        }

        let durations_ms = canonicalize_config_map(ThemeTokenKind::DurationMs, &cfg.durations_ms);
        for (key, v) in &durations_ms {
            self.configured_durations_ms.insert(key.to_string());
            match self.extra_durations_ms.get(key.as_str()).copied() {
                Some(prev) if prev == *v => {}
                _ => {
                    self.extra_durations_ms.insert(key.to_string(), *v);
                    changed = true;
                }
            }
        }

        let easings = canonicalize_config_map(ThemeTokenKind::Easing, &cfg.easings);
        for (key, v) in &easings {
            self.configured_easings.insert(key.to_string());
            match self.extra_easings.get(key.as_str()).copied() {
                Some(prev) if prev == *v => {}
                _ => {
                    self.extra_easings.insert(key.to_string(), *v);
                    changed = true;
                }
            }
        }

        let text_styles = canonicalize_config_map(ThemeTokenKind::TextStyle, &cfg.text_styles);
        for (key, v) in &text_styles {
            self.configured_text_styles.insert(key.to_string());
            match self.extra_text_styles.get(key.as_str()) {
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
            surface_background: parse_default_theme_hex_color("surface_background", "#24272E"),
            panel_background: parse_default_theme_hex_color("panel_background", "#2B3038"),
            panel_border: parse_default_theme_hex_color("panel_border", "#3A424D"),
            text_primary: parse_default_theme_hex_color("text_primary", "#D7DEE9"),
            text_muted: parse_default_theme_hex_color("text_muted", "#AAB3C2"),
            text_disabled: parse_default_theme_hex_color("text_disabled", "#7D8798"),
            accent: parse_default_theme_hex_color("accent", "#3D8BFF"),
            selection_background: parse_default_theme_hex_color(
                "selection_background",
                "#3D8BFF66",
            ),
            selection_inactive_background: parse_default_theme_hex_color(
                "selection_inactive_background",
                "#3D8BFF3D",
            ),
            selection_window_inactive_background: parse_default_theme_hex_color(
                "selection_window_inactive_background",
                "#3D8BFF24",
            ),
            hover_background: parse_default_theme_hex_color("hover_background", "#363C46"),
            focus_ring: parse_default_theme_hex_color("focus_ring", "#3D8BFFCC"),
            menu_background: parse_default_theme_hex_color("menu_background", "#2B3038"),
            menu_border: parse_default_theme_hex_color("menu_border", "#3A424D"),
            menu_item_hover: parse_default_theme_hex_color("menu_item_hover", "#363C46"),
            menu_item_selected: parse_default_theme_hex_color("menu_item_selected", "#3D8BFF66"),
            list_background: parse_default_theme_hex_color("list_background", "#2B3038"),
            list_border: parse_default_theme_hex_color("list_border", "#3A424D"),
            list_row_hover: parse_default_theme_hex_color("list_row_hover", "#363C46"),
            list_row_selected: parse_default_theme_hex_color("list_row_selected", "#3D8BFF66"),
            scrollbar_track: parse_default_theme_hex_color("scrollbar_track", "#1C1F25"),
            scrollbar_thumb: parse_default_theme_hex_color("scrollbar_thumb", "#4C5666"),
            scrollbar_thumb_hover: parse_default_theme_hex_color(
                "scrollbar_thumb_hover",
                "#5A687D",
            ),

            viewport_selection_fill: parse_default_theme_hex_color(
                "viewport_selection_fill",
                "#3D8BFF29",
            ),
            viewport_selection_stroke: parse_default_theme_hex_color(
                "viewport_selection_stroke",
                "#3D8BFFCC",
            ),
            viewport_marker: parse_default_theme_hex_color("viewport_marker", "#3D8BFFFF"),
            viewport_drag_line_pan: parse_default_theme_hex_color(
                "viewport_drag_line_pan",
                "#33E684D9",
            ),
            viewport_drag_line_orbit: parse_default_theme_hex_color(
                "viewport_drag_line_orbit",
                "#FFC44AD9",
            ),
            viewport_gizmo_x: parse_default_theme_hex_color("viewport_gizmo_x", "#E74C3CFF"),
            viewport_gizmo_y: parse_default_theme_hex_color("viewport_gizmo_y", "#2ECC71FF"),
            viewport_gizmo_handle_background: parse_default_theme_hex_color(
                "viewport_gizmo_handle_background",
                "#1E2229FF",
            ),
            viewport_gizmo_handle_border: parse_default_theme_hex_color(
                "viewport_gizmo_handle_border",
                "#D7DEE9FF",
            ),
            viewport_rotate_gizmo: parse_default_theme_hex_color(
                "viewport_rotate_gizmo",
                "#FFC44AFF",
            ),
        };

        Theme {
            name: "Fret Default (Dark)".to_string(),
            author: Some("Fret".to_string()),
            url: None,
            revision: 1,
            metrics,
            colors,
            extra_colors: Arc::new(default_color_tokens(colors)),
            extra_metrics: Arc::new(default_metric_tokens(metrics)),
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
            // shadcn/new-york v4 extended palette + legacy aliases.
            "chart-1",
            "chart-2",
            "chart-3",
            "chart-4",
            "chart-5",
            "chart.1",
            "chart.2",
            "chart.3",
            "chart.4",
            "chart.5",
            "sidebar",
            "sidebar.background",
            "sidebar-background",
            "sidebar-foreground",
            "sidebar.foreground",
            "sidebar-primary",
            "sidebar.primary",
            "sidebar-primary-foreground",
            "sidebar.primary.foreground",
            "sidebar-accent",
            "sidebar.accent",
            "sidebar-accent-foreground",
            "sidebar.accent.foreground",
            "sidebar-border",
            "sidebar.border",
            "sidebar-ring",
            "sidebar.ring",
        ] {
            assert!(theme.color_by_key(key).is_some(), "missing alias {key}");
        }
    }

    #[test]
    fn theme_snapshot_includes_configured_color_tokens() {
        let host = crate::test_host::TestHost::default();
        let mut theme = Theme::global(&host).clone();

        let mut cfg = ThemeConfig::default();
        cfg.colors
            .insert("muted".to_string(), "#ff0000".to_string());
        theme.apply_config(&cfg);

        let snapshot = theme.snapshot();
        assert_eq!(theme.color_token("muted"), snapshot.color_token("muted"));
    }

    #[test]
    fn theme_snapshot_matches_theme_for_common_semantic_tokens() {
        let host = crate::test_host::TestHost::default();
        let theme = Theme::global(&host);
        let snapshot = theme.snapshot();

        for key in [
            "background",
            "foreground",
            "border",
            "card",
            "card-foreground",
            "muted",
            "muted-foreground",
            "accent",
            "accent-foreground",
            "primary",
            "primary-foreground",
            "popover",
            "popover-foreground",
            "chart-1",
            "sidebar",
            "sidebar-foreground",
        ] {
            assert_eq!(
                theme.color_token(key),
                snapshot.color_token(key),
                "key={key}"
            );
        }

        for key in ["metric.size.sm", "metric.size.md", "metric.size.lg"] {
            assert_eq!(
                theme.metric_token(key),
                snapshot.metric_token(key),
                "key={key}"
            );
        }
    }

    #[test]
    fn missing_theme_token_diagnostics_warn_once_per_key() {
        // Use a unique key to avoid cross-test coupling (the warn-once cache is process-global).
        let key = format!("color.__missing_theme_token_test__{}", line!());
        assert!(super::warn_missing_theme_token_once(
            crate::theme_registry::ThemeTokenKind::Color,
            &key
        ));
        assert!(!super::warn_missing_theme_token_once(
            crate::theme_registry::ThemeTokenKind::Color,
            &key
        ));
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

        // Baseline dotted keys should mirror the resolved typed baseline even when the config only
        // provided semantic aliases.
        assert_eq!(theme.color_by_key("color.surface.background"), Some(bg));
        assert_eq!(theme.color_by_key("color.text.primary"), Some(fg));
        assert_eq!(theme.color_by_key("color.panel.border"), Some(border));
        assert_eq!(theme.color_by_key("color.focus.ring"), Some(ring));
        assert_eq!(theme.color_by_key("color.accent"), Some(primary));
        assert_eq!(theme.color_by_key("color.text.muted"), Some(muted_fg));
    }

    #[test]
    fn baseline_dotted_keys_update_typed_theme_colors() {
        let mut theme = Theme::global(&crate::test_host::TestHost::default()).clone();

        let mut colors = HashMap::new();
        colors.insert(
            "color.surface.background".to_string(),
            "#010203".to_string(),
        );
        colors.insert("color.text.primary".to_string(), "#AABBCC".to_string());
        colors.insert("color.panel.border".to_string(), "#112233".to_string());
        let cfg = ThemeConfig {
            name: "Baseline Dotted".to_string(),
            colors,
            ..Default::default()
        };
        theme.apply_config(&cfg);

        let bg = theme
            .color_by_key("color.surface.background")
            .expect("color.surface.background");
        let fg = theme
            .color_by_key("color.text.primary")
            .expect("color.text.primary");
        let border = theme
            .color_by_key("color.panel.border")
            .expect("color.panel.border");

        assert_eq!(theme.colors.surface_background, bg);
        assert_eq!(theme.colors.text_primary, fg);
        assert_eq!(theme.colors.panel_border, border);

        // Semantic aliases should mirror the typed baseline after normalization.
        assert_eq!(theme.color_by_key("background"), Some(bg));
        assert_eq!(theme.color_by_key("foreground"), Some(fg));
        assert_eq!(theme.color_by_key("border"), Some(border));
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
    fn required_accessors_do_not_panic_when_tokens_are_missing_by_default() {
        let _guard = super::strict_theme_for_tests(false);
        let host = crate::test_host::TestHost::default();
        let theme = Theme::global(&host);

        let _ = theme.color_required("missing.color.token");
        let _ = theme.metric_required("missing.metric.token");
        let _ = theme.corners_required("missing.corners.token");
        let _ = theme.number_required("missing.number.token");
        let _ = theme.duration_ms_required("missing.duration.token");
        let _ = theme.easing_required("missing.easing.token");
        let _ = theme.text_style_required("missing.text_style.token");

        let snap = theme.snapshot();
        let _ = snap.color_required("missing.color.token");
        let _ = snap.metric_required("missing.metric.token");
    }

    #[test]
    fn required_accessors_panic_in_strict_runtime_mode() {
        let _guard = super::strict_theme_for_tests(true);
        let host = crate::test_host::TestHost::default();
        let theme = Theme::global(&host);

        let result = std::panic::catch_unwind(|| {
            let _ = theme.color_required("missing.color.token");
        });
        assert!(result.is_err());
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
                    vertical_placement: fret_core::TextVerticalPlacement::CenterMetricsBox,
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

    #[test]
    fn apply_config_prefers_canonical_keys_over_aliases() {
        let mut theme = Theme::global(&crate::test_host::TestHost::default()).clone();

        theme.apply_config(&ThemeConfig {
            name: "Cfg".to_string(),
            colors: HashMap::from([
                ("primary-foreground".to_string(), "#ffffff".to_string()),
                ("primary.foreground".to_string(), "#000000".to_string()),
            ]),
            ..ThemeConfig::default()
        });

        assert!(theme.color_key_configured("primary-foreground"));
        assert!(theme.color_key_configured("primary.foreground"));
        assert_eq!(
            theme.color_by_key("primary-foreground"),
            parse_color_to_linear("#ffffff")
        );
    }

    #[test]
    fn apply_config_patch_prefers_canonical_keys_over_aliases() {
        let mut theme = Theme::global(&crate::test_host::TestHost::default()).clone();

        theme.apply_config_patch(&ThemeConfig {
            name: "Patch".to_string(),
            colors: HashMap::from([
                ("primary-foreground".to_string(), "#ffffff".to_string()),
                ("primary.foreground".to_string(), "#000000".to_string()),
            ]),
            ..ThemeConfig::default()
        });

        assert!(theme.color_key_configured("primary-foreground"));
        assert!(theme.color_key_configured("primary.foreground"));
        assert_eq!(
            theme.color_by_key("primary-foreground"),
            parse_color_to_linear("#ffffff")
        );
    }

    #[test]
    fn extend_tokens_from_config_prefers_canonical_keys_over_aliases_without_touching_configured() {
        let mut theme = Theme::global(&crate::test_host::TestHost::default()).clone();

        theme.apply_config(&ThemeConfig {
            name: "Base".to_string(),
            metrics: HashMap::from([("metric.padding.sm".to_string(), 7.0)]),
            ..ThemeConfig::default()
        });
        assert!(theme.metric_key_configured("metric.padding.sm"));
        assert!(!theme.color_key_configured("primary-foreground"));

        theme.extend_tokens_from_config(&ThemeConfig {
            name: "Extras".to_string(),
            colors: HashMap::from([
                ("primary-foreground".to_string(), "#ffffff".to_string()),
                ("primary.foreground".to_string(), "#000000".to_string()),
            ]),
            ..ThemeConfig::default()
        });

        assert!(theme.metric_key_configured("metric.padding.sm"));
        assert!(!theme.color_key_configured("primary-foreground"));
        assert_eq!(
            theme.color_by_key("primary-foreground"),
            parse_color_to_linear("#ffffff")
        );
    }
}
