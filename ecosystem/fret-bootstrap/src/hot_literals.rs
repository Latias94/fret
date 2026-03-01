//! Hot-reloadable string literals for UI development workflows.
//!
//! This is an ecosystem-level convenience that intentionally keeps the core UI runtime policy-free.
//! The literals live in a host global and can be updated at runtime by the app driver (for example
//! a polling watcher that reloads `.fret/literals.json`).

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use fret_runtime::GlobalsHost;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default)]
pub struct HotLiterals {
    map: Arc<HashMap<Arc<str>, Arc<str>>>,
}

impl HotLiterals {
    pub fn from_json_slice(bytes: &[u8]) -> Result<Self, String> {
        let raw: HashMap<String, String> =
            serde_json::from_slice(bytes).map_err(|e| format!("invalid literals JSON: {e}"))?;

        let mut map: HashMap<Arc<str>, Arc<str>> = HashMap::with_capacity(raw.len());
        for (k, v) in raw {
            if k.trim().is_empty() {
                continue;
            }
            map.insert(Arc::<str>::from(k), Arc::<str>::from(v));
        }

        Ok(Self { map: Arc::new(map) })
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.map.get(key).map(|v| v.as_ref())
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn global<H: GlobalsHost>(host: &H) -> &HotLiterals {
        match host.global::<HotLiterals>() {
            Some(v) => v,
            None => default_hot_literals(),
        }
    }
}

fn default_hot_literals() -> &'static HotLiterals {
    static DEFAULT: OnceLock<HotLiterals> = OnceLock::new();
    DEFAULT.get_or_init(HotLiterals::default)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HotLiteralsFile(pub HashMap<String, String>);

pub mod ui;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_get() {
        let bytes = br#"{ "demo.headline": "Hello", "empty": "" }"#;
        let lits = HotLiterals::from_json_slice(bytes).expect("valid json");
        assert_eq!(lits.get("demo.headline"), Some("Hello"));
        assert_eq!(lits.get("missing"), None);
    }
}
