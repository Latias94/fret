use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use fret_core::AppWindowId;
use fret_runtime::{Effect, TimerToken};

use crate::config_files::LayeredConfigPaths;
use crate::{App, KeymapFileError, SettingsError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FileStamp {
    len: u64,
    modified: Option<SystemTime>,
}

fn file_stamp(path: &Path) -> Option<FileStamp> {
    let meta = std::fs::metadata(path).ok()?;
    Some(FileStamp {
        len: meta.len(),
        modified: meta.modified().ok(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct LayeredStamps {
    user: Option<FileStamp>,
    project: Option<FileStamp>,
}

impl LayeredStamps {
    fn is_unchanged_from(&self, next: &LayeredStamps) -> bool {
        self == next
    }
}

#[derive(Debug, Clone)]
pub struct ConfigFilesWatcherTick {
    pub reloaded_settings: bool,
    pub reloaded_keymap: bool,
    pub settings_error: Option<String>,
    pub keymap_error: Option<String>,
    pub actionable_keymap_conflicts: usize,
    pub keymap_conflict_samples: Vec<String>,
}

#[derive(Debug, Default)]
pub struct ConfigFilesWatcher {
    token: Option<TimerToken>,
    poll_interval: Duration,
    project_root: PathBuf,
    stamps_settings: LayeredStamps,
    stamps_keymap: LayeredStamps,
}

impl ConfigFilesWatcher {
    pub fn install(app: &mut App, poll_interval: Duration, project_root: impl AsRef<Path>) {
        if cfg!(target_arch = "wasm32") {
            return;
        }

        let project_root = project_root.as_ref().to_path_buf();

        let mut existing = app
            .global::<ConfigFilesWatcher>()
            .map(|w| (w.token, w.poll_interval, w.project_root.clone()));

        if let Some((token, prev_interval, prev_root)) = existing.take()
            && token.is_some()
            && prev_interval == poll_interval
            && prev_root == project_root
        {
            return;
        }

        if let Some((Some(token), _, _)) = existing {
            app.push_effect(Effect::CancelTimer { token });
        }

        let paths = LayeredConfigPaths::for_project_root(&project_root);
        let next_settings = LayeredStamps {
            user: paths.user_settings_json().and_then(|p| file_stamp(&p)),
            project: file_stamp(&paths.project_settings_json()),
        };
        let next_keymap = LayeredStamps {
            user: paths.user_keymap_json().and_then(|p| file_stamp(&p)),
            project: file_stamp(&paths.project_keymap_json()),
        };

        let token = app.next_timer_token();
        app.push_effect(Effect::SetTimer {
            window: None,
            token,
            after: poll_interval,
            repeat: Some(poll_interval),
        });

        app.set_global(ConfigFilesWatcher {
            token: Some(token),
            poll_interval,
            project_root,
            stamps_settings: next_settings,
            stamps_keymap: next_keymap,
        });
    }

    pub fn token(&self) -> Option<TimerToken> {
        self.token
    }

    fn poll_and_reload(&mut self, app: &mut App, window: AppWindowId) -> ConfigFilesWatcherTick {
        let paths = LayeredConfigPaths::for_project_root(&self.project_root);

        let next_settings = LayeredStamps {
            user: paths.user_settings_json().and_then(|p| file_stamp(&p)),
            project: file_stamp(&paths.project_settings_json()),
        };
        let next_keymap = LayeredStamps {
            user: paths.user_keymap_json().and_then(|p| file_stamp(&p)),
            project: file_stamp(&paths.project_keymap_json()),
        };

        let settings_changed = !self.stamps_settings.is_unchanged_from(&next_settings);
        let keymap_changed = !self.stamps_keymap.is_unchanged_from(&next_keymap);

        self.stamps_settings = next_settings;
        self.stamps_keymap = next_keymap;

        let mut tick = ConfigFilesWatcherTick {
            reloaded_settings: false,
            reloaded_keymap: false,
            settings_error: None,
            keymap_error: None,
            actionable_keymap_conflicts: 0,
            keymap_conflict_samples: Vec::new(),
        };

        if settings_changed {
            match crate::config_files::load_layered_settings(&paths) {
                Ok((settings, _report)) => {
                    app.set_global(settings.clone());
                    app.set_global(settings.docking_interaction_settings());
                    app.request_redraw(window);
                    tick.reloaded_settings = true;
                }
                Err(e) => {
                    tick.settings_error = Some(format_settings_error(&e));
                }
            }
        }

        if keymap_changed {
            match crate::config_files::load_layered_keymap(&paths) {
                Ok((layered, report)) => {
                    crate::keymap::apply_layered_keymap(app, layered);
                    app.request_redraw(window);
                    tick.reloaded_keymap = true;

                    let actionable = report
                        .conflicts
                        .iter()
                        .filter(|c| c.kind != fret_runtime::keymap::KeymapConflictKind::Redundant)
                        .count();
                    tick.actionable_keymap_conflicts = actionable;
                    tick.keymap_conflict_samples = report
                        .conflicts
                        .iter()
                        .filter(|c| c.kind != fret_runtime::keymap::KeymapConflictKind::Redundant)
                        .take(5)
                        .map(format_keymap_conflict)
                        .collect();
                }
                Err(e) => {
                    tick.keymap_error = Some(format_keymap_error(&e));
                }
            }
        }

        tick
    }
}

pub fn handle_config_files_watcher_timer(
    app: &mut App,
    window: AppWindowId,
    token: TimerToken,
) -> Option<ConfigFilesWatcherTick> {
    let watcher_token = app.global::<ConfigFilesWatcher>().and_then(|w| w.token());
    if watcher_token != Some(token) {
        return None;
    }

    let mut out: Option<ConfigFilesWatcherTick> = None;
    app.with_global_mut(ConfigFilesWatcher::default, |w, app| {
        if w.token != Some(token) {
            return;
        }
        out = Some(w.poll_and_reload(app, window));
    });
    out
}

fn format_settings_error(e: &SettingsError) -> String {
    match e {
        SettingsError::Read { path, source } => format!("read failed: {path}: {source}"),
        SettingsError::Parse { path, source } => format!("parse failed: {path}: {source}"),
    }
}

fn format_keymap_error(e: &KeymapFileError) -> String {
    match e {
        KeymapFileError::Read { path, source } => format!("read failed: {path}: {source}"),
        KeymapFileError::Parse { path, source } => format!("parse failed: {path}: {source}"),
    }
}

fn format_keymap_conflict(c: &fret_runtime::keymap::KeymapConflict) -> String {
    let platform = match c.signature.platform {
        fret_runtime::PlatformFilter::Macos => fret_runtime::Platform::Macos,
        fret_runtime::PlatformFilter::Windows => fret_runtime::Platform::Windows,
        fret_runtime::PlatformFilter::Linux => fret_runtime::Platform::Linux,
        fret_runtime::PlatformFilter::Web => fret_runtime::Platform::Web,
        fret_runtime::PlatformFilter::All => fret_runtime::Platform::current(),
    };
    let seq = fret_runtime::format_sequence(platform, &c.signature.sequence);
    let when = c
        .signature
        .when
        .as_ref()
        .map(|w| format!("{w:?}"))
        .unwrap_or_else(|| "<none>".to_string());
    let entries = c
        .entries
        .iter()
        .map(|e| {
            let cmd = e.command.as_ref().map(|c| c.as_str()).unwrap_or("<unbind>");
            format!("#{}={}", e.index, cmd)
        })
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        "kind={:?} platform={:?} when={} keys={} [{}]",
        c.kind, c.signature.platform, when, seq, entries
    )
}

