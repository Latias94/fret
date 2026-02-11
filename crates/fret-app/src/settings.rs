use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

pub type FontsSettingsV1 = fret_core::TextFontFamilyConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MenuBarSettingsV1 {
    pub os: MenuBarIntegrationModeV1,
    pub in_window: MenuBarIntegrationModeV1,
}

impl Default for MenuBarSettingsV1 {
    fn default() -> Self {
        Self {
            os: MenuBarIntegrationModeV1::Auto,
            in_window: MenuBarIntegrationModeV1::Auto,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LocaleSettingsV1 {
    pub primary: String,
    pub fallbacks: Vec<String>,
    pub pseudo: bool,
}

impl Default for LocaleSettingsV1 {
    fn default() -> Self {
        Self {
            primary: "en-US".to_string(),
            fallbacks: Vec::new(),
            pseudo: false,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MenuBarIntegrationModeV1 {
    /// Use platform defaults:
    /// - OS menubar on Windows/macOS
    /// - in-window menubar on Linux/Web
    #[default]
    Auto,
    /// Enable this surface (best-effort on platforms without an OS menubar mapping).
    On,
    /// Disable this surface.
    Off,
}

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("failed to read settings file: {path}")]
    Read {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to parse settings file JSON: {path}")]
    Parse {
        path: String,
        source: serde_json::Error,
    },
}

/// Project/user settings file (v1).
///
/// This is intentionally small and strongly typed. See ADR 0014.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SettingsFileV1 {
    pub settings_version: u32,
    pub fonts: FontsSettingsV1,
    pub docking: DockingSettingsV1,
    pub menu_bar: MenuBarSettingsV1,
    pub locale: LocaleSettingsV1,
}

impl Default for SettingsFileV1 {
    fn default() -> Self {
        Self {
            settings_version: 1,
            fonts: FontsSettingsV1::default(),
            docking: DockingSettingsV1::default(),
            menu_bar: MenuBarSettingsV1::default(),
            locale: LocaleSettingsV1::default(),
        }
    }
}

impl SettingsFileV1 {
    pub fn load_json(path: impl AsRef<Path>) -> Result<Self, SettingsError> {
        let path = path.as_ref();
        let bytes = std::fs::read(path).map_err(|source| SettingsError::Read {
            path: path.display().to_string(),
            source,
        })?;
        serde_json::from_slice(&bytes).map_err(|source| SettingsError::Parse {
            path: path.display().to_string(),
            source,
        })
    }

    pub fn load_json_if_exists(path: impl AsRef<Path>) -> Result<Option<Self>, SettingsError> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(None);
        }
        Self::load_json(path).map(Some)
    }

    pub fn load_json_value(path: impl AsRef<Path>) -> Result<Value, SettingsError> {
        let path = path.as_ref();
        let bytes = std::fs::read(path).map_err(|source| SettingsError::Read {
            path: path.display().to_string(),
            source,
        })?;
        serde_json::from_slice(&bytes).map_err(|source| SettingsError::Parse {
            path: path.display().to_string(),
            source,
        })
    }

    pub fn load_json_value_if_exists(
        path: impl AsRef<Path>,
    ) -> Result<Option<Value>, SettingsError> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(None);
        }
        Self::load_json_value(path).map(Some)
    }

    pub fn resolved_locales(&self) -> Vec<fret_runtime::fret_i18n::LocaleId> {
        let mut out = Vec::new();

        let primary = parse_locale_or_default(&self.locale.primary);
        out.push(primary);

        for fallback in &self.locale.fallbacks {
            let Some(locale) = parse_locale_if_well_formed(fallback) else {
                continue;
            };
            if !out.contains(&locale) {
                out.push(locale);
            }
        }

        if out.is_empty() {
            out.push(fret_runtime::fret_i18n::LocaleId::default());
        }

        out
    }

