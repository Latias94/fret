use std::any::Any;
use std::collections::HashSet;
use std::sync::Arc;

use fret_runtime::CommandMeta;
use fret_runtime::{CommandId, KeymapService, keymap::Binding};

use crate::App;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PluginId(pub Arc<str>);

impl PluginId {
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&'static str> for PluginId {
    fn from(value: &'static str) -> Self {
        Self(Arc::<str>::from(value))
    }
}

/// Minimal plugin boundary contract (ADR 0016).
///
/// Plugins register into an app-owned registry and may contribute:
/// - command IDs + metadata (plus optional default keybindings),
/// - keymap bindings (rare; prefer command metadata defaults),
/// - app globals used by their UI/policies.
pub trait Plugin: Send + Sync + 'static {
    fn id(&self) -> PluginId;
    fn register(&self, registrar: &mut PluginRegistrar<'_>);
}

pub struct PluginRegistrar<'a> {
    app: &'a mut App,
}

impl<'a> PluginRegistrar<'a> {
    pub fn new(app: &'a mut App) -> Self {
        Self { app }
    }

    pub fn register_command(&mut self, id: impl Into<CommandId>, meta: CommandMeta) {
        self.app.commands_mut().register(id.into(), meta);
    }

    pub fn register_keymap_binding(&mut self, binding: Binding) {
        self.app
            .with_global_mut(KeymapService::default, |svc, _app| {
                svc.keymap.push_binding(binding);
            });
    }

    pub fn set_global<T: Any>(&mut self, value: T) {
        self.app.set_global(value);
    }

    pub fn app_mut(&mut self) -> &mut App {
        self.app
    }
}

#[derive(Debug, Default)]
pub struct PluginHost {
    installed: HashSet<PluginId>,
}

impl PluginHost {
    pub fn is_installed(&self, id: &PluginId) -> bool {
        self.installed.contains(id)
    }

    pub fn install(&mut self, app: &mut App, plugin: &dyn Plugin) -> bool {
        let id = plugin.id();
        if !self.installed.insert(id) {
            return false;
        }

        let mut registrar = PluginRegistrar::new(app);
        plugin.register(&mut registrar);
        true
    }
}

pub fn install_plugins<'a>(app: &mut App, plugins: impl IntoIterator<Item = &'a dyn Plugin>) {
    let plugins: Vec<&dyn Plugin> = plugins.into_iter().collect();
    if plugins.is_empty() {
        return;
    }

    app.with_global_mut(PluginHost::default, |host, app| {
        for plugin in plugins {
            host.install(app, plugin);
        }
    });

    crate::keymap::install_command_default_keybindings_into_keymap(app);
}
