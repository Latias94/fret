use fret_app::App;
use fret_runtime::{KeymapService, keymap::Binding};

/// Installs all command-provided default keybindings into the app keymap.
///
/// Demos use this to keep shortcut behavior aligned with the command registry metadata, without
/// hardcoding per-demo keymap tables.
pub(crate) fn install_default_keybindings_into_keymap(app: &mut App) {
    let mut bindings: Vec<Binding> = Vec::new();

    for (id, meta) in app.commands().iter() {
        for kb in meta.default_keybindings.iter().cloned() {
            bindings.push(Binding {
                platform: kb.platform,
                sequence: vec![kb.chord],
                when: kb.when.clone().or_else(|| meta.when.clone()),
                command: Some(id.clone()),
            });
        }
    }

    if bindings.is_empty() {
        return;
    }

    app.with_global_mut(KeymapService::default, |svc, _app| {
        for b in bindings {
            svc.keymap.push_binding(b);
        }
    });
}
