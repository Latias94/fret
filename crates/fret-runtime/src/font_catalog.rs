use serde::{Deserialize, Serialize};

/// Best-effort font family catalog for settings UIs.
///
/// This is populated by the runner from the renderer's text backend and is platform-dependent by
/// design.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct FontCatalog {
    pub families: Vec<String>,
    /// Monotonic revision that increments when the catalog is refreshed.
    pub revision: u64,
}

/// Stable key representing the current effective text font stack / fallback configuration.
///
/// Runners should update this whenever the renderer text backend changes in a way that can affect
/// shaping/metrics: font family overrides, user font loading, web font injection, etc.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextFontStackKey(pub u64);
