use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::KeymapFileError;
use crate::{SettingsError, SettingsFileV1};
use fret_runtime::Keymap;

pub const PROJECT_CONFIG_DIR: &str = ".fret";
pub const SETTINGS_JSON: &str = "settings.json";
pub const KEYMAP_JSON: &str = "keymap.json";
pub const MENUBAR_JSON: &str = "menubar.json";

#[derive(Debug, Clone, Default)]
pub struct LayeredConfigPaths {
    pub user_dir: Option<PathBuf>,
    pub project_dir: PathBuf,
}

impl LayeredConfigPaths {
    pub fn for_project_root(project_root: impl AsRef<Path>) -> Self {
        Self {
            user_dir: default_user_config_dir(),
            project_dir: project_root.as_ref().join(PROJECT_CONFIG_DIR),
        }
    }

    pub fn user_settings_json(&self) -> Option<PathBuf> {
        self.user_dir.as_ref().map(|d| d.join(SETTINGS_JSON))
    }

    pub fn project_settings_json(&self) -> PathBuf {
        self.project_dir.join(SETTINGS_JSON)
    }

    pub fn user_keymap_json(&self) -> Option<PathBuf> {
        self.user_dir.as_ref().map(|d| d.join(KEYMAP_JSON))
    }

    pub fn project_keymap_json(&self) -> PathBuf {
        self.project_dir.join(KEYMAP_JSON)
    }

    pub fn user_menubar_json(&self) -> Option<PathBuf> {
        self.user_dir.as_ref().map(|d| d.join(MENUBAR_JSON))
    }

    pub fn project_menubar_json(&self) -> PathBuf {
        self.project_dir.join(MENUBAR_JSON)
    }
}

