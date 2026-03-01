use std::collections::HashMap;
use std::path::PathBuf;

use fret_app::App;
use fret_core::AppWindowId;
use serde_json::Value;

#[derive(Debug, Clone)]
pub enum DevStateExport {
    /// Leave the previous outgoing value untouched.
    Noop,
    /// Remove the outgoing key (clears persisted state on next flush).
    Remove,
    /// Set the outgoing value.
    Set(Value),
}

#[derive(Debug, Clone)]
pub struct DevStateSnapshot {
    pub epoch: u64,
    pub data: HashMap<String, Value>,
}

#[derive(Debug, Default)]
pub struct DevStateWindowKeyRegistry {
    epoch: u64,
    keys: HashMap<AppWindowId, String>,
}

impl DevStateWindowKeyRegistry {
    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    pub fn snapshot(&self) -> Vec<(AppWindowId, String)> {
        self.keys
            .iter()
            .map(|(window, key)| (*window, key.clone()))
            .collect()
    }

    pub fn register(&mut self, window: AppWindowId, key: impl Into<String>) {
        let key = key.into();
        if self.keys.get(&window).is_some_and(|prev| prev == &key) {
            return;
        }
        self.keys.insert(window, key);
        self.epoch = self.epoch.saturating_add(1);
    }

    pub fn unregister(&mut self, window: AppWindowId) {
        if self.keys.remove(&window).is_some() {
            self.epoch = self.epoch.saturating_add(1);
        }
    }
}

#[derive(Debug, Default)]
pub struct DevStateService {
    incoming: HashMap<String, Value>,
    outgoing: HashMap<String, Value>,
    outgoing_epoch: u64,
    path: Option<PathBuf>,
}

type DevStateExportFn = dyn Fn(&App) -> DevStateExport + Send + Sync;
type DevStateImportFn = dyn Fn(&mut App, Value) -> Result<(), String> + Send + Sync;

pub struct DevStateHook {
    key: String,
    export: Box<DevStateExportFn>,
    import: Option<Box<DevStateImportFn>>,
}

impl std::fmt::Debug for DevStateHook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DevStateHook")
            .field("key", &self.key)
            .finish_non_exhaustive()
    }
}

impl DevStateHook {
    pub fn new(
        key: impl Into<String>,
        export: impl Fn(&App) -> DevStateExport + Send + Sync + 'static,
    ) -> Self {
        Self {
            key: key.into(),
            export: Box::new(export),
            import: None,
        }
    }

    pub fn with_import(
        mut self,
        import: impl Fn(&mut App, Value) -> Result<(), String> + Send + Sync + 'static,
    ) -> Self {
        self.import = Some(Box::new(import));
        self
    }
}

#[derive(Debug, Default)]
pub struct DevStateHooks {
    hooks: Vec<DevStateHook>,
}

impl DevStateHooks {
    pub fn register(&mut self, hook: DevStateHook) {
        self.hooks.push(hook);
    }

    pub fn import_all(app: &mut App) {
        if app.global::<DevStateService>().is_none() {
            return;
        }
        if app.global::<DevStateHooks>().is_none() {
            return;
        }

        app.with_global_mut_untracked(DevStateHooks::default, |hooks, app| {
            if hooks.hooks.is_empty() {
                return;
            }

            for hook in &hooks.hooks {
                let Some(import) = hook.import.as_ref() else {
                    continue;
                };

                let key = hook.key.clone();
                let incoming = app
                    .with_global_mut_untracked(DevStateService::default, |svc, _app| {
                        svc.take_incoming(&key)
                    });
                let Some(value) = incoming else {
                    continue;
                };

                if let Err(err) = (import)(app, value) {
                    tracing::warn!(key = %key, error = %err, "dev_state: import hook failed");
                }
            }
        });
    }

    pub fn export_all(app: &mut App) {
        if app.global::<DevStateService>().is_none() {
            return;
        }
        if app.global::<DevStateHooks>().is_none() {
            return;
        }

        app.with_global_mut_untracked(DevStateHooks::default, |hooks, app| {
            if hooks.hooks.is_empty() {
                return;
            }

            for hook in &hooks.hooks {
                match (hook.export)(app) {
                    DevStateExport::Noop => {}
                    DevStateExport::Remove => {
                        app.with_global_mut_untracked(DevStateService::default, |svc, _app| {
                            svc.remove_outgoing(&hook.key);
                        });
                    }
                    DevStateExport::Set(value) => {
                        app.with_global_mut_untracked(DevStateService::default, |svc, _app| {
                            svc.set_outgoing_if_changed(&hook.key, value);
                        });
                    }
                }
            }
        });
    }
}

impl DevStateService {
    pub fn new(incoming: HashMap<String, Value>, path: PathBuf) -> Self {
        Self {
            incoming: incoming.clone(),
            outgoing: incoming,
            outgoing_epoch: 0,
            path: Some(path),
        }
    }

    pub fn path(&self) -> Option<&PathBuf> {
        self.path.as_ref()
    }

    pub fn take_incoming(&mut self, key: &str) -> Option<Value> {
        self.incoming.remove(key)
    }

    pub fn set_outgoing(&mut self, key: impl Into<String>, value: Value) {
        self.outgoing.insert(key.into(), value);
        self.outgoing_epoch = self.outgoing_epoch.saturating_add(1);
    }

    pub fn set_outgoing_if_changed(&mut self, key: &str, value: Value) -> bool {
        if self.outgoing.get(key).is_some_and(|prev| prev == &value) {
            return false;
        }
        self.outgoing.insert(key.to_string(), value);
        self.outgoing_epoch = self.outgoing_epoch.saturating_add(1);
        true
    }

    pub fn remove_outgoing(&mut self, key: &str) -> Option<Value> {
        let prev = self.outgoing.remove(key);
        if prev.is_some() {
            self.outgoing_epoch = self.outgoing_epoch.saturating_add(1);
        }
        prev
    }

    pub fn outgoing_snapshot(&self) -> DevStateSnapshot {
        DevStateSnapshot {
            epoch: self.outgoing_epoch,
            data: self.outgoing.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_outgoing_if_changed_bumps_epoch_only_on_change() {
        let mut svc = DevStateService::default();
        assert_eq!(svc.outgoing_snapshot().epoch, 0);
        assert!(svc.set_outgoing_if_changed("k", Value::from(1)));
        let e1 = svc.outgoing_snapshot().epoch;
        assert!(e1 > 0);
        assert!(!svc.set_outgoing_if_changed("k", Value::from(1)));
        assert_eq!(svc.outgoing_snapshot().epoch, e1);
        assert!(svc.set_outgoing_if_changed("k", Value::from(2)));
        assert!(svc.outgoing_snapshot().epoch > e1);
    }
}
