use std::collections::HashMap;
use std::path::PathBuf;

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