#[derive(Debug, Clone, Default)]
pub struct LayeredSettingsReport {
    pub user: Option<PathBuf>,
    pub project: Option<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct LayeredKeymapReport {
    pub user: Option<PathBuf>,
    pub project: Option<PathBuf>,
    pub conflicts: Vec<fret_runtime::keymap::KeymapConflict>,
    pub user_conflicts: Vec<fret_runtime::keymap::KeymapConflict>,
    pub project_conflicts: Vec<fret_runtime::keymap::KeymapConflict>,
}

#[derive(Debug, Clone, Default)]
pub struct LayeredMenuBarReport {
    pub user: Option<PathBuf>,
    pub project: Option<PathBuf>,
}

pub fn load_layered_settings(
    paths: &LayeredConfigPaths,
) -> Result<(SettingsFileV1, LayeredSettingsReport), SettingsError> {
    let mut merged = serde_json::to_value(SettingsFileV1::default())
        .expect("SettingsFileV1::default must be serializable");

    let mut report = LayeredSettingsReport::default();

    if let Some(path) = paths.user_settings_json()
        && let Some(value) = SettingsFileV1::load_json_value_if_exists(&path)?
    {
        merge_json(&mut merged, value);
        report.user = Some(path);
    }

    let project_path = paths.project_settings_json();
    if let Some(value) = SettingsFileV1::load_json_value_if_exists(&project_path)? {
        merge_json(&mut merged, value);
        report.project = Some(project_path);
    }

    let out: SettingsFileV1 =
        serde_json::from_value(merged).map_err(|source| SettingsError::Parse {
            path: "<layered settings.json>".to_string(),
            source,
        })?;

    Ok((out, report))
}

pub fn load_layered_keymap(
    paths: &LayeredConfigPaths,
) -> Result<(Keymap, LayeredKeymapReport), KeymapFileError> {
    let mut keymap = Keymap::default();
    let mut report = LayeredKeymapReport::default();

    if let Some(path) = paths.user_keymap_json()
        && let Some(layer) = crate::keymap::keymap_from_file_if_exists(&path)?
    {
        report.user_conflicts = layer.conflicts();
        keymap.extend(layer);
        report.user = Some(path);
    }

    let project_path = paths.project_keymap_json();
    if let Some(layer) = crate::keymap::keymap_from_file_if_exists(&project_path)? {
        report.project_conflicts = layer.conflicts();
        keymap.extend(layer);
        report.project = Some(project_path);
    }

    report.conflicts = keymap.conflicts();
    Ok((keymap, report))
}

pub fn load_layered_menu_bar(
    paths: &LayeredConfigPaths,
) -> Result<
    (crate::menu_bar::LayeredMenuBarConfig, LayeredMenuBarReport),
    crate::menu_bar::MenuBarFileError,
> {
    let mut out = crate::menu_bar::LayeredMenuBarConfig::default();
    let mut report = LayeredMenuBarReport::default();

    if let Some(path) = paths.user_menubar_json()
        && let Some(menu_bar) = crate::menu_bar::menu_bar_from_file_if_exists(&path)?
    {
        out.user = Some((path.clone(), menu_bar));
        report.user = Some(path);
    }

    let project_path = paths.project_menubar_json();
    if let Some(menu_bar) = crate::menu_bar::menu_bar_from_file_if_exists(&project_path)? {
        out.project = Some((project_path.clone(), menu_bar));
        report.project = Some(project_path);
    }

    Ok((out, report))
}

fn merge_json(base: &mut Value, overlay: Value) {
    match (base, overlay) {
        (Value::Object(base), Value::Object(overlay)) => {
            for (k, v) in overlay {
                match base.get_mut(&k) {
                    Some(slot) => merge_json(slot, v),
                    None => {
                        base.insert(k, v);
                    }
                }
            }
        }
        (base, overlay) => {
            *base = overlay;
        }
    }
}

pub fn default_user_config_dir() -> Option<PathBuf> {
    if cfg!(target_arch = "wasm32") {
        return None;
    }

    if cfg!(target_os = "windows") {
        let appdata = std::env::var_os("APPDATA")?;
        return Some(PathBuf::from(appdata).join("fret"));
    }

    if cfg!(target_os = "macos") {
        let home = std::env::var_os("HOME")?;
        return Some(
            PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("fret"),
        );
    }

    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(xdg).join("fret"));
    }
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join(".config").join("fret"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layered_settings_merge_does_not_override_with_defaults() {
        let mut merged = serde_json::to_value(SettingsFileV1::default()).unwrap();

        let overlay: Value = serde_json::json!({
            "settings_version": 1,
            "docking": { "drag_inversion": { "modifier": "ctrl" } }
        });
        merge_json(&mut merged, overlay);

        let out: SettingsFileV1 = serde_json::from_value(merged).unwrap();
        assert_eq!(out.settings_version, 1);
        assert_eq!(
            out.docking.drag_inversion.modifier,
            crate::DockDragInversionModifierV1::Ctrl
        );
        assert_eq!(out.fonts, crate::FontsSettingsV1::default());
    }

    #[test]
    fn layered_settings_merge_applies_locale_fields() {
        let mut merged = serde_json::to_value(SettingsFileV1::default()).unwrap();

        let overlay: Value = serde_json::json!({
            "locale": {
                "primary": "zh-CN",
                "fallbacks": ["en-US"],
                "pseudo": true
            }
        });
        merge_json(&mut merged, overlay);

        let out: SettingsFileV1 = serde_json::from_value(merged).unwrap();
        assert_eq!(out.locale.primary, "zh-CN");
        assert_eq!(out.locale.fallbacks, vec!["en-US"]);
        assert!(out.locale.pseudo);
    }

    #[test]
    fn layered_settings_partial_locale_overlay_keeps_defaults() {
        let mut merged = serde_json::to_value(SettingsFileV1::default()).unwrap();

        let overlay: Value = serde_json::json!({
            "locale": {
                "pseudo": true
            }
        });
        merge_json(&mut merged, overlay);

        let out: SettingsFileV1 = serde_json::from_value(merged).unwrap();
        assert_eq!(out.locale.primary, "en-US");
        assert!(out.locale.fallbacks.is_empty());
        assert!(out.locale.pseudo);
    }
}
