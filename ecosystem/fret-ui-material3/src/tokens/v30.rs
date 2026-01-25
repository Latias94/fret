//! Material 3 token preset for Material Web `tokens/versions/v30_0` (web/static font context).
//!
//! This module intentionally targets **outcome alignment** (visual + interaction) and exposes a
//! stable "inject tokens into ThemeConfig" surface. It does not attempt to mirror the
//! `@material/web` API or DOM/Lit implementation details.

use fret_core::{Corners, FontId, Px};
use fret_ui::theme::ThemeConfig;
use material_colors::color::Argb;
use material_colors::dynamic_color::Variant as MaterialVariant;
use material_colors::theme::ThemeBuilder;

use super::material_web_v30;

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
    /// Typeface used for roles that map to `md.ref.typeface.plain`.
    pub plain_font: FontId,
    /// Typeface used for roles that map to `md.ref.typeface.brand`.
    pub brand_font: FontId,
}

impl Default for TypographyOptions {
    fn default() -> Self {
        Self {
            rem_in_px: 16.0,
            plain_font: FontId::Ui,
            brand_font: FontId::Ui,
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
    // Compose `minimumInteractiveComponentSize()` default (48dp).
    cfg.metrics
        .insert("md.sys.layout.minimum-touch-target.size".to_string(), 48.0);

    material_web_v30::inject_sys_state(cfg);
    material_web_v30::inject_sys_state_focus_indicator(cfg);
    material_web_v30::inject_sys_motion(cfg);
    material_web_v30::inject_sys_shape(cfg);
    material_web_v30::inject_sys_typescale(cfg, typography);
    inject_comp_button_scalars(cfg);
    inject_comp_icon_button_scalars(cfg);
    inject_comp_checkbox_scalars(cfg);
    inject_comp_switch_scalars(cfg);
    inject_comp_radio_button_scalars(cfg);
    inject_comp_outlined_text_field_scalars(cfg);
    inject_comp_filled_text_field_scalars(cfg);
    inject_comp_primary_navigation_tab_scalars(cfg);
    inject_comp_navigation_bar_scalars(cfg);
    inject_comp_navigation_drawer_scalars(cfg);
    inject_comp_navigation_rail_scalars(cfg);
    inject_comp_menu_scalars(cfg);
    inject_comp_list_scalars(cfg);
    inject_comp_plain_tooltip_scalars(cfg);
    inject_comp_rich_tooltip_scalars(cfg);
    inject_comp_snackbar_scalars(cfg);
    inject_comp_dialog_scalars(cfg);
    inject_comp_full_screen_dialog_scalars(cfg);

    // Material Web v30 notes that the navigation drawer scrim tokens are deprecated and do not
    // represent the intended M3 defaults. Prefer Neutral-Variant10 at 50% opacity for scrims.
    cfg.numbers
        .insert("md.comp.navigation-drawer.scrim.opacity".to_string(), 0.5);
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

    // Palette tokens (limited subset) for outcomes that are specified in terms of raw palette
    // tones rather than scheme roles (e.g. scrims).
    insert_color(
        cfg,
        "md.ref.palette.neutral-variant10",
        theme.palettes.neutral_variant.tone(10),
    );

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
    inject_comp_primary_navigation_tab_colors_from_sys(&mut cfg);
    inject_comp_navigation_bar_colors_from_sys(&mut cfg);
    inject_comp_navigation_drawer_colors_from_sys(&mut cfg);
    inject_comp_navigation_rail_colors_from_sys(&mut cfg);
    inject_comp_menu_colors_from_sys(&mut cfg);
    inject_comp_list_colors_from_sys(&mut cfg);
    inject_comp_plain_tooltip_colors_from_sys(&mut cfg);
    inject_comp_rich_tooltip_colors_from_sys(&mut cfg);
    inject_comp_snackbar_colors_from_sys(&mut cfg);
    inject_comp_dialog_colors_from_sys(&mut cfg);
    inject_comp_full_screen_dialog_colors_from_sys(&mut cfg);
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
    material_web_v30::inject_comp_button_scalars(cfg);
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
    material_web_v30::inject_comp_icon_button_scalars(cfg);
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

fn copy_number(cfg: &mut ThemeConfig, to_key: &str, from_key: &str) {
    let Some(v) = cfg.numbers.get(from_key).copied() else {
        return;
    };
    cfg.numbers.insert(to_key.to_string(), v);
}

fn inject_comp_checkbox_scalars(cfg: &mut ThemeConfig) {
    material_web_v30::inject_comp_checkbox_scalars(cfg);
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
    material_web_v30::inject_comp_switch_scalars(cfg);
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
    material_web_v30::inject_comp_outlined_text_field_scalars(cfg);
}

fn inject_comp_filled_text_field_scalars(cfg: &mut ThemeConfig) {
    material_web_v30::inject_comp_filled_text_field_scalars(cfg);
}

fn inject_comp_primary_navigation_tab_scalars(cfg: &mut ThemeConfig) {
    material_web_v30::inject_comp_primary_navigation_tab_scalars(cfg);
}

fn inject_comp_navigation_bar_scalars(cfg: &mut ThemeConfig) {
    material_web_v30::inject_comp_navigation_bar_scalars(cfg);
}

fn inject_comp_navigation_drawer_scalars(cfg: &mut ThemeConfig) {
    material_web_v30::inject_comp_navigation_drawer_scalars(cfg);
}

fn inject_comp_navigation_rail_scalars(cfg: &mut ThemeConfig) {
    material_web_v30::inject_comp_navigation_rail_scalars(cfg);
}

fn inject_comp_menu_scalars(cfg: &mut ThemeConfig) {
    material_web_v30::inject_comp_menu_scalars(cfg);
}

fn inject_comp_list_scalars(cfg: &mut ThemeConfig) {
    // Source: repo-ref/material-web/tokens/versions/v30_0/sass/_md-comp-list.scss

    cfg.metrics.insert(
        "md.comp.list.list-item.one-line.container.height".to_string(),
        56.0,
    );
    cfg.metrics.insert(
        "md.comp.list.list-item.two-line.container.height".to_string(),
        72.0,
    );
    cfg.metrics.insert(
        "md.comp.list.list-item.three-line.container.height".to_string(),
        88.0,
    );

    cfg.metrics
        .insert("md.comp.list.list-item.leading-space".to_string(), 16.0);
    cfg.metrics
        .insert("md.comp.list.list-item.trailing-space".to_string(), 16.0);
    cfg.metrics
        .insert("md.comp.list.list-item.between-space".to_string(), 12.0);
    cfg.metrics
        .insert("md.comp.list.list-item.top-space".to_string(), 10.0);
    cfg.metrics
        .insert("md.comp.list.list-item.bottom-space".to_string(), 10.0);

    cfg.metrics
        .insert("md.comp.list.list-item.leading-icon.size".to_string(), 24.0);
    cfg.metrics.insert(
        "md.comp.list.list-item.trailing-icon.size".to_string(),
        24.0,
    );

    cfg.corners.insert(
        "md.comp.list.list-item.container.shape".to_string(),
        Corners::all(Px(0.0)),
    );
    cfg.corners.insert(
        "md.comp.list.list-item.selected.container.shape".to_string(),
        Corners::all(Px(0.0)),
    );
}

fn inject_comp_plain_tooltip_scalars(cfg: &mut ThemeConfig) {
    material_web_v30::inject_comp_plain_tooltip_scalars(cfg);
}

fn inject_comp_rich_tooltip_scalars(cfg: &mut ThemeConfig) {
    material_web_v30::inject_comp_rich_tooltip_scalars(cfg);
}

fn inject_comp_snackbar_scalars(cfg: &mut ThemeConfig) {
    material_web_v30::inject_comp_snackbar_scalars(cfg);
}

fn inject_comp_dialog_scalars(cfg: &mut ThemeConfig) {
    material_web_v30::inject_comp_dialog_scalars(cfg);
}

fn inject_comp_full_screen_dialog_scalars(cfg: &mut ThemeConfig) {
    material_web_v30::inject_comp_full_screen_dialog_scalars(cfg);
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
        "md.comp.outlined-text-field.error.input-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.outlined-text-field.error.focus.input-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.outlined-text-field.error.hover.input-text.color",
        "md.sys.color.on-surface",
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
        "md.comp.outlined-text-field.error.hover.outline.color",
        "md.sys.color.on-error-container",
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

fn inject_comp_primary_navigation_tab_colors_from_sys(cfg: &mut ThemeConfig) {
    // Source: repo-ref/material-web/tokens/versions/v30_0/sass/_md-comp-primary-navigation-tab.scss

    copy_color(
        cfg,
        "md.comp.primary-navigation-tab.container.color",
        "md.sys.color.surface",
    );
    copy_color(
        cfg,
        "md.comp.primary-navigation-tab.active-indicator.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.primary-navigation-tab.focus.indicator.color",
        "md.sys.color.secondary",
    );

    copy_color(
        cfg,
        "md.comp.primary-navigation-tab.active.focus.state-layer.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.primary-navigation-tab.active.hover.state-layer.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.primary-navigation-tab.active.pressed.state-layer.color",
        "md.sys.color.primary",
    );

    copy_color(
        cfg,
        "md.comp.primary-navigation-tab.inactive.focus.state-layer.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.primary-navigation-tab.inactive.hover.state-layer.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.primary-navigation-tab.inactive.pressed.state-layer.color",
        "md.sys.color.primary",
    );

    copy_color(
        cfg,
        "md.comp.primary-navigation-tab.with-label-text.active.label-text.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.primary-navigation-tab.with-label-text.active.focus.label-text.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.primary-navigation-tab.with-label-text.active.hover.label-text.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.primary-navigation-tab.with-label-text.active.pressed.label-text.color",
        "md.sys.color.primary",
    );

    copy_color(
        cfg,
        "md.comp.primary-navigation-tab.with-label-text.inactive.label-text.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.primary-navigation-tab.with-label-text.inactive.focus.label-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.primary-navigation-tab.with-label-text.inactive.hover.label-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.primary-navigation-tab.with-label-text.inactive.pressed.label-text.color",
        "md.sys.color.on-surface",
    );
}

fn inject_comp_navigation_bar_colors_from_sys(cfg: &mut ThemeConfig) {
    copy_color(
        cfg,
        "md.comp.navigation-bar.active-indicator.color",
        "md.sys.color.secondary-container",
    );
    copy_color(
        cfg,
        "md.comp.navigation-bar.active.focus.icon.color",
        "md.sys.color.on-secondary-container",
    );
    copy_color(
        cfg,
        "md.comp.navigation-bar.active.focus.label-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-bar.active.focus.state-layer.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-bar.active.hover.icon.color",
        "md.sys.color.on-secondary-container",
    );
    copy_color(
        cfg,
        "md.comp.navigation-bar.active.hover.label-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-bar.active.hover.state-layer.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-bar.active.icon.color",
        "md.sys.color.on-secondary-container",
    );
    copy_color(
        cfg,
        "md.comp.navigation-bar.active.label-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-bar.active.pressed.icon.color",
        "md.sys.color.on-secondary-container",
    );
    copy_color(
        cfg,
        "md.comp.navigation-bar.active.pressed.label-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-bar.active.pressed.state-layer.color",
        "md.sys.color.on-surface",
    );

    copy_color(
        cfg,
        "md.comp.navigation-bar.container.color",
        "md.sys.color.surface-container",
    );
    copy_color(
        cfg,
        "md.comp.navigation-bar.container.shadow-color",
        "md.sys.color.shadow",
    );
    copy_color(
        cfg,
        "md.comp.navigation-bar.container.surface-tint-layer.color",
        "md.sys.color.surface-tint",
    );

    copy_color(
        cfg,
        "md.comp.navigation-bar.focus.indicator.color",
        "md.sys.color.secondary",
    );

    copy_color(
        cfg,
        "md.comp.navigation-bar.inactive.focus.icon.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-bar.inactive.focus.label-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-bar.inactive.focus.state-layer.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-bar.inactive.hover.icon.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-bar.inactive.hover.label-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-bar.inactive.hover.state-layer.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-bar.inactive.icon.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.navigation-bar.inactive.label-text.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.navigation-bar.inactive.pressed.icon.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-bar.inactive.pressed.label-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-bar.inactive.pressed.state-layer.color",
        "md.sys.color.on-surface",
    );
}

fn inject_comp_navigation_drawer_colors_from_sys(cfg: &mut ThemeConfig) {
    copy_color(
        cfg,
        "md.comp.navigation-drawer.active-indicator.color",
        "md.sys.color.secondary-container",
    );
    copy_color(
        cfg,
        "md.comp.navigation-drawer.active.focus.icon.color",
        "md.sys.color.on-secondary-container",
    );
    copy_color(
        cfg,
        "md.comp.navigation-drawer.active.focus.label-text.color",
        "md.sys.color.on-secondary-container",
    );
    copy_color(
        cfg,
        "md.comp.navigation-drawer.active.focus.state-layer.color",
        "md.sys.color.on-secondary-container",
    );
    copy_color(
        cfg,
        "md.comp.navigation-drawer.active.hover.icon.color",
        "md.sys.color.on-secondary-container",
    );
    copy_color(
        cfg,
        "md.comp.navigation-drawer.active.hover.label-text.color",
        "md.sys.color.on-secondary-container",
    );
    copy_color(
        cfg,
        "md.comp.navigation-drawer.active.hover.state-layer.color",
        "md.sys.color.on-secondary-container",
    );
    copy_color(
        cfg,
        "md.comp.navigation-drawer.active.icon.color",
        "md.sys.color.on-secondary-container",
    );
    copy_color(
        cfg,
        "md.comp.navigation-drawer.active.label-text.color",
        "md.sys.color.on-secondary-container",
    );
    copy_color(
        cfg,
        "md.comp.navigation-drawer.active.pressed.icon.color",
        "md.sys.color.on-secondary-container",
    );
    copy_color(
        cfg,
        "md.comp.navigation-drawer.active.pressed.label-text.color",
        "md.sys.color.on-secondary-container",
    );
    copy_color(
        cfg,
        "md.comp.navigation-drawer.active.pressed.state-layer.color",
        "md.sys.color.on-secondary-container",
    );

    copy_color(
        cfg,
        "md.comp.navigation-drawer.container.color",
        "md.sys.color.surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-drawer.standard.container.color",
        "md.sys.color.surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-drawer.modal.container.color",
        "md.sys.color.surface-container-low",
    );
    copy_color(
        cfg,
        "md.comp.navigation-drawer.container.surface-tint-layer.color",
        "md.sys.color.surface-tint",
    );
    copy_color(
        cfg,
        "md.comp.navigation-drawer.divider.color",
        "md.sys.color.outline",
    );

    copy_color(
        cfg,
        "md.comp.navigation-drawer.focus.indicator.color",
        "md.sys.color.secondary",
    );

    copy_color(
        cfg,
        "md.comp.navigation-drawer.headline.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.navigation-drawer.large-badge-label.color",
        "md.sys.color.on-surface-variant",
    );

    copy_color(
        cfg,
        "md.comp.navigation-drawer.inactive.focus.icon.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-drawer.inactive.focus.label-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-drawer.inactive.focus.state-layer.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-drawer.inactive.hover.icon.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-drawer.inactive.hover.label-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-drawer.inactive.hover.state-layer.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-drawer.inactive.icon.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.navigation-drawer.inactive.label-text.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.navigation-drawer.inactive.pressed.icon.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-drawer.inactive.pressed.label-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-drawer.inactive.pressed.state-layer.color",
        "md.sys.color.on-surface",
    );

    copy_color(
        cfg,
        "md.comp.navigation-drawer.scrim.color",
        "md.ref.palette.neutral-variant10",
    );
}

fn inject_comp_navigation_rail_colors_from_sys(cfg: &mut ThemeConfig) {
    // Source: repo-ref/material-web/tokens/versions/v30_0/sass/_md-comp-navigation-rail.scss

    copy_color(
        cfg,
        "md.comp.navigation-rail.active-indicator.color",
        "md.sys.color.secondary-container",
    );
    copy_color(
        cfg,
        "md.comp.navigation-rail.active.focus.icon.color",
        "md.sys.color.on-secondary-container",
    );
    copy_color(
        cfg,
        "md.comp.navigation-rail.active.focus.label-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-rail.active.focus.state-layer.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-rail.active.hover.icon.color",
        "md.sys.color.on-secondary-container",
    );
    copy_color(
        cfg,
        "md.comp.navigation-rail.active.hover.label-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-rail.active.hover.state-layer.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-rail.active.icon.color",
        "md.sys.color.on-secondary-container",
    );
    copy_color(
        cfg,
        "md.comp.navigation-rail.active.label-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-rail.active.pressed.icon.color",
        "md.sys.color.on-secondary-container",
    );
    copy_color(
        cfg,
        "md.comp.navigation-rail.active.pressed.label-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-rail.active.pressed.state-layer.color",
        "md.sys.color.on-surface",
    );

    copy_color(
        cfg,
        "md.comp.navigation-rail.container.color",
        "md.sys.color.surface",
    );

    copy_color(
        cfg,
        "md.comp.navigation-rail.inactive.focus.icon.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-rail.inactive.focus.label-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-rail.inactive.focus.state-layer.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-rail.inactive.hover.icon.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-rail.inactive.hover.label-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-rail.inactive.hover.state-layer.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-rail.inactive.icon.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.navigation-rail.inactive.label-text.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.navigation-rail.inactive.pressed.icon.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-rail.inactive.pressed.label-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.navigation-rail.inactive.pressed.state-layer.color",
        "md.sys.color.on-surface",
    );
}

fn inject_comp_menu_colors_from_sys(cfg: &mut ThemeConfig) {
    // Source: repo-ref/material-web/tokens/versions/v30_0/sass/_md-comp-menu.scss

    copy_color(
        cfg,
        "md.comp.menu.container.color",
        "md.sys.color.surface-container",
    );
    // Not a Material Web v30 token, but a convenient escape hatch for renderer-level shadow tuning.
    // Defaults to `md.sys.color.shadow`.
    copy_color(
        cfg,
        "md.comp.menu.container.shadow-color",
        "md.sys.color.shadow",
    );
    copy_color(
        cfg,
        "md.comp.menu.divider.color",
        "md.sys.color.surface-variant",
    );

    copy_color(
        cfg,
        "md.comp.menu.list-item.label-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.menu.list-item.hover.label-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.menu.list-item.focus.label-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.menu.list-item.pressed.label-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.menu.list-item.disabled.label-text.color",
        "md.sys.color.on-surface",
    );

    copy_color(
        cfg,
        "md.comp.menu.list-item.hover.state-layer.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.menu.list-item.focus.state-layer.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.menu.list-item.pressed.state-layer.color",
        "md.sys.color.on-surface",
    );
}

fn inject_comp_list_colors_from_sys(cfg: &mut ThemeConfig) {
    // Source: repo-ref/material-web/tokens/versions/v30_0/sass/_md-comp-list.scss

    copy_color(
        cfg,
        "md.comp.list.list-item.container.color",
        "md.sys.color.surface",
    );
    copy_color(
        cfg,
        "md.comp.list.list-item.label-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.list.list-item.leading-icon.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.list.list-item.pressed.leading-icon.icon.color",
        "md.sys.color.on-surface-variant",
    );

    copy_color(
        cfg,
        "md.comp.list.list-item.hover.label-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.list.list-item.focus.label-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.list.list-item.pressed.label-text.color",
        "md.sys.color.on-surface",
    );

    copy_color(
        cfg,
        "md.comp.list.list-item.hover.state-layer.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.list.list-item.focus.state-layer.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.list.list-item.pressed.state-layer.color",
        "md.sys.color.on-surface",
    );

    copy_color(
        cfg,
        "md.comp.list.list-item.disabled.label-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.list.list-item.disabled.leading-icon.color",
        "md.sys.color.on-surface",
    );

    copy_color(
        cfg,
        "md.comp.list.list-item.selected.container.color",
        "md.sys.color.secondary-container",
    );
    copy_color(
        cfg,
        "md.comp.list.list-item.selected.label-text.color",
        "md.sys.color.on-secondary-container",
    );
    copy_color(
        cfg,
        "md.comp.list.list-item.selected.leading-icon.color",
        "md.sys.color.on-surface",
    );

    copy_color(
        cfg,
        "md.comp.list.list-item.selected.hover.label-text.color",
        "md.sys.color.on-secondary-container",
    );
    copy_color(
        cfg,
        "md.comp.list.list-item.selected.focus.label-text.color",
        "md.sys.color.on-secondary-container",
    );
    copy_color(
        cfg,
        "md.comp.list.list-item.selected.pressed.label-text.color",
        "md.sys.color.on-secondary-container",
    );

    copy_color(
        cfg,
        "md.comp.list.list-item.selected.hover.state-layer.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.list.list-item.selected.focus.state-layer.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.list.list-item.selected.pressed.state-layer.color",
        "md.sys.color.on-surface",
    );

    copy_color(
        cfg,
        "md.comp.list.list-item.selected.pressed.leading-icon.color",
        "md.sys.color.on-surface",
    );

    copy_color(
        cfg,
        "md.comp.list.list-item.selected.disabled.container.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.list.list-item.selected.disabled.label-text.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.list.list-item.selected.disabled.leading-icon.color",
        "md.sys.color.on-surface",
    );

    copy_color(
        cfg,
        "md.comp.list.focus.indicator.color",
        "md.sys.color.secondary",
    );

    // Opacity tokens used by outcomes.
    copy_number(
        cfg,
        "md.comp.list.list-item.hover.state-layer.opacity",
        "md.sys.state.hover.state-layer-opacity",
    );
    copy_number(
        cfg,
        "md.comp.list.list-item.focus.state-layer.opacity",
        "md.sys.state.focus.state-layer-opacity",
    );
    copy_number(
        cfg,
        "md.comp.list.list-item.pressed.state-layer.opacity",
        "md.sys.state.pressed.state-layer-opacity",
    );
    copy_number(
        cfg,
        "md.comp.list.list-item.selected.hover.state-layer.opacity",
        "md.sys.state.hover.state-layer-opacity",
    );
    copy_number(
        cfg,
        "md.comp.list.list-item.selected.focus.state-layer.opacity",
        "md.sys.state.focus.state-layer-opacity",
    );
    copy_number(
        cfg,
        "md.comp.list.list-item.selected.pressed.state-layer.opacity",
        "md.sys.state.pressed.state-layer-opacity",
    );

    copy_number(
        cfg,
        "md.comp.list.list-item.disabled.label-text.opacity",
        "md.sys.state.disabled.state-layer-opacity",
    );
    copy_number(
        cfg,
        "md.comp.list.list-item.disabled.leading-icon.opacity",
        "md.sys.state.disabled.state-layer-opacity",
    );

    copy_number(
        cfg,
        "md.comp.list.list-item.selected.disabled.label-text.opacity",
        "md.sys.state.disabled.state-layer-opacity",
    );
    copy_number(
        cfg,
        "md.comp.list.list-item.selected.disabled.leading-icon.opacity",
        "md.sys.state.disabled.state-layer-opacity",
    );
    copy_number(
        cfg,
        "md.comp.list.list-item.selected.disabled.container.opacity",
        "md.sys.state.disabled.state-layer-opacity",
    );
}

fn inject_comp_plain_tooltip_colors_from_sys(cfg: &mut ThemeConfig) {
    // Source: repo-ref/material-web/tokens/versions/v30_0/sass/_md-comp-plain-tooltip.scss

    copy_color(
        cfg,
        "md.comp.plain-tooltip.container.color",
        "md.sys.color.inverse-surface",
    );
    copy_color(
        cfg,
        "md.comp.plain-tooltip.supporting-text.color",
        "md.sys.color.inverse-on-surface",
    );
}

fn inject_comp_rich_tooltip_colors_from_sys(cfg: &mut ThemeConfig) {
    // Source: repo-ref/material-web/tokens/versions/v30_0/sass/_md-comp-rich-tooltip.scss

    copy_color(
        cfg,
        "md.comp.rich-tooltip.action.label-text.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.rich-tooltip.action.hover.label-text.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.rich-tooltip.action.hover.state-layer.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.rich-tooltip.action.focus.label-text.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.rich-tooltip.action.focus.state-layer.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.rich-tooltip.action.pressed.label-text.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.rich-tooltip.action.pressed.state-layer.color",
        "md.sys.color.primary",
    );

    copy_color(
        cfg,
        "md.comp.rich-tooltip.container.color",
        "md.sys.color.surface-container",
    );
    copy_color(
        cfg,
        "md.comp.rich-tooltip.container.shadow-color",
        "md.sys.color.shadow",
    );
    copy_color(
        cfg,
        "md.comp.rich-tooltip.container.surface-tint-layer.color",
        "md.sys.color.surface-tint",
    );

    copy_color(
        cfg,
        "md.comp.rich-tooltip.subhead.color",
        "md.sys.color.on-surface-variant",
    );
    copy_color(
        cfg,
        "md.comp.rich-tooltip.supporting-text.color",
        "md.sys.color.on-surface-variant",
    );
}

fn inject_comp_snackbar_colors_from_sys(cfg: &mut ThemeConfig) {
    // Source: repo-ref/material-web/tokens/versions/v30_0/sass/_md-comp-snackbar.scss

    copy_color(
        cfg,
        "md.comp.snackbar.container.color",
        "md.sys.color.inverse-surface",
    );
    copy_color(
        cfg,
        "md.comp.snackbar.container.shadow-color",
        "md.sys.color.shadow",
    );

    copy_color(
        cfg,
        "md.comp.snackbar.supporting-text.color",
        "md.sys.color.inverse-on-surface",
    );

    copy_color(
        cfg,
        "md.comp.snackbar.action.label-text.color",
        "md.sys.color.inverse-primary",
    );
    copy_color(
        cfg,
        "md.comp.snackbar.action.hover.label-text.color",
        "md.sys.color.inverse-primary",
    );
    copy_color(
        cfg,
        "md.comp.snackbar.action.hover.state-layer.color",
        "md.sys.color.inverse-primary",
    );
    copy_color(
        cfg,
        "md.comp.snackbar.action.focus.label-text.color",
        "md.sys.color.inverse-primary",
    );
    copy_color(
        cfg,
        "md.comp.snackbar.action.focus.state-layer.color",
        "md.sys.color.inverse-primary",
    );
    copy_color(
        cfg,
        "md.comp.snackbar.action.pressed.label-text.color",
        "md.sys.color.inverse-primary",
    );
    copy_color(
        cfg,
        "md.comp.snackbar.action.pressed.state-layer.color",
        "md.sys.color.inverse-primary",
    );

    copy_color(
        cfg,
        "md.comp.snackbar.icon.color",
        "md.sys.color.inverse-on-surface",
    );
    copy_color(
        cfg,
        "md.comp.snackbar.icon.hover.icon.color",
        "md.sys.color.inverse-on-surface",
    );
    copy_color(
        cfg,
        "md.comp.snackbar.icon.hover.state-layer.color",
        "md.sys.color.inverse-on-surface",
    );
    copy_color(
        cfg,
        "md.comp.snackbar.icon.focus.icon.color",
        "md.sys.color.inverse-on-surface",
    );
    copy_color(
        cfg,
        "md.comp.snackbar.icon.focus.state-layer.color",
        "md.sys.color.inverse-on-surface",
    );
    copy_color(
        cfg,
        "md.comp.snackbar.icon.pressed.icon.color",
        "md.sys.color.inverse-on-surface",
    );
    copy_color(
        cfg,
        "md.comp.snackbar.icon.pressed.state-layer.color",
        "md.sys.color.inverse-on-surface",
    );
}

fn inject_comp_dialog_colors_from_sys(cfg: &mut ThemeConfig) {
    // Source: repo-ref/material-web/tokens/versions/v30_0/sass/_md-comp-dialog.scss

    copy_color(
        cfg,
        "md.comp.dialog.container.color",
        "md.sys.color.surface-container-high",
    );
    // Not a Material Web v30 token, but a convenient escape hatch for renderer-level shadow tuning.
    // Defaults to `md.sys.color.shadow`.
    copy_color(
        cfg,
        "md.comp.dialog.container.shadow-color",
        "md.sys.color.shadow",
    );
    copy_color(
        cfg,
        "md.comp.dialog.container.surface-tint-layer.color",
        "md.sys.color.surface-tint",
    );
    copy_color(
        cfg,
        "md.comp.dialog.headline.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.dialog.supporting-text.color",
        "md.sys.color.on-surface-variant",
    );

    copy_color(
        cfg,
        "md.comp.dialog.action.label-text.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.dialog.action.hover.label-text.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.dialog.action.focus.label-text.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.dialog.action.pressed.label-text.color",
        "md.sys.color.primary",
    );

    copy_color(
        cfg,
        "md.comp.dialog.action.hover.state-layer.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.dialog.action.focus.state-layer.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.dialog.action.pressed.state-layer.color",
        "md.sys.color.primary",
    );

    copy_color(
        cfg,
        "md.comp.dialog.with-icon.icon.color",
        "md.sys.color.secondary",
    );
    copy_color(
        cfg,
        "md.comp.dialog.with-divider.divider.color",
        "md.sys.color.outline",
    );

    // Deprecated tokens preserved for compatibility with the upstream sassvars.
    copy_color(
        cfg,
        "md.comp.dialog.subhead.color",
        "md.sys.color.on-surface",
    );
}

fn inject_comp_full_screen_dialog_colors_from_sys(cfg: &mut ThemeConfig) {
    // Source: repo-ref/material-web/tokens/versions/v30_0/sass/_md-comp-full-screen-dialog.scss

    copy_color(
        cfg,
        "md.comp.full-screen-dialog.action-bar.container.color",
        "md.sys.color.surface",
    );
    copy_color(
        cfg,
        "md.comp.full-screen-dialog.action-bar.label-text.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.full-screen-dialog.action-bar.hover.label-text.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.full-screen-dialog.action-bar.focus.label-text.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.full-screen-dialog.action-bar.pressed.label-text.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.full-screen-dialog.action-bar.hover.state-layer.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.full-screen-dialog.action-bar.focus.state-layer.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.full-screen-dialog.action-bar.pressed.state-layer.color",
        "md.sys.color.primary",
    );

    copy_color(
        cfg,
        "md.comp.full-screen-dialog.header.container.color",
        "md.sys.color.surface",
    );
    copy_color(
        cfg,
        "md.comp.full-screen-dialog.header.container.surface-tint-layer.color",
        "md.sys.color.surface-tint",
    );
    copy_color(
        cfg,
        "md.comp.full-screen-dialog.header.headline.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.full-screen-dialog.header.icon.color",
        "md.sys.color.on-surface",
    );
    copy_color(
        cfg,
        "md.comp.full-screen-dialog.header.on-scroll.container.color",
        "md.sys.color.surface-container",
    );

    copy_color(
        cfg,
        "md.comp.full-screen-dialog.header.action.label-text.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.full-screen-dialog.header.action.hover.label-text.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.full-screen-dialog.header.action.focus.label-text.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.full-screen-dialog.header.action.pressed.label-text.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.full-screen-dialog.header.action.hover.state-layer.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.full-screen-dialog.header.action.focus.state-layer.color",
        "md.sys.color.primary",
    );
    copy_color(
        cfg,
        "md.comp.full-screen-dialog.header.action.pressed.state-layer.color",
        "md.sys.color.primary",
    );

    copy_color(
        cfg,
        "md.comp.full-screen-dialog.with-divider.divider.color",
        "md.sys.color.surface-variant",
    );
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
        assert_eq!(
            cfg.metrics
                .get("md.sys.layout.minimum-touch-target.size")
                .copied(),
            Some(48.0)
        );
        assert_eq!(
            cfg.numbers
                .get("md.sys.motion.spring.fast.spatial.stiffness")
                .copied(),
            Some(1400.0)
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
        assert!(
            cfg.corners
                .contains_key("md.sys.shape.corner.extra-small.top"),
            "expected corner set token"
        );
        assert!(
            cfg.corners
                .contains_key("md.comp.primary-navigation-tab.active-indicator.shape"),
            "expected component corner set token"
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

    #[test]
    fn v30_navigation_drawer_scrim_defaults_follow_material_web_note() {
        let cfg = theme_config_with_colors(
            TypographyOptions::default(),
            ColorSchemeOptions {
                mode: SchemeMode::Dark,
                variant: DynamicVariant::TonalSpot,
                ..Default::default()
            },
        );

        assert_eq!(
            cfg.numbers
                .get("md.comp.navigation-drawer.scrim.opacity")
                .copied(),
            Some(0.5)
        );

        let palette = cfg
            .colors
            .get("md.ref.palette.neutral-variant10")
            .cloned()
            .expect("expected palette token for neutral-variant10");
        let scrim = cfg
            .colors
            .get("md.comp.navigation-drawer.scrim.color")
            .cloned()
            .expect("expected navigation-drawer scrim color token");
        assert_eq!(scrim, palette);
    }
}
