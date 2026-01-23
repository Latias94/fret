//! Material 3 token preset for Material Web `tokens/versions/v30_0` (web/static font context).
//!
//! This module intentionally targets **outcome alignment** (visual + interaction) and exposes a
//! stable "inject tokens into ThemeConfig" surface. It does not attempt to mirror the
//! `@material/web` API or DOM/Lit implementation details.

use fret_core::{FontId, FontWeight, Px, TextSlant, TextStyle};
use fret_ui::theme::{CubicBezier, ThemeConfig};
use material_colors::color::Argb;
use material_colors::dynamic_color::Variant as MaterialVariant;
use material_colors::theme::ThemeBuilder;

/// Material token version string (from Material Web generation metadata).
pub const MATERIAL_WEB_VERSION: &str = "30.0.14";

/// Default Material seed color (`#6750A4`), used by M3 documentation examples.
pub const DEFAULT_SEED_ARGB: u32 = 0xFF67_50A4;

/// Options for generating Material typescale text styles.
#[derive(Debug, Clone)]
pub struct TypographyOptions {
    /// How many logical pixels one `rem` maps to when converting Material Web tokens.
    ///
    /// Material Web emits typography tokens in `rem` units. For a desktop UI toolkit, we need a
    /// conventional mapping. The default uses the web convention: `1rem = 16px`.
    pub rem_in_px: f32,
    /// The default font id used for Material typescale roles.
    pub font: FontId,
}

