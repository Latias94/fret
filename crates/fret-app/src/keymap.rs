use std::collections::HashSet;
use std::path::Path;

use fret_runtime::CommandId;
pub use fret_runtime::keymap::*;

use crate::App;

#[derive(Debug, thiserror::Error)]
pub enum KeymapFileError {
    #[error("failed to read keymap file: {path}")]
    Read {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to parse keymap file: {path}")]
    Parse { path: String, source: KeymapError },
}

/// Loads a keymap file from disk and parses it into a `Keymap`.
///
/// Note: `fret-runtime` is intentionally IO-free; file IO lives at the app/runner boundary.
pub fn keymap_from_file(path: &Path) -> Result<Keymap, KeymapError> {
    let bytes = std::fs::read(path).map_err(|source| KeymapError::ReadFailed { source })?;
    Keymap::from_bytes(&bytes)
}

pub fn keymap_from_file_if_exists(
    path: impl AsRef<Path>,
) -> Result<Option<Keymap>, KeymapFileError> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(None);
    }
    let bytes = std::fs::read(path).map_err(|source| KeymapFileError::Read {
        path: path.display().to_string(),
        source,
    })?;
    Keymap::from_bytes(&bytes)
        .map(Some)
        .map_err(|source| KeymapFileError::Parse {
            path: path.display().to_string(),
            source,
        })
}

#[derive(Debug, Default)]
struct InstalledCommandDefaultKeybindings {
    installed: HashSet<CommandId>,
}

/// Installs all command-provided default keybindings into the app keymap.
///
/// This keeps shortcut behavior aligned with the command registry metadata, without hardcoding
/// per-app keymap tables.
pub fn install_command_default_keybindings_into_keymap(app: &mut App) {
    let installed = app
        .global::<InstalledCommandDefaultKeybindings>()
        .map(|svc| svc.installed.clone())
        .unwrap_or_default();

    let mut bindings: Vec<Binding> = Vec::new();
    let mut newly_installed: Vec<CommandId> = Vec::new();

    for (id, meta) in app.commands().iter() {
        if installed.contains(id) {
            continue;
        }

        for kb in meta.default_keybindings.iter().cloned() {
            bindings.push(Binding {
                platform: kb.platform,
                sequence: kb.sequence,
                when: kb.when.clone().or_else(|| meta.when.clone()),
                command: Some(id.clone()),
            });
        }

        if !meta.default_keybindings.is_empty() {
            newly_installed.push(id.clone());
        }
    }

    if bindings.is_empty() {
        return;
    }

    app.with_global_mut(crate::KeymapService::default, |svc, _app| {
        for b in bindings {
            svc.keymap.push_binding(b);
        }
    });

    app.with_global_mut(InstalledCommandDefaultKeybindings::default, |svc, _app| {
        for id in newly_installed {
            svc.installed.insert(id);
        }
    });
}

#[derive(Debug, Default)]
struct LayeredKeymapState {
    baseline: Option<Keymap>,
    layered: Keymap,
}

/// Applies a "disk layered" keymap on top of the app's baseline keymap.
///
/// This preserves the baseline keymap (defaults + plugin keybindings) and makes it possible to
/// re-apply user/project overrides without accumulating duplicate bindings.
pub fn apply_layered_keymap(app: &mut App, layered: Keymap) {
    let current = app
        .global::<crate::KeymapService>()
        .map(|svc| svc.keymap.clone())
        .unwrap_or_default();

    let (baseline, layered) = app.with_global_mut(LayeredKeymapState::default, |state, _app| {
        if state.baseline.is_none() {
            state.baseline = Some(current);
        }
        state.layered = layered;
        (
            state.baseline.clone().unwrap_or_default(),
            state.layered.clone(),
        )
    });

    app.with_global_mut(crate::KeymapService::default, |svc, _app| {
        svc.keymap = baseline;
        svc.keymap.extend(layered);
    });
}
