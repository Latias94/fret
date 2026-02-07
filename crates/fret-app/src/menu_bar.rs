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

    pub(crate) fn set_baseline(&mut self, menu_bar: MenuBar) -> bool {
        if self.baseline.as_ref() == Some(&menu_bar) {
            return false;
        }
        self.baseline = Some(menu_bar);
        true
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
    layered: Option<LayeredMenuBarConfig>,
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

pub fn set_menu_bar_baseline(app: &mut App, menu_bar: MenuBar) {
    let changed = app.with_global_mut_untracked(MenuBarBaselineService::default, |svc, _app| {
        svc.set_baseline(menu_bar)
    });

    if !changed {
        return;
    }

    let layered = app
        .global::<MenuBarOverlayState>()
        .and_then(|state| state.layered.clone());

    let Some(layered) = layered else {
        return;
    };

    let _ = apply_layered_menu_bar(app, None, layered);
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

#[derive(Debug, Clone, Default, PartialEq)]
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
    let (has_baseline, base) = match app.global::<MenuBarBaselineService>() {
        Some(svc) => match svc.baseline().cloned() {
            Some(baseline) => (true, baseline),
            None => (false, MenuBar::empty()),
        },
        None => (false, MenuBar::empty()),
    };

    let mut effective = base.clone();
    let mut has_layers = false;

    if let Some((path, cfg)) = layered.user.as_ref() {
        has_layers = true;
        effective =
            apply_config(effective, cfg.clone()).map_err(|source| MenuBarFileError::Parse {
                path: path.display().to_string(),
                source,
            })?;
    }

    if let Some((path, cfg)) = layered.project.as_ref() {
        has_layers = true;
        effective =
            apply_config(effective, cfg.clone()).map_err(|source| MenuBarFileError::Parse {
                path: path.display().to_string(),
                source,
            })?;
    }

    let layered_for_state = has_layers.then(|| layered.clone());
    let next_effective = if has_layers {
        Some(effective)
    } else if has_baseline {
        None
    } else {
        Some(base.clone())
    };

    let mut should_push = false;
    app.with_global_mut_untracked(MenuBarOverlayState::default, |state, _app| {
        if state.layered.as_ref() != layered_for_state.as_ref()
            || state.effective.as_ref() != next_effective.as_ref()
        {
            state.layered = layered_for_state.clone();
            state.effective = next_effective.clone();
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
