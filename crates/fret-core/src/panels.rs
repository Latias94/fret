use serde::{Deserialize, Serialize};

/// Stable panel identity used for docking persistence and plugin registration.
///
/// This must remain stable across runs and should be namespaced (e.g. `core.scene`, `plugin.foo.panel`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PanelKind(pub String);

impl PanelKind {
    pub fn new(kind: impl Into<String>) -> Self {
        Self(kind.into())
    }
}

/// Stable identity for a specific dockable panel instance.
///
/// Most panels will be singletons and keep `instance = None`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PanelKey {
    pub kind: PanelKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
}

impl PanelKey {
    pub fn new(kind: impl Into<String>) -> Self {
        Self {
            kind: PanelKind::new(kind),
            instance: None,
        }
    }

    pub fn with_instance(kind: impl Into<String>, instance: impl Into<String>) -> Self {
        Self {
            kind: PanelKind::new(kind),
            instance: Some(instance.into()),
        }
    }
}
