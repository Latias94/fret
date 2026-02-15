use std::collections::HashMap;
use std::path::PathBuf;

use fret_app::App;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct DevStateSnapshot {
    pub epoch: u64,
    pub data: HashMap<String, Value>,
}

#[derive(Debug)]
pub struct DevStateService {
    incoming: HashMap<String, Value>,
    outgoing: HashMap<String, Value>,
    outgoing_epoch: u64,
    path: Option<PathBuf>,
}

pub struct DevStateHook {
    key: String,
    export: Box<dyn Fn(&App) -> Option<Value> + Send + Sync>,
    import: Option<Box<dyn Fn(&mut App, Value) -> Result<(), String> + Send + Sync>>,
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
        export: impl Fn(&App) -> Option<Value> + Send + Sync + 'static,
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
                let value = (hook.export)(app);
                match value {
                    Some(value) => {
                        app.with_global_mut_untracked(DevStateService::default, |svc, _app| {
                            svc.set_outgoing_if_changed(&hook.key, value);
                        });
                    }
                    None => {
                        app.with_global_mut_untracked(DevStateService::default, |svc, _app| {
                            svc.remove_outgoing(&hook.key);
                        });
                    }
                }
            }
        });
    }
}

impl Default for DevStateService {
    fn default() -> Self {
        Self {
            incoming: HashMap::new(),
            outgoing: HashMap::new(),
            outgoing_epoch: 0,
            path: None,
        }
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
