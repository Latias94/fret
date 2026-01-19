use std::path::{Path, PathBuf};

use fret_core::AppWindowId;
use fret_runtime::{Effect, MenuBar, MenuBarConfig, MenuBarError, Platform};

use crate::App;

#[derive(Debug, thiserror::Error)]
pub enum MenuBarFileError {
    #[error("failed to read menubar file: {path}")]
    Read {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to parse menubar file: {path}")]
    Parse { path: String, source: MenuBarError },
}

#[derive(Debug, Default)]
pub struct MenuBarBaselineService {
    baseline: Option<MenuBar>,
}

impl MenuBarBaselineService {
    pub fn baseline(&self) -> Option<&MenuBar> {
        self.baseline.as_ref()
    }

    pub(crate) fn note_default_menu_bar(&mut self, menu_bar: &MenuBar) {
        let should_overwrite_empty_baseline = self
            .baseline
            .as_ref()
            .is_some_and(|b| b.menus.is_empty() && !menu_bar.menus.is_empty());

        if self.baseline.is_none() || should_overwrite_empty_baseline {
            self.baseline = Some(menu_bar.clone());
        }
    }
}

#[derive(Debug, Default)]
pub struct MenuBarOverlayState {
    effective: Option<MenuBar>,
}

pub fn effective_menu_bar(app: &App) -> Option<MenuBar> {
    let baseline = app
        .global::<MenuBarBaselineService>()
        .and_then(|svc| svc.baseline().cloned());

    let overlaid = app
        .global::<MenuBarOverlayState>()
        .and_then(|s| s.effective.clone());

    overlaid.or(baseline)
}

pub fn should_publish_os_menu_bar(app: &App, platform: Platform) -> bool {
    app.global::<crate::SettingsFileV1>()
        .map(|s| s.menu_bar_os_enabled(platform))
        .unwrap_or_else(|| crate::SettingsFileV1::default().menu_bar_os_enabled(platform))
}

pub fn should_render_in_window_menu_bar(app: &App, platform: Platform) -> bool {
    app.global::<crate::SettingsFileV1>()
        .map(|s| s.menu_bar_in_window_enabled(platform))
        .unwrap_or_else(|| crate::SettingsFileV1::default().menu_bar_in_window_enabled(platform))
}

pub fn sync_os_menu_bar(app: &mut App) {
    let platform = Platform::current();
    if should_publish_os_menu_bar(app, platform) {
        if let Some(menu_bar) = effective_menu_bar(app) {
            app.push_effect(Effect::SetMenuBar {
                window: None,
                menu_bar,
            });
        }
    } else if platform != Platform::Web {
        app.push_effect(Effect::SetMenuBar {
            window: None,
            menu_bar: MenuBar::empty(),
        });
    }
}

#[derive(Debug, Clone, Default)]
pub struct LayeredMenuBarConfig {
    pub user: Option<(PathBuf, MenuBarConfig)>,
    pub project: Option<(PathBuf, MenuBarConfig)>,
}

/// Note: `fret-runtime` is intentionally IO-free; file IO lives at the app/runner boundary.
pub fn menu_bar_from_file_if_exists(
    path: impl AsRef<Path>,
) -> Result<Option<MenuBarConfig>, MenuBarFileError> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(None);
    }

    let bytes = std::fs::read(path).map_err(|source| MenuBarFileError::Read {
        path: path.display().to_string(),
        source,
    })?;
    MenuBarConfig::from_bytes(&bytes)
        .map(Some)
        .map_err(|source| MenuBarFileError::Parse {
            path: path.display().to_string(),
            source,
        })
}

pub fn apply_layered_menu_bar(
    app: &mut App,
    window: Option<AppWindowId>,
    layered: LayeredMenuBarConfig,
) -> Result<(), MenuBarFileError> {
    let base = app
        .global::<MenuBarBaselineService>()
        .and_then(|svc| svc.baseline().cloned())
        .unwrap_or_else(MenuBar::empty);

    let mut effective = base.clone();
    let mut has_layers = false;

    if let Some((path, cfg)) = layered.user {
        has_layers = true;
        effective = apply_config(effective, cfg).map_err(|source| MenuBarFileError::Parse {
            path: path.display().to_string(),
            source,
        })?;
    }

    if let Some((path, cfg)) = layered.project {
        has_layers = true;
        effective = apply_config(effective, cfg).map_err(|source| MenuBarFileError::Parse {
            path: path.display().to_string(),
            source,
        })?;
    }

    let next_effective = if has_layers { effective } else { base };

    let mut should_push = false;
    app.with_global_mut_untracked(MenuBarOverlayState::default, |state, _app| {
        if !has_layers && state.effective.is_none() {
            return;
        }
        if state.effective.as_ref() != Some(&next_effective) {
            state.effective = Some(next_effective.clone());
            should_push = true;
        }
    });

    if should_push {
        sync_os_menu_bar(app);
        if let Some(window) = window {
            app.request_redraw(window);
        }
    }

    Ok(())
}

fn apply_config(mut base: MenuBar, cfg: MenuBarConfig) -> Result<MenuBar, MenuBarError> {
    match cfg {
        MenuBarConfig::Replace(menu_bar) => Ok(menu_bar),
        MenuBarConfig::Patch(patch) => {
            patch.apply_to(&mut base)?;
            Ok(base)
        }
    }
}