    pub fn i18n_service(&self) -> fret_runtime::fret_i18n::I18nService {
        let mut service = fret_runtime::fret_i18n::I18nService::new(self.resolved_locales());
        service.set_pseudo_enabled(self.locale.pseudo);
        service
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DockingSettingsV1 {
    pub drag_inversion: DockDragInversionSettingsV1,
    pub tab_drag_threshold_px: f32,
    pub dock_hint_scale_inner: f32,
    pub dock_hint_scale_outer: f32,
}

impl Default for DockingSettingsV1 {
    fn default() -> Self {
        Self {
            drag_inversion: DockDragInversionSettingsV1::default(),
            tab_drag_threshold_px: 6.0,
            dock_hint_scale_inner: 1.0,
            dock_hint_scale_outer: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DockDragInversionSettingsV1 {
    pub modifier: DockDragInversionModifierV1,
    pub policy: DockDragInversionPolicyV1,
}

impl Default for DockDragInversionSettingsV1 {
    fn default() -> Self {
        Self {
            modifier: DockDragInversionModifierV1::Shift,
            policy: DockDragInversionPolicyV1::DockByDefault,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DockDragInversionModifierV1 {
    None,
    #[default]
    Shift,
    Ctrl,
    Alt,
    AltGr,
    Meta,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DockDragInversionPolicyV1 {
    #[default]
    DockByDefault,
    DockOnlyWhenModifier,
}

impl SettingsFileV1 {
    pub fn docking_interaction_settings(&self) -> fret_runtime::DockingInteractionSettings {
        let default_tab_drag_threshold_px = fret_runtime::DockingInteractionSettings::default()
            .tab_drag_threshold
            .0;
        let tab_drag_threshold_px = if self.docking.tab_drag_threshold_px.is_finite() {
            self.docking.tab_drag_threshold_px.max(0.0)
        } else {
            default_tab_drag_threshold_px
        };

        let default_dock_hint_scale_inner =
            fret_runtime::DockingInteractionSettings::default().dock_hint_scale_inner;
        let dock_hint_scale_inner = if self.docking.dock_hint_scale_inner.is_finite() {
            self.docking.dock_hint_scale_inner.max(0.0)
        } else {
            default_dock_hint_scale_inner
        };

        let default_dock_hint_scale_outer =
            fret_runtime::DockingInteractionSettings::default().dock_hint_scale_outer;
        let dock_hint_scale_outer = if self.docking.dock_hint_scale_outer.is_finite() {
            self.docking.dock_hint_scale_outer.max(0.0)
        } else {
            default_dock_hint_scale_outer
        };

        fret_runtime::DockingInteractionSettings {
            drag_inversion: self.docking.drag_inversion.clone().into(),
            tab_drag_threshold: fret_core::Px(tab_drag_threshold_px),
            dock_hint_scale_inner,
            dock_hint_scale_outer,
            ..Default::default()
        }
    }
}

pub fn apply_settings_globals(app: &mut crate::App, settings: &SettingsFileV1) {
    app.set_global(settings.clone());
    app.set_global(settings.docking_interaction_settings());

    let mut i18n = settings.i18n_service();
    if let Some(existing) = app.global::<fret_runtime::fret_i18n::I18nService>() {
        if let Some(lookup) = existing.lookup() {
            i18n.set_lookup(Some(lookup.clone()));
        }
        i18n.set_missing_message_behavior(existing.missing_message_behavior());
    }
    app.set_global(i18n);
    crate::core_commands::apply_core_command_localization(app);
}

fn parse_locale_or_default(input: &str) -> fret_runtime::fret_i18n::LocaleId {
    parse_locale_if_well_formed(input).unwrap_or_default()
}

fn parse_locale_if_well_formed(input: &str) -> Option<fret_runtime::fret_i18n::LocaleId> {
    let locale = fret_runtime::fret_i18n::LocaleId::parse(input).ok()?;
    let langid = locale.as_langid();

    let normalized = input.trim();
    let has_explicit_subtag_separator = normalized.contains('-') || normalized.contains('_');

    let has_region_or_script = langid.region.is_some() || langid.script.is_some();
    let has_min_length = langid.language.as_str().len() >= 2;
    let has_variants = langid.variants().len() > 0;

    if has_variants {
        return None;
    }

    if has_region_or_script || (has_explicit_subtag_separator && has_min_length) {
        Some(locale)
    } else {
        None
    }
}

impl SettingsFileV1 {
    pub fn menu_bar_os_enabled(&self, platform: fret_runtime::Platform) -> bool {
        match self.menu_bar.os {
            MenuBarIntegrationModeV1::Off => false,
            MenuBarIntegrationModeV1::On => platform != fret_runtime::Platform::Web,
            MenuBarIntegrationModeV1::Auto => matches!(
                platform,
                fret_runtime::Platform::Windows | fret_runtime::Platform::Macos
            ),
        }
    }

    pub fn menu_bar_in_window_enabled(&self, platform: fret_runtime::Platform) -> bool {
        match self.menu_bar.in_window {
            MenuBarIntegrationModeV1::Off => false,
            MenuBarIntegrationModeV1::On => true,
            MenuBarIntegrationModeV1::Auto => matches!(
                platform,
                fret_runtime::Platform::Linux | fret_runtime::Platform::Web
            ),
        }
    }
}

impl From<DockDragInversionSettingsV1> for fret_runtime::DockDragInversionSettings {
    fn from(value: DockDragInversionSettingsV1) -> Self {
        Self {
            modifier: value.modifier.into(),
            policy: value.policy.into(),
        }
    }
}

impl From<DockDragInversionModifierV1> for fret_runtime::DockDragInversionModifier {
    fn from(value: DockDragInversionModifierV1) -> Self {
        match value {
            DockDragInversionModifierV1::None => Self::None,
            DockDragInversionModifierV1::Shift => Self::Shift,
            DockDragInversionModifierV1::Ctrl => Self::Ctrl,
            DockDragInversionModifierV1::Alt => Self::Alt,
            DockDragInversionModifierV1::AltGr => Self::AltGr,
            DockDragInversionModifierV1::Meta => Self::Meta,
        }
    }
}

impl From<DockDragInversionPolicyV1> for fret_runtime::DockDragInversionPolicy {
    fn from(value: DockDragInversionPolicyV1) -> Self {
        match value {
            DockDragInversionPolicyV1::DockByDefault => Self::DockByDefault,
            DockDragInversionPolicyV1::DockOnlyWhenModifier => Self::DockOnlyWhenModifier,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn docking_tab_drag_threshold_negative_is_clamped_to_zero() {
        let mut settings = SettingsFileV1::default();
        settings.docking.tab_drag_threshold_px = -1.0;

        assert_eq!(
            settings.docking_interaction_settings().tab_drag_threshold,
            fret_core::Px(0.0)
        );
    }

    #[test]
    fn docking_tab_drag_threshold_nan_falls_back_to_default() {
        let mut settings = SettingsFileV1::default();
        settings.docking.tab_drag_threshold_px = f32::NAN;

        assert_eq!(
            settings.docking_interaction_settings().tab_drag_threshold,
            fret_runtime::DockingInteractionSettings::default().tab_drag_threshold
        );
    }

    #[test]
    fn docking_hint_scale_negative_is_clamped_to_zero() {
        let mut settings = SettingsFileV1::default();
        settings.docking.dock_hint_scale_inner = -1.0;
        settings.docking.dock_hint_scale_outer = -2.0;

        let resolved = settings.docking_interaction_settings();
        assert_eq!(resolved.dock_hint_scale_inner, 0.0);
        assert_eq!(resolved.dock_hint_scale_outer, 0.0);
    }

    #[test]
    fn docking_hint_scale_nan_falls_back_to_default() {
        let mut settings = SettingsFileV1::default();
        settings.docking.dock_hint_scale_inner = f32::NAN;
        settings.docking.dock_hint_scale_outer = f32::NAN;

        let resolved = settings.docking_interaction_settings();
        assert_eq!(
            resolved.dock_hint_scale_inner,
            fret_runtime::DockingInteractionSettings::default().dock_hint_scale_inner
        );
        assert_eq!(
            resolved.dock_hint_scale_outer,
            fret_runtime::DockingInteractionSettings::default().dock_hint_scale_outer
        );
    }

    #[test]
    fn locale_defaults_to_en_us_without_fallbacks() {
        let settings = SettingsFileV1::default();
        assert_eq!(settings.locale.primary, "en-US");
        assert!(settings.locale.fallbacks.is_empty());
        assert!(!settings.locale.pseudo);
    }

    #[test]
    fn resolved_locales_falls_back_to_default_when_primary_is_invalid() {
        let mut settings = SettingsFileV1::default();
        settings.locale.primary = "invalid-locale".to_string();

        let locales = settings.resolved_locales();
        assert_eq!(locales, vec![fret_runtime::fret_i18n::LocaleId::default()]);
    }

    #[test]
    fn resolved_locales_dedupes_and_skips_invalid_fallbacks() {
        let mut settings = SettingsFileV1::default();
        settings.locale.primary = "en-US".to_string();
        settings.locale.fallbacks = vec![
            "zh-CN".to_string(),
            "invalid".to_string(),
            "en-US".to_string(),
        ];

        let locales = settings.resolved_locales();
        assert_eq!(
            locales,
            vec![
                fret_runtime::fret_i18n::LocaleId::parse("en-US").expect("valid locale"),
                fret_runtime::fret_i18n::LocaleId::parse("zh-CN").expect("valid locale"),
            ]
        );
    }

    #[test]
    fn i18n_service_applies_pseudo_from_settings() {
        let mut settings = SettingsFileV1::default();
        settings.locale.pseudo = true;

        let service = settings.i18n_service();
        assert!(service.pseudo_enabled());
        assert_eq!(service.preferred_locales().len(), 1);
    }

    #[test]
    fn apply_settings_globals_sets_i18n_service() {
        let mut app = crate::App::new();

        let mut settings = SettingsFileV1::default();
        settings.locale.primary = "zh-CN".to_string();
        settings.locale.pseudo = true;
        apply_settings_globals(&mut app, &settings);

        let service = app
            .global::<fret_runtime::fret_i18n::I18nService>()
            .expect("i18n service must be installed");
        assert!(service.pseudo_enabled());
        assert_eq!(
            service.preferred_locales(),
            &[fret_runtime::fret_i18n::LocaleId::parse("zh-CN").expect("valid locale")]
        );
    }
}