impl Default for TypographyOptions {
    fn default() -> Self {
        Self {
            rem_in_px: 16.0,
            font: FontId::Ui,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemeMode {
    Light,
    Dark,
}

impl Default for SchemeMode {
    fn default() -> Self {
        Self::Dark
    }
}

/// Options for generating `md.sys.color.*` roles via Material dynamic color.
#[derive(Debug, Clone, Copy)]
pub struct ColorSchemeOptions {
    pub seed_argb: u32,
    pub mode: SchemeMode,
    pub variant: DynamicVariant,
}

impl Default for ColorSchemeOptions {
    fn default() -> Self {
        Self {
            seed_argb: DEFAULT_SEED_ARGB,
            mode: SchemeMode::default(),
            variant: DynamicVariant::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DynamicVariant {
    TonalSpot,
    Expressive,
}

impl Default for DynamicVariant {
    fn default() -> Self {
        Self::TonalSpot
    }
}

impl DynamicVariant {
    fn to_material(self) -> MaterialVariant {
        match self {
            Self::TonalSpot => MaterialVariant::TonalSpot,
            Self::Expressive => MaterialVariant::Expressive,
        }
    }
}

/// Injects a baseline subset of Material 3 tokens (v30) into an existing `ThemeConfig`.
///
/// Notes:
/// - This does not set `cfg.name`/`cfg.author`/`cfg.url`.
pub fn inject_tokens(cfg: &mut ThemeConfig, typography: &TypographyOptions) {
    inject_sys_state(cfg);
    inject_sys_state_focus_indicator(cfg);
    inject_sys_motion(cfg);
    inject_sys_shape(cfg);
    inject_sys_typescale(cfg, typography);
    inject_comp_button_scalars(cfg);
    inject_comp_icon_button_scalars(cfg);
    inject_comp_checkbox_scalars(cfg);
    inject_comp_switch_scalars(cfg);
    inject_comp_radio_button_scalars(cfg);
    inject_comp_outlined_text_field_scalars(cfg);
    inject_comp_filled_text_field_scalars(cfg);
}

/// Injects `md.sys.color.*` roles into `ThemeConfig`.
///
/// This uses `material-colors` (material-color-utilities port) to generate a dynamic scheme from a
/// seed color. The output is suitable for outcome-focused Material components built on top of
/// Fret's token-based theme registry.
pub fn inject_sys_colors(cfg: &mut ThemeConfig, options: ColorSchemeOptions) {
    let theme = ThemeBuilder::with_source(Argb::from_u32(options.seed_argb))
        .variant(options.variant.to_material())
        .build();

    let scheme = match options.mode {
        SchemeMode::Light => theme.schemes.light,
        SchemeMode::Dark => theme.schemes.dark,
    };

    insert_color(cfg, "md.sys.color.primary", scheme.primary);
    insert_color(cfg, "md.sys.color.on-primary", scheme.on_primary);
    insert_color(
        cfg,
        "md.sys.color.primary-container",
        scheme.primary_container,
    );
    insert_color(
        cfg,
        "md.sys.color.on-primary-container",
        scheme.on_primary_container,
    );
    insert_color(cfg, "md.sys.color.inverse-primary", scheme.inverse_primary);

    insert_color(cfg, "md.sys.color.secondary", scheme.secondary);
    insert_color(cfg, "md.sys.color.on-secondary", scheme.on_secondary);
    insert_color(
        cfg,
        "md.sys.color.secondary-container",
        scheme.secondary_container,
    );
    insert_color(
        cfg,
        "md.sys.color.on-secondary-container",
        scheme.on_secondary_container,
    );

    insert_color(cfg, "md.sys.color.tertiary", scheme.tertiary);
    insert_color(cfg, "md.sys.color.on-tertiary", scheme.on_tertiary);
    insert_color(
        cfg,
        "md.sys.color.tertiary-container",
        scheme.tertiary_container,
    );
    insert_color(
        cfg,
        "md.sys.color.on-tertiary-container",
        scheme.on_tertiary_container,
    );

    insert_color(cfg, "md.sys.color.error", scheme.error);
    insert_color(cfg, "md.sys.color.on-error", scheme.on_error);
    insert_color(cfg, "md.sys.color.error-container", scheme.error_container);
    insert_color(
        cfg,
        "md.sys.color.on-error-container",
        scheme.on_error_container,
    );

    insert_color(cfg, "md.sys.color.surface", scheme.surface);
    insert_color(cfg, "md.sys.color.surface-dim", scheme.surface_dim);
    insert_color(cfg, "md.sys.color.surface-bright", scheme.surface_bright);
    insert_color(cfg, "md.sys.color.surface-tint", scheme.surface_tint);
    insert_color(
        cfg,
        "md.sys.color.surface-container-lowest",
        scheme.surface_container_lowest,
    );
    insert_color(
        cfg,
        "md.sys.color.surface-container-low",
        scheme.surface_container_low,
    );
    insert_color(
        cfg,
        "md.sys.color.surface-container",
        scheme.surface_container,
    );
    insert_color(
        cfg,
        "md.sys.color.surface-container-high",
        scheme.surface_container_high,
    );
    insert_color(
        cfg,
        "md.sys.color.surface-container-highest",
        scheme.surface_container_highest,
    );
    insert_color(cfg, "md.sys.color.surface-variant", scheme.surface_variant);

    insert_color(cfg, "md.sys.color.background", scheme.background);
    insert_color(cfg, "md.sys.color.on-background", scheme.on_background);
    insert_color(cfg, "md.sys.color.on-surface", scheme.on_surface);
    insert_color(
        cfg,
        "md.sys.color.on-surface-variant",
        scheme.on_surface_variant,
    );

    insert_color(cfg, "md.sys.color.outline", scheme.outline);
    insert_color(cfg, "md.sys.color.outline-variant", scheme.outline_variant);

    insert_color(cfg, "md.sys.color.inverse-surface", scheme.inverse_surface);
    insert_color(
        cfg,
        "md.sys.color.inverse-on-surface",
        scheme.inverse_on_surface,
    );

    insert_color(cfg, "md.sys.color.shadow", scheme.shadow);
    insert_color(cfg, "md.sys.color.scrim", scheme.scrim);
}

/// Convenience helper returning a standalone `ThemeConfig` with Material 3 v30 tokens injected.
pub fn theme_config(typography: TypographyOptions) -> ThemeConfig {
    let mut cfg = ThemeConfig::default();
    inject_tokens(&mut cfg, &typography);
    cfg
}

/// Convenience helper returning a standalone `ThemeConfig` with Material 3 v30 tokens (including
/// `md.sys.color.*`) injected.
pub fn theme_config_with_colors(
    typography: TypographyOptions,
    colors: ColorSchemeOptions,
) -> ThemeConfig {
    let mut cfg = theme_config(typography);
    inject_sys_colors(&mut cfg, colors);
    inject_comp_button_colors_from_sys(&mut cfg);
    inject_comp_icon_button_colors_from_sys(&mut cfg);
    inject_comp_checkbox_colors_from_sys(&mut cfg);
    inject_comp_switch_colors_from_sys(&mut cfg);
    inject_comp_radio_button_colors_from_sys(&mut cfg);
    inject_comp_outlined_text_field_colors_from_sys(&mut cfg);
    inject_comp_filled_text_field_colors_from_sys(&mut cfg);
    cfg
}

fn insert_color(cfg: &mut ThemeConfig, key: &str, argb: Argb) {
    let value = if argb.alpha == 255 {
        format!("#{:02X}{:02X}{:02X}", argb.red, argb.green, argb.blue)
    } else {
        format!(
            "#{:02X}{:02X}{:02X}{:02X}",
            argb.red, argb.green, argb.blue, argb.alpha
        )
    };
    cfg.colors.insert(key.to_string(), value);
}

fn inject_comp_button_scalars(cfg: &mut ThemeConfig) {
    // Sources:
    // - repo-ref/material-web/tokens/versions/v30_0/sass/_md-comp-button-small.scss
    // - repo-ref/material-web/tokens/versions/v30_0/sass/_md-comp-button-*.scss

    cfg.metrics
        .insert("md.comp.button.small.container.height".to_string(), 40.0);
    cfg.metrics
        .insert("md.comp.button.small.leading-space".to_string(), 16.0);
    cfg.metrics
        .insert("md.comp.button.small.trailing-space".to_string(), 16.0);
    cfg.metrics
        .insert("md.comp.button.small.icon-label-space".to_string(), 8.0);
    cfg.metrics
        .insert("md.comp.button.small.icon.size".to_string(), 20.0);
    cfg.metrics.insert(
        "md.comp.button.small.outlined.outline.width".to_string(),
        1.0,
    );

    for variant in ["filled", "tonal", "elevated", "outlined", "text"] {
        cfg.numbers.insert(
            format!("md.comp.button.{variant}.disabled.container.opacity"),
            0.1,
        );
        cfg.numbers.insert(
            format!("md.comp.button.{variant}.disabled.label-text.opacity"),
            0.38,
        );

        cfg.numbers.insert(
            format!("md.comp.button.{variant}.hovered.state-layer.opacity"),
            0.08,
        );
        cfg.numbers.insert(
            format!("md.comp.button.{variant}.focused.state-layer.opacity"),
            0.1,
        );
        cfg.numbers.insert(
            format!("md.comp.button.{variant}.pressed.state-layer.opacity"),
            0.1,
        );
    }
}

fn inject_comp_button_colors_from_sys(cfg: &mut ThemeConfig) {
    // This builds a minimal set of `md.comp.button.*` color tokens based on the currently-injected
    // system color roles. The goal is to keep component recipes "token shaped" while still being
    // able to drive the scheme via dynamic color generation.

    copy_color(
        cfg,
        "md.comp.button.filled.container.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.button.filled.label-text.color",
        "md.sys.color.on-primary",
    );
    copy_color(
        cfg,
        "md.comp.button.filled.hovered.state-layer.color",
        "md.sys.color.on-primary",
    );
    copy_color(
        cfg,
        "md.comp.button.filled.focused.state-layer.color",
        "md.sys.color.on-primary",
    );
    copy_color(
        cfg,
        "md.comp.button.filled.pressed.state-layer.color",
        "md.sys.color.on-primary",
    );
    copy_color(
        cfg,
        "md.comp.button.filled.disabled.container.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.button.filled.disabled.label-text.color",
        "md.sys.color.on-surface",
    );

    copy_color(
        cfg,
        "md.comp.button.tonal.container.color",
        "md.sys.color.secondary-container",
    );
    copy_color(
        cfg,
        "md.comp.button.tonal.label-text.color",
        "md.sys.color.on-secondary-container",
    );
    copy_color(
        cfg,
        "md.comp.button.tonal.hovered.state-layer.color",
        "md.sys.color.on-secondary-container",
    );
    copy_color(
        cfg,
        "md.comp.button.tonal.focused.state-layer.color",
        "md.sys.color.on-secondary-container",
    );
    copy_color(
        cfg,
        "md.comp.button.tonal.pressed.state-layer.color",
        "md.sys.color.on-secondary-container",
    );
    copy_color(
        cfg,
        "md.comp.button.tonal.disabled.container.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.button.tonal.disabled.label-text.color",
        "md.sys.color.on-surface",
    );

    copy_color(
        cfg,
        "md.comp.button.elevated.container.color",
        "md.sys.color.surface-container-low",
    );
    copy_color(
        cfg,
        "md.comp.button.elevated.label-text.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.button.elevated.hovered.state-layer.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.button.elevated.focused.state-layer.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.button.elevated.pressed.state-layer.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.button.elevated.disabled.container.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.button.elevated.disabled.label-text.color",
        "md.sys.color.on-surface",
    );

    copy_color(
        cfg,
        "md.comp.button.outlined.label-text.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.button.outlined.outline.color",
        "md.sys.color.outline-variant",
    );
    copy_color(
        cfg,
        "md.comp.button.outlined.hovered.state-layer.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.button.outlined.focused.state-layer.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.button.outlined.pressed.state-layer.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.button.outlined.disabled.label-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.button.outlined.disabled.outline.color",
        "md.sys.color.outline-variant",
    );

    copy_color(
        cfg,
        "md.comp.button.text.label-text.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.button.text.hovered.state-layer.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.button.text.focused.state-layer.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.button.text.pressed.state-layer.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.button.text.disabled.label-text.color",
        "md.sys.color.on-surface",
    );
}

fn inject_comp_icon_button_scalars(cfg: &mut ThemeConfig) {
    // Sources:
    // - repo-ref/material-web/tokens/versions/v30_0/sass/_md-comp-icon-button-small.scss
    // - repo-ref/material-web/tokens/versions/v30_0/sass/_md-comp-icon-button-*.scss

    cfg.metrics.insert(
        "md.comp.icon-button.small.container.height".to_string(),
        40.0,
    );
    cfg.metrics.insert(
        "md.comp.icon-button.small.default.leading-space".to_string(),
        8.0,
    );
    cfg.metrics.insert(
        "md.comp.icon-button.small.default.trailing-space".to_string(),
        8.0,
    );
    cfg.metrics
        .insert("md.comp.icon-button.small.icon.size".to_string(), 24.0);
    cfg.metrics.insert(
        "md.comp.icon-button.small.outlined.outline.width".to_string(),
        1.0,
    );

    for variant in ["standard", "filled", "tonal", "outlined"] {
        cfg.numbers.insert(
            format!("md.comp.icon-button.{variant}.disabled.icon.opacity"),
            0.38,
        );

        cfg.numbers.insert(
            format!("md.comp.icon-button.{variant}.hovered.state-layer.opacity"),
            0.08,
        );
        cfg.numbers.insert(
            format!("md.comp.icon-button.{variant}.focused.state-layer.opacity"),
            0.1,
        );
        cfg.numbers.insert(
            format!("md.comp.icon-button.{variant}.pressed.state-layer.opacity"),
            0.1,
        );
    }

    // Filled/tonal also have a disabled container opacity.
    for variant in ["filled", "tonal"] {
        cfg.numbers.insert(
            format!("md.comp.icon-button.{variant}.disabled.container.opacity"),
            0.1,
        );
    }
    // Outlined selected state carries a disabled container opacity.
    cfg.numbers.insert(
        "md.comp.icon-button.outlined.selected.disabled.container.opacity".to_string(),
        0.1,
    );
}

fn inject_comp_icon_button_colors_from_sys(cfg: &mut ThemeConfig) {
    // See the corresponding Material Web token sets:
    // - `_md-comp-icon-button-standard.scss`
    // - `_md-comp-icon-button-filled.scss`
    // - `_md-comp-icon-button-tonal.scss`
    // - `_md-comp-icon-button-outlined.scss`

    // Standard.
    copy_color(
        cfg,
        "md.comp.icon-button.standard.icon.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.icon-button.standard.hovered.state-layer.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.icon-button.standard.focused.state-layer.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.icon-button.standard.pressed.state-layer.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.icon-button.standard.disabled.icon.color",
        "md.sys.color.on-surface",
    );

    copy_color(
        cfg,
        "md.comp.icon-button.standard.selected.icon.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.icon-button.standard.selected.hovered.state-layer.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.icon-button.standard.selected.focused.state-layer.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.icon-button.standard.selected.pressed.state-layer.color",
        "md.sys.color.primary",
    );

    // Filled.
    copy_color(
        cfg,
        "md.comp.icon-button.filled.container.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.icon-button.filled.icon.color",
        "md.sys.color.on-primary",
    );
    copy_color(
        cfg,
        "md.comp.icon-button.filled.hovered.state-layer.color",
        "md.sys.color.on-primary",
    );
    copy_color(
        cfg,
        "md.comp.icon-button.filled.focused.state-layer.color",
        "md.sys.color.on-primary",
    );
    copy_color(
        cfg,
        "md.comp.icon-button.filled.pressed.state-layer.color",
        "md.sys.color.on-primary",
    );
    copy_color(
        cfg,
        "md.comp.icon-button.filled.disabled.container.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.icon-button.filled.disabled.icon.color",
        "md.sys.color.on-surface",
    );

    // Filled selected/unselected container behavior.
    copy_color(
        cfg,
        "md.comp.icon-button.filled.selected.container.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.icon-button.filled.selected.icon.color",
        "md.sys.color.on-primary",
    );
    copy_color(
        cfg,
        "md.comp.icon-button.filled.unselected.container.color",
        "md.sys.color.surface-container",
    );
    copy_color(
        cfg,
        "md.comp.icon-button.filled.unselected.icon.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.icon-button.filled.unselected.hovered.icon.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.icon-button.filled.unselected.focused.icon.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.icon-button.filled.unselected.pressed.icon.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.icon-button.filled.unselected.hovered.state-layer.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.icon-button.filled.unselected.focused.state-layer.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.icon-button.filled.unselected.pressed.state-layer.color",
        "md.sys.color.on-surface-variant",
    );

    // Tonal.
    copy_color(
        cfg,
        "md.comp.icon-button.tonal.container.color",
        "md.sys.color.secondary-container",
    );
    copy_color(
        cfg,
        "md.comp.icon-button.tonal.icon.color",
        "md.sys.color.on-secondary-container",
    );
    copy_color(
        cfg,
        "md.comp.icon-button.tonal.hovered.state-layer.color",
        "md.sys.color.on-secondary-container",
    );
    copy_color(
        cfg,
        "md.comp.icon-button.tonal.focused.state-layer.color",
        "md.sys.color.on-secondary-container",
    );
    copy_color(
        cfg,
        "md.comp.icon-button.tonal.pressed.state-layer.color",
        "md.sys.color.on-secondary-container",
    );
    copy_color(
        cfg,
        "md.comp.icon-button.tonal.disabled.container.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.icon-button.tonal.disabled.icon.color",
        "md.sys.color.on-surface",
    );

    copy_color(
        cfg,
        "md.comp.icon-button.tonal.selected.container.color",
        "md.sys.color.secondary",
    );
    copy_color(
        cfg,
        "md.comp.icon-button.tonal.selected.icon.color",
        "md.sys.color.on-secondary",
    );

    // Outlined.
    copy_color(
        cfg,
        "md.comp.icon-button.outlined.icon.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.icon-button.outlined.outline.color",
        "md.sys.color.outline-variant",
    );
    copy_color(
        cfg,
        "md.comp.icon-button.outlined.hovered.state-layer.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.icon-button.outlined.focused.state-layer.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.icon-button.outlined.pressed.state-layer.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.icon-button.outlined.disabled.icon.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.icon-button.outlined.disabled.outline.color",
        "md.sys.color.outline-variant",
    );

    copy_color(
        cfg,
        "md.comp.icon-button.outlined.selected.container.color",
        "md.sys.color.inverse-surface",
    );
    copy_color(
        cfg,
        "md.comp.icon-button.outlined.selected.icon.color",
        "md.sys.color.inverse-on-surface",
    );
    copy_color(
        cfg,
        "md.comp.icon-button.outlined.selected.disabled.container.color",
        "md.sys.color.on-surface",
    );
}

fn copy_color(cfg: &mut ThemeConfig, to_key: &str, from_key: &str) {
    let Some(c) = cfg.colors.get(from_key).cloned() else {
        return;
    };
    cfg.colors.insert(to_key.to_string(), c);
}

fn inject_sys_state(cfg: &mut ThemeConfig) {
    // Source: repo-ref/material-web/tokens/versions/v30_0/sass/_md-sys-state.scss
    cfg.numbers.insert(
        "md.sys.state.disabled.state-layer-opacity".to_string(),
        0.38,
    );
    cfg.numbers
        .insert("md.sys.state.dragged.state-layer-opacity".to_string(), 0.16);
    cfg.numbers
        .insert("md.sys.state.focus.state-layer-opacity".to_string(), 0.1);
    cfg.numbers
        .insert("md.sys.state.hover.state-layer-opacity".to_string(), 0.08);
    cfg.numbers
        .insert("md.sys.state.pressed.state-layer-opacity".to_string(), 0.1);
}

fn inject_sys_state_focus_indicator(cfg: &mut ThemeConfig) {
    // Source: repo-ref/material-web/tokens/versions/v30_0/sass/_md-sys-state-focus-indicator.scss
    cfg.metrics.insert(
        "md.sys.state.focus-indicator.inner-offset".to_string(),
        -3.0,
    );
    cfg.metrics
        .insert("md.sys.state.focus-indicator.outer-offset".to_string(), 2.0);
    cfg.metrics
        .insert("md.sys.state.focus-indicator.thickness".to_string(), 3.0);
}

fn inject_comp_checkbox_scalars(cfg: &mut ThemeConfig) {
    // Sources:
    // - repo-ref/material-web/tokens/versions/v30_0/sass/_md-comp-checkbox.scss
    // - repo-ref/material-web/tokens/versions/v30_0/sass/_md-sys-state-focus-indicator.scss

    cfg.metrics
        .insert("md.comp.checkbox.container.size".to_string(), 18.0);
    cfg.metrics
        .insert("md.comp.checkbox.container.shape".to_string(), 2.0);
    cfg.metrics
        .insert("md.comp.checkbox.icon.size".to_string(), 18.0);
    cfg.metrics
        .insert("md.comp.checkbox.state-layer.size".to_string(), 40.0);

    cfg.metrics
        .insert("md.comp.checkbox.selected.outline.width".to_string(), 0.0);
    cfg.metrics
        .insert("md.comp.checkbox.unselected.outline.width".to_string(), 2.0);
    cfg.metrics.insert(
        "md.comp.checkbox.selected.disabled.container.outline.width".to_string(),
        0.0,
    );
    cfg.metrics.insert(
        "md.comp.checkbox.unselected.disabled.outline.width".to_string(),
        2.0,
    );

    cfg.numbers.insert(
        "md.comp.checkbox.selected.disabled.container.opacity".to_string(),
        0.38,
    );
    cfg.numbers.insert(
        "md.comp.checkbox.unselected.disabled.container.opacity".to_string(),
        0.38,
    );

    cfg.metrics.insert(
        "md.comp.checkbox.focus.indicator.outline.offset".to_string(),
        2.0,
    );
    cfg.metrics.insert(
        "md.comp.checkbox.focus.indicator.thickness".to_string(),
        3.0,
    );
}

fn inject_comp_checkbox_colors_from_sys(cfg: &mut ThemeConfig) {
    copy_color(
        cfg,
        "md.comp.checkbox.selected.container.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.checkbox.selected.icon.color",
        "md.sys.color.on-primary",
    );
    copy_color(
        cfg,
        "md.comp.checkbox.selected.hover.state-layer.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.checkbox.selected.focus.state-layer.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.checkbox.selected.pressed.state-layer.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.checkbox.selected.disabled.container.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.checkbox.selected.disabled.icon.color",
        "md.sys.color.surface",
    );

    copy_color(
        cfg,
        "md.comp.checkbox.unselected.outline.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.checkbox.unselected.hover.outline.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.checkbox.unselected.focus.outline.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.checkbox.unselected.pressed.outline.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.checkbox.unselected.disabled.outline.color",
        "md.sys.color.on-surface",
    );

    copy_color(
        cfg,
        "md.comp.checkbox.unselected.hover.state-layer.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.checkbox.unselected.focus.state-layer.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.checkbox.unselected.pressed.state-layer.color",
        "md.sys.color.primary",
    );

    copy_color(
        cfg,
        "md.comp.checkbox.focus.indicator.color",
        "md.sys.color.secondary",
    );
}

fn inject_comp_switch_scalars(cfg: &mut ThemeConfig) {
    // Sources:
    // - repo-ref/material-web/tokens/versions/v30_0/sass/_md-comp-switch.scss

    cfg.metrics
        .insert("md.comp.switch.state-layer.size".to_string(), 40.0);
    cfg.metrics
        .insert("md.comp.switch.track.width".to_string(), 52.0);
    cfg.metrics
        .insert("md.comp.switch.track.height".to_string(), 32.0);
    cfg.metrics
        .insert("md.comp.switch.track.outline.width".to_string(), 2.0);

    cfg.metrics
        .insert("md.comp.switch.selected.handle.size".to_string(), 24.0);
    cfg.metrics
        .insert("md.comp.switch.unselected.handle.size".to_string(), 16.0);
    cfg.metrics
        .insert("md.comp.switch.pressed.handle.size".to_string(), 28.0);

    cfg.metrics
        .insert("md.comp.switch.focus.indicator.offset".to_string(), 2.0);
    cfg.metrics
        .insert("md.comp.switch.focus.indicator.thickness".to_string(), 3.0);

    cfg.numbers
        .insert("md.comp.switch.disabled.track.opacity".to_string(), 0.12);
    cfg.numbers.insert(
        "md.comp.switch.disabled.selected.handle.opacity".to_string(),
        1.0,
    );
    cfg.numbers.insert(
        "md.comp.switch.disabled.unselected.handle.opacity".to_string(),
        0.38,
    );

    // State layer opacities are derived from sys state by default.
    for group in ["selected", "unselected"] {
        cfg.numbers.insert(
            format!("md.comp.switch.{group}.hover.state-layer.opacity"),
            0.08,
        );
        cfg.numbers.insert(
            format!("md.comp.switch.{group}.focus.state-layer.opacity"),
            0.1,
        );
        cfg.numbers.insert(
            format!("md.comp.switch.{group}.pressed.state-layer.opacity"),
            0.1,
        );
    }
}

fn inject_comp_switch_colors_from_sys(cfg: &mut ThemeConfig) {
    // Selected (on)
    copy_color(
        cfg,
        "md.comp.switch.selected.track.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.switch.selected.hover.track.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.switch.selected.pressed.track.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.switch.selected.focus.track.color",
        "md.sys.color.primary",
    );

    copy_color(
        cfg,
        "md.comp.switch.selected.handle.color",
        "md.sys.color.on-primary",
    );
    copy_color(
        cfg,
        "md.comp.switch.selected.hover.handle.color",
        "md.sys.color.primary-container",
    );
    copy_color(
        cfg,
        "md.comp.switch.selected.pressed.handle.color",
        "md.sys.color.primary-container",
    );
    copy_color(
        cfg,
        "md.comp.switch.selected.focus.handle.color",
        "md.sys.color.primary-container",
    );

    copy_color(
        cfg,
        "md.comp.switch.selected.hover.state-layer.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.switch.selected.focus.state-layer.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.switch.selected.pressed.state-layer.color",
        "md.sys.color.primary",
    );

    // Unselected (off)
    copy_color(
        cfg,
        "md.comp.switch.unselected.track.color",
        "md.sys.color.surface-container-highest",
    );
    copy_color(
        cfg,
        "md.comp.switch.unselected.hover.track.color",
        "md.sys.color.surface-container-highest",
    );
    copy_color(
        cfg,
        "md.comp.switch.unselected.pressed.track.color",
        "md.sys.color.surface-container-highest",
    );
    copy_color(
        cfg,
        "md.comp.switch.unselected.focus.track.color",
        "md.sys.color.surface-container-highest",
    );

    copy_color(
        cfg,
        "md.comp.switch.unselected.track.outline.color",
        "md.sys.color.outline",
    );
    copy_color(
        cfg,
        "md.comp.switch.unselected.hover.track.outline.color",
        "md.sys.color.outline",
    );
    copy_color(
        cfg,
        "md.comp.switch.unselected.pressed.track.outline.color",
        "md.sys.color.outline",
    );
    copy_color(
        cfg,
        "md.comp.switch.unselected.focus.track.outline.color",
        "md.sys.color.outline",
    );

    copy_color(
        cfg,
        "md.comp.switch.unselected.handle.color",
        "md.sys.color.outline",
    );
    copy_color(
        cfg,
        "md.comp.switch.unselected.hover.handle.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.switch.unselected.pressed.handle.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.switch.unselected.focus.handle.color",
        "md.sys.color.on-surface-variant",
    );

    copy_color(
        cfg,
        "md.comp.switch.unselected.hover.state-layer.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.switch.unselected.focus.state-layer.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.switch.unselected.pressed.state-layer.color",
        "md.sys.color.on-surface",
    );

    // Disabled colors
    copy_color(
        cfg,
        "md.comp.switch.disabled.selected.track.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.switch.disabled.selected.handle.color",
        "md.sys.color.surface",
    );
    copy_color(
        cfg,
        "md.comp.switch.disabled.unselected.track.color",
        "md.sys.color.surface-container-highest",
    );
    copy_color(
        cfg,
        "md.comp.switch.disabled.unselected.track.outline.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.switch.disabled.unselected.handle.color",
        "md.sys.color.on-surface",
    );

    copy_color(
        cfg,
        "md.comp.switch.focus.indicator.color",
        "md.sys.color.secondary",
    );
}

fn inject_comp_radio_button_scalars(cfg: &mut ThemeConfig) {
    // Sources:
    // - repo-ref/material-web/tokens/versions/v30_0/sass/_md-comp-radio-button.scss

    cfg.metrics
        .insert("md.comp.radio-button.icon.size".to_string(), 20.0);
    cfg.metrics
        .insert("md.comp.radio-button.state-layer.size".to_string(), 40.0);

    cfg.numbers.insert(
        "md.comp.radio-button.disabled.selected.icon.opacity".to_string(),
        0.38,
    );
    cfg.numbers.insert(
        "md.comp.radio-button.disabled.unselected.icon.opacity".to_string(),
        0.38,
    );

    // State layer opacities are derived from sys state by default.
    for group in ["selected", "unselected"] {
        cfg.numbers.insert(
            format!("md.comp.radio-button.{group}.hover.state-layer.opacity"),
            0.08,
        );
        cfg.numbers.insert(
            format!("md.comp.radio-button.{group}.focus.state-layer.opacity"),
            0.1,
        );
        cfg.numbers.insert(
            format!("md.comp.radio-button.{group}.pressed.state-layer.opacity"),
            0.1,
        );
    }
}

fn inject_comp_radio_button_colors_from_sys(cfg: &mut ThemeConfig) {
    copy_color(
        cfg,
        "md.comp.radio-button.disabled.selected.icon.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.radio-button.disabled.unselected.icon.color",
        "md.sys.color.on-surface",
    );

    copy_color(
        cfg,
        "md.comp.radio-button.selected.icon.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.radio-button.selected.hover.icon.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.radio-button.selected.focus.icon.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.radio-button.selected.pressed.icon.color",
        "md.sys.color.primary",
    );

    copy_color(
        cfg,
        "md.comp.radio-button.selected.hover.state-layer.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.radio-button.selected.focus.state-layer.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.radio-button.selected.pressed.state-layer.color",
        "md.sys.color.on-surface",
    );

    copy_color(
        cfg,
        "md.comp.radio-button.unselected.icon.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.radio-button.unselected.hover.icon.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.radio-button.unselected.focus.icon.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.radio-button.unselected.pressed.icon.color",
        "md.sys.color.on-surface",
    );

    copy_color(
        cfg,
        "md.comp.radio-button.unselected.hover.state-layer.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.radio-button.unselected.focus.state-layer.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.radio-button.unselected.pressed.state-layer.color",
        "md.sys.color.primary",
    );
}

fn inject_comp_outlined_text_field_scalars(cfg: &mut ThemeConfig) {
    // Source: repo-ref/material-web/tokens/versions/v30_0/sass/_md-comp-outlined-text-field.scss

    cfg.metrics.insert(
        "md.comp.outlined-text-field.container.height".to_string(),
        56.0,
    );
    cfg.metrics.insert(
        "md.comp.outlined-text-field.container.shape".to_string(),
        4.0,
    );

    cfg.metrics
        .insert("md.comp.outlined-text-field.outline.width".to_string(), 1.0);
    cfg.metrics.insert(
        "md.comp.outlined-text-field.hover.outline.width".to_string(),
        1.0,
    );
    cfg.metrics.insert(
        "md.comp.outlined-text-field.focus.outline.width".to_string(),
        3.0,
    );
    cfg.metrics.insert(
        "md.comp.outlined-text-field.disabled.outline.width".to_string(),
        1.0,
    );

    cfg.metrics.insert(
        "md.comp.outlined-text-field.leading-icon.size".to_string(),
        24.0,
    );
    cfg.metrics.insert(
        "md.comp.outlined-text-field.trailing-icon.size".to_string(),
        24.0,
    );

    cfg.numbers.insert(
        "md.comp.outlined-text-field.disabled.input-text.opacity".to_string(),
        0.38,
    );
    cfg.numbers.insert(
        "md.comp.outlined-text-field.disabled.label-text.opacity".to_string(),
        0.38,
    );
    cfg.numbers.insert(
        "md.comp.outlined-text-field.disabled.supporting-text.opacity".to_string(),
        0.38,
    );
    cfg.numbers.insert(
        "md.comp.outlined-text-field.disabled.outline.opacity".to_string(),
        0.12,
    );
}

fn inject_comp_filled_text_field_scalars(cfg: &mut ThemeConfig) {
    // Source: repo-ref/material-web/tokens/versions/v30_0/sass/_md-comp-filled-text-field.scss

    cfg.metrics.insert(
        "md.comp.filled-text-field.container.height".to_string(),
        56.0,
    );
    cfg.metrics
        .insert("md.comp.filled-text-field.container.shape".to_string(), 4.0);

    cfg.metrics.insert(
        "md.comp.filled-text-field.active-indicator.height".to_string(),
        1.0,
    );
    cfg.metrics.insert(
        "md.comp.filled-text-field.hover.active-indicator.height".to_string(),
        1.0,
    );
    cfg.metrics.insert(
        "md.comp.filled-text-field.focus.active-indicator.height".to_string(),
        2.0,
    );
    cfg.metrics.insert(
        "md.comp.filled-text-field.disabled.active-indicator.height".to_string(),
        1.0,
    );

    cfg.metrics.insert(
        "md.comp.filled-text-field.leading-icon.size".to_string(),
        24.0,
    );
    cfg.metrics.insert(
        "md.comp.filled-text-field.trailing-icon.size".to_string(),
        24.0,
    );

    cfg.numbers.insert(
        "md.comp.filled-text-field.disabled.active-indicator.opacity".to_string(),
        0.38,
    );
    cfg.numbers.insert(
        "md.comp.filled-text-field.disabled.container.opacity".to_string(),
        0.04,
    );
    cfg.numbers.insert(
        "md.comp.filled-text-field.disabled.input-text.opacity".to_string(),
        0.38,
    );
    cfg.numbers.insert(
        "md.comp.filled-text-field.disabled.label-text.opacity".to_string(),
        0.38,
    );
    cfg.numbers.insert(
        "md.comp.filled-text-field.disabled.leading-icon.opacity".to_string(),
        0.38,
    );
    cfg.numbers.insert(
        "md.comp.filled-text-field.disabled.supporting-text.opacity".to_string(),
        0.38,
    );
    cfg.numbers.insert(
        "md.comp.filled-text-field.disabled.trailing-icon.opacity".to_string(),
        0.38,
    );

    cfg.numbers.insert(
        "md.comp.filled-text-field.hover.state-layer.opacity".to_string(),
        0.08,
    );
    cfg.numbers.insert(
        "md.comp.filled-text-field.error.hover.state-layer.opacity".to_string(),
        0.08,
    );
}

fn inject_comp_outlined_text_field_colors_from_sys(cfg: &mut ThemeConfig) {
    copy_color(
        cfg,
        "md.comp.outlined-text-field.caret.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.outlined-text-field.error.focus.caret.color",
        "md.sys.color.error",
    );

    copy_color(
        cfg,
        "md.comp.outlined-text-field.input-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.outlined-text-field.focus.input-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.outlined-text-field.hover.input-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.outlined-text-field.input-text.placeholder.color",
        "md.sys.color.on-surface-variant",
    );

    copy_color(
        cfg,
        "md.comp.outlined-text-field.label-text.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.outlined-text-field.focus.label-text.color",
        "md.sys.color.primary",
    );

    copy_color(
        cfg,
        "md.comp.outlined-text-field.outline.color",
        "md.sys.color.outline",
    );
    copy_color(
        cfg,
        "md.comp.outlined-text-field.hover.outline.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.outlined-text-field.focus.outline.color",
        "md.sys.color.primary",
    );

    copy_color(
        cfg,
        "md.comp.outlined-text-field.disabled.input-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.outlined-text-field.disabled.label-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.outlined-text-field.disabled.supporting-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.outlined-text-field.disabled.outline.color",
        "md.sys.color.on-surface",
    );

    copy_color(
        cfg,
        "md.comp.outlined-text-field.supporting-text.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.outlined-text-field.focus.supporting-text.color",
        "md.sys.color.on-surface-variant",
    );

    copy_color(
        cfg,
        "md.comp.outlined-text-field.error.label-text.color",
        "md.sys.color.error",
    );
    copy_color(
        cfg,
        "md.comp.outlined-text-field.error.focus.label-text.color",
        "md.sys.color.error",
    );
    copy_color(
        cfg,
        "md.comp.outlined-text-field.error.outline.color",
        "md.sys.color.error",
    );
    copy_color(
        cfg,
        "md.comp.outlined-text-field.error.focus.outline.color",
        "md.sys.color.error",
    );
    copy_color(
        cfg,
        "md.comp.outlined-text-field.error.supporting-text.color",
        "md.sys.color.error",
    );
    copy_color(
        cfg,
        "md.comp.outlined-text-field.error.focus.supporting-text.color",
        "md.sys.color.error",
    );
}

fn inject_comp_filled_text_field_colors_from_sys(cfg: &mut ThemeConfig) {
    // Source: repo-ref/material-web/tokens/versions/v30_0/sass/_md-comp-filled-text-field.scss

    copy_color(
        cfg,
        "md.comp.filled-text-field.active-indicator.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.filled-text-field.hover.active-indicator.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.filled-text-field.hover.state-layer.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.filled-text-field.focus.active-indicator.color",
        "md.sys.color.primary",
    );

    copy_color(
        cfg,
        "md.comp.filled-text-field.disabled.active-indicator.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.filled-text-field.disabled.container.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.filled-text-field.disabled.input-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.filled-text-field.disabled.label-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.filled-text-field.disabled.supporting-text.color",
        "md.sys.color.on-surface",
    );

    copy_color(
        cfg,
        "md.comp.filled-text-field.caret.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.filled-text-field.error.focus.caret.color",
        "md.sys.color.error",
    );

    copy_color(
        cfg,
        "md.comp.filled-text-field.container.color",
        "md.sys.color.surface-container-highest",
    );
    copy_color(
        cfg,
        "md.comp.filled-text-field.input-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.filled-text-field.focus.input-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.filled-text-field.hover.input-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.filled-text-field.input-text.placeholder.color",
        "md.sys.color.on-surface-variant",
    );

    copy_color(
        cfg,
        "md.comp.filled-text-field.label-text.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.filled-text-field.hover.label-text.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.filled-text-field.focus.label-text.color",
        "md.sys.color.primary",
    );

    copy_color(
        cfg,
        "md.comp.filled-text-field.supporting-text.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.filled-text-field.hover.supporting-text.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.filled-text-field.focus.supporting-text.color",
        "md.sys.color.on-surface-variant",
    );

    copy_color(
        cfg,
        "md.comp.filled-text-field.error.active-indicator.color",
        "md.sys.color.error",
    );
    copy_color(
        cfg,
        "md.comp.filled-text-field.error.focus.active-indicator.color",
        "md.sys.color.error",
    );
    copy_color(
        cfg,
        "md.comp.filled-text-field.error.hover.active-indicator.color",
        "md.sys.color.on-error-container",
    );
    copy_color(
        cfg,
        "md.comp.filled-text-field.error.hover.state-layer.color",
        "md.sys.color.on-surface",
    );

    copy_color(
        cfg,
        "md.comp.filled-text-field.error.input-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.filled-text-field.error.hover.input-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.filled-text-field.error.focus.input-text.color",
        "md.sys.color.on-surface",
    );

    copy_color(
        cfg,
        "md.comp.filled-text-field.error.label-text.color",
        "md.sys.color.error",
    );
    copy_color(
        cfg,
        "md.comp.filled-text-field.error.focus.label-text.color",
        "md.sys.color.error",
    );
    copy_color(
        cfg,
        "md.comp.filled-text-field.error.hover.label-text.color",
        "md.sys.color.on-error-container",
    );

    copy_color(
        cfg,
        "md.comp.filled-text-field.error.supporting-text.color",
        "md.sys.color.error",
    );
    copy_color(
        cfg,
        "md.comp.filled-text-field.error.hover.supporting-text.color",
        "md.sys.color.error",
    );
    copy_color(
        cfg,
        "md.comp.filled-text-field.error.focus.supporting-text.color",
        "md.sys.color.error",
    );
}

fn inject_sys_motion(cfg: &mut ThemeConfig) {
    // Source: repo-ref/material-web/tokens/versions/v30_0/sass/_md-sys-motion.scss
    cfg.durations_ms
        .insert("md.sys.motion.duration.short1".to_string(), 50);
    cfg.durations_ms
        .insert("md.sys.motion.duration.short2".to_string(), 100);
    cfg.durations_ms
        .insert("md.sys.motion.duration.short3".to_string(), 150);
    cfg.durations_ms
        .insert("md.sys.motion.duration.short4".to_string(), 200);

    cfg.durations_ms
        .insert("md.sys.motion.duration.medium1".to_string(), 250);
    cfg.durations_ms
        .insert("md.sys.motion.duration.medium2".to_string(), 300);
    cfg.durations_ms
        .insert("md.sys.motion.duration.medium3".to_string(), 350);
    cfg.durations_ms
        .insert("md.sys.motion.duration.medium4".to_string(), 400);

    cfg.durations_ms
        .insert("md.sys.motion.duration.long1".to_string(), 450);
    cfg.durations_ms
        .insert("md.sys.motion.duration.long2".to_string(), 500);
    cfg.durations_ms
        .insert("md.sys.motion.duration.long3".to_string(), 550);
    cfg.durations_ms
        .insert("md.sys.motion.duration.long4".to_string(), 600);

    cfg.durations_ms
        .insert("md.sys.motion.duration.extra-long1".to_string(), 700);
    cfg.durations_ms
        .insert("md.sys.motion.duration.extra-long2".to_string(), 800);
    cfg.durations_ms
        .insert("md.sys.motion.duration.extra-long3".to_string(), 900);
    cfg.durations_ms
        .insert("md.sys.motion.duration.extra-long4".to_string(), 1000);

    cfg.easings.insert(
        "md.sys.motion.easing.emphasized.accelerate".to_string(),
        CubicBezier {
            x1: 0.3,
            y1: 0.0,
            x2: 0.8,
            y2: 0.15,
        },
    );
    cfg.easings.insert(
        "md.sys.motion.easing.emphasized.decelerate".to_string(),
        CubicBezier {
            x1: 0.05,
            y1: 0.7,
            x2: 0.1,
            y2: 1.0,
        },
    );
    cfg.easings.insert(
        "md.sys.motion.easing.legacy".to_string(),
        CubicBezier {
            x1: 0.4,
            y1: 0.0,
            x2: 0.2,
            y2: 1.0,
        },
    );
    cfg.easings.insert(
        "md.sys.motion.easing.legacy.accelerate".to_string(),
        CubicBezier {
            x1: 0.4,
            y1: 0.0,
            x2: 1.0,
            y2: 1.0,
        },
    );
    cfg.easings.insert(
        "md.sys.motion.easing.legacy.decelerate".to_string(),
        CubicBezier {
            x1: 0.0,
            y1: 0.0,
            x2: 0.2,
            y2: 1.0,
        },
    );
    cfg.easings.insert(
        "md.sys.motion.easing.linear".to_string(),
        CubicBezier {
            x1: 0.0,
            y1: 0.0,
            x2: 1.0,
            y2: 1.0,
        },
    );
    cfg.easings.insert(
        "md.sys.motion.easing.standard".to_string(),
        CubicBezier {
            x1: 0.2,
            y1: 0.0,
            x2: 0.0,
            y2: 1.0,
        },
    );
    cfg.easings.insert(
        "md.sys.motion.easing.standard.accelerate".to_string(),
        CubicBezier {
            x1: 0.3,
            y1: 0.0,
            x2: 1.0,
            y2: 1.0,
        },
    );
    cfg.easings.insert(
        "md.sys.motion.easing.standard.decelerate".to_string(),
        CubicBezier {
            x1: 0.0,
            y1: 0.0,
            x2: 0.0,
            y2: 1.0,
        },
    );

    // In the v30_0 generated file, `emphasized` is defined as `$easing-standard`.
    cfg.easings.insert(
        "md.sys.motion.easing.emphasized".to_string(),
        CubicBezier {
            x1: 0.2,
            y1: 0.0,
            x2: 0.0,
            y2: 1.0,
        },
    );
}

fn inject_sys_shape(cfg: &mut ThemeConfig) {
    // Source: repo-ref/material-web/tokens/versions/v30_0/sass/_md-sys-shape.scss
    //
    // Note: Material also defines composite corner sets (e.g. `corner.large.top`) which require a
    // structured token kind. For now we only inject the single-radius variants into `metrics`.

    for (key, px) in [
        ("md.sys.shape.corner-value.none", 0.0),
        ("md.sys.shape.corner-value.extra-small", 4.0),
        ("md.sys.shape.corner-value.small", 8.0),
        ("md.sys.shape.corner-value.medium", 12.0),
        ("md.sys.shape.corner-value.large", 16.0),
        ("md.sys.shape.corner-value.large-increased", 20.0),
        ("md.sys.shape.corner-value.extra-large", 28.0),
        ("md.sys.shape.corner-value.extra-large-increased", 32.0),
        ("md.sys.shape.corner-value.extra-extra-large", 48.0),
        ("md.sys.shape.corner.none", 0.0),
        ("md.sys.shape.corner.extra-small", 4.0),
        ("md.sys.shape.corner.small", 8.0),
        ("md.sys.shape.corner.medium", 12.0),
        ("md.sys.shape.corner.large", 16.0),
        ("md.sys.shape.corner.large-increased", 20.0),
        ("md.sys.shape.corner.extra-large", 28.0),
        ("md.sys.shape.corner.extra-large-increased", 32.0),
        ("md.sys.shape.corner.extra-extra-large", 48.0),
        ("md.sys.shape.corner.full", 9999.0),
    ] {
        cfg.metrics.insert(key.to_string(), px);
    }
}

#[derive(Debug, Clone, Copy)]
struct TypescaleRoleRem {
    key: &'static str,
    size_rem: f32,
    line_height_rem: f32,
    tracking_rem: f32,
    weight: u16,
}

fn inject_sys_typescale(cfg: &mut ThemeConfig, typography: &TypographyOptions) {
    // Source: repo-ref/material-web/tokens/versions/v30_0/sass/_md-sys-typescale.scss
    //
    // We inject a *composed* `TextStyle` per role, keyed by `md.sys.typescale.<role>`.
    // This avoids making every widget compute line-height/tracking manually.

    let rem_in_px = typography.rem_in_px;
    let font = typography.font.clone();

    for role in [
        TypescaleRoleRem {
            key: "md.sys.typescale.display-large",
            size_rem: 3.5625,
            line_height_rem: 4.0,
            tracking_rem: -0.015625,
            weight: 400,
        },
        TypescaleRoleRem {
            key: "md.sys.typescale.display-medium",
            size_rem: 2.8125,
            line_height_rem: 3.25,
            tracking_rem: 0.0,
            weight: 400,
        },
        TypescaleRoleRem {
            key: "md.sys.typescale.display-small",
            size_rem: 2.25,
            line_height_rem: 2.75,
            tracking_rem: 0.0,
            weight: 400,
        },
        TypescaleRoleRem {
            key: "md.sys.typescale.headline-large",
            size_rem: 2.0,
            line_height_rem: 2.5,
            tracking_rem: 0.0,
            weight: 400,
        },
        TypescaleRoleRem {
            key: "md.sys.typescale.headline-medium",
            size_rem: 1.75,
            line_height_rem: 2.25,
            tracking_rem: 0.0,
            weight: 400,
        },
        TypescaleRoleRem {
            key: "md.sys.typescale.headline-small",
            size_rem: 1.5,
            line_height_rem: 2.0,
            tracking_rem: 0.0,
            weight: 400,
        },
        TypescaleRoleRem {
            key: "md.sys.typescale.title-large",
            size_rem: 1.375,
            line_height_rem: 1.75,
            tracking_rem: 0.0,
            weight: 400,
        },
        TypescaleRoleRem {
            key: "md.sys.typescale.title-medium",
            size_rem: 1.0,
            line_height_rem: 1.5,
            tracking_rem: 0.009375,
            weight: 500,
        },
        TypescaleRoleRem {
            key: "md.sys.typescale.title-small",
            size_rem: 0.875,
            line_height_rem: 1.25,
            tracking_rem: 0.00625,
            weight: 500,
        },
        TypescaleRoleRem {
            key: "md.sys.typescale.label-large",
            size_rem: 0.875,
            line_height_rem: 1.25,
            tracking_rem: 0.00625,
            weight: 500,
        },
        TypescaleRoleRem {
            key: "md.sys.typescale.label-medium",
            size_rem: 0.75,
            line_height_rem: 1.0,
            tracking_rem: 0.03125,
            weight: 500,
        },
        TypescaleRoleRem {
            key: "md.sys.typescale.label-small",
            size_rem: 0.6875,
            line_height_rem: 1.0,
            tracking_rem: 0.03125,
            weight: 500,
        },
        TypescaleRoleRem {
            key: "md.sys.typescale.body-large",
            size_rem: 1.0,
            line_height_rem: 1.5,
            tracking_rem: 0.03125,
            weight: 400,
        },
        TypescaleRoleRem {
            key: "md.sys.typescale.body-medium",
            size_rem: 0.875,
            line_height_rem: 1.25,
            tracking_rem: 0.015625,
            weight: 400,
        },
        TypescaleRoleRem {
            key: "md.sys.typescale.body-small",
            size_rem: 0.75,
            line_height_rem: 1.0,
            tracking_rem: 0.025,
            weight: 400,
        },
    ] {
        let size_px = Px(role.size_rem * rem_in_px);
        let line_height_px = Px(role.line_height_rem * rem_in_px);
        let tracking_em = if role.size_rem.abs() <= f32::EPSILON {
            0.0
        } else {
            role.tracking_rem / role.size_rem
        };

        cfg.text_styles.insert(
            role.key.to_string(),
            TextStyle {
                font: font.clone(),
                size: size_px,
                weight: FontWeight(role.weight),
                slant: TextSlant::Normal,
                line_height: Some(line_height_px),
                letter_spacing_em: Some(tracking_em),
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ColorSchemeOptions, DynamicVariant, SchemeMode, TypographyOptions, inject_sys_colors,
        inject_tokens, theme_config, theme_config_with_colors,
    };

    #[test]
    fn v30_injects_state_motion_and_typescale_tokens() {
        let cfg = theme_config(TypographyOptions::default());

        assert_eq!(
            cfg.numbers
                .get("md.sys.state.hover.state-layer-opacity")
                .copied(),
            Some(0.08)
        );
        assert_eq!(
            cfg.durations_ms
                .get("md.sys.motion.duration.short1")
                .copied(),
            Some(50)
        );
        assert!(
            cfg.easings.contains_key("md.sys.motion.easing.standard"),
            "expected standard easing token"
        );
        assert!(
            cfg.text_styles.contains_key("md.sys.typescale.body-large"),
            "expected composed body-large text style"
        );

        let title_medium = cfg
            .text_styles
            .get("md.sys.typescale.title-medium")
            .expect("expected title-medium text style");
        assert_eq!(title_medium.weight, fret_core::FontWeight(500));
        assert_eq!(title_medium.line_height, Some(fret_core::Px(24.0)));
        let spacing = title_medium
            .letter_spacing_em
            .expect("expected letter spacing");
        assert!(
            (spacing - 0.009375).abs() < 1e-6,
            "unexpected letter_spacing_em: {spacing}"
        );
        assert_eq!(
            cfg.metrics.get("md.sys.shape.corner.medium").copied(),
            Some(12.0)
        );
        assert_eq!(
            cfg.metrics
                .get("md.comp.button.small.container.height")
                .copied(),
            Some(40.0)
        );
        assert_eq!(
            cfg.metrics
                .get("md.comp.icon-button.small.container.height")
                .copied(),
            Some(40.0)
        );
        assert_eq!(
            cfg.metrics
                .get("md.sys.state.focus-indicator.thickness")
                .copied(),
            Some(3.0)
        );
        assert_eq!(
            cfg.metrics
                .get("md.comp.checkbox.state-layer.size")
                .copied(),
            Some(40.0)
        );
        assert_eq!(
            cfg.metrics.get("md.comp.checkbox.container.size").copied(),
            Some(18.0)
        );
        assert_eq!(
            cfg.metrics.get("md.comp.switch.track.width").copied(),
            Some(52.0)
        );
        assert_eq!(
            cfg.metrics.get("md.comp.switch.track.height").copied(),
            Some(32.0)
        );
        assert_eq!(
            cfg.metrics.get("md.comp.radio-button.icon.size").copied(),
            Some(20.0)
        );
        assert_eq!(
            cfg.metrics
                .get("md.comp.radio-button.state-layer.size")
                .copied(),
            Some(40.0)
        );
        assert_eq!(
            cfg.numbers
                .get("md.comp.button.filled.disabled.label-text.opacity")
                .copied(),
            Some(0.38)
        );
        assert_eq!(
            cfg.numbers
                .get("md.comp.icon-button.filled.disabled.container.opacity")
                .copied(),
            Some(0.1)
        );

        // Inject into an existing config should merge/overwrite.
        let mut cfg2 = fret_ui::theme::ThemeConfig::default();
        inject_tokens(&mut cfg2, &TypographyOptions::default());
        assert!(
            cfg2.text_styles
                .contains_key("md.sys.typescale.title-medium")
        );
    }

    #[test]
    fn v30_injects_sys_color_roles() {
        let cfg = theme_config_with_colors(
            TypographyOptions::default(),
            ColorSchemeOptions {
                mode: SchemeMode::Dark,
                ..Default::default()
            },
        );

        let primary = cfg
            .colors
            .get("md.sys.color.primary")
            .cloned()
            .expect("expected primary role");
        assert!(primary.starts_with('#'), "expected hex color string");
        assert!(
            primary.len() == 7 || primary.len() == 9,
            "expected #RRGGBB or #RRGGBBAA"
        );
        assert!(
            cfg.colors.contains_key("md.sys.color.on-primary"),
            "expected on-primary role"
        );
        assert!(
            cfg.colors.contains_key("md.sys.color.surface-container"),
            "expected surface-container role"
        );
        assert!(
            cfg.colors.contains_key("md.sys.color.outline-variant"),
            "expected outline-variant role"
        );
        assert!(
            cfg.colors
                .contains_key("md.comp.checkbox.selected.container.color"),
            "expected checkbox color tokens to be derived from sys roles"
        );
        assert!(
            cfg.colors
                .contains_key("md.comp.switch.selected.track.color"),
            "expected switch color tokens to be derived from sys roles"
        );
        assert!(
            cfg.colors
                .contains_key("md.comp.radio-button.selected.icon.color"),
            "expected radio-button color tokens to be derived from sys roles"
        );
    }

    #[test]
    fn v30_expressive_variant_changes_scheme_output() {
        let mut cfg_tonal = fret_ui::theme::ThemeConfig::default();
        inject_sys_colors(
            &mut cfg_tonal,
            ColorSchemeOptions {
                mode: SchemeMode::Dark,
                variant: DynamicVariant::TonalSpot,
                ..Default::default()
            },
        );

        let mut cfg_expressive = fret_ui::theme::ThemeConfig::default();
        inject_sys_colors(
            &mut cfg_expressive,
            ColorSchemeOptions {
                mode: SchemeMode::Dark,
                variant: DynamicVariant::Expressive,
                ..Default::default()
            },
        );

        let tonal_primary = cfg_tonal
            .colors
            .get("md.sys.color.primary")
            .cloned()
            .expect("expected primary role");
        let expressive_primary = cfg_expressive
            .colors
            .get("md.sys.color.primary")
            .cloned()
            .expect("expected primary role");

        assert_ne!(
            tonal_primary, expressive_primary,
            "expected expressive scheme to differ"
        );
    }
}
