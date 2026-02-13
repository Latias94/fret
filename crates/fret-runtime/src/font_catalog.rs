use serde::{Deserialize, Serialize};

/// Best-effort metadata for a variable font axis.
///
/// Floats are stored as raw `f32` bit patterns to keep the struct `Eq` and stable under
/// serialization while remaining lossless.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct FontVariableAxisInfo {
    pub tag: String,
    pub min_bits: u32,
    pub max_bits: u32,
    pub default_bits: u32,
}

impl FontVariableAxisInfo {
    pub fn min(&self) -> f32 {
        f32::from_bits(self.min_bits)
    }

    pub fn max(&self) -> f32 {
        f32::from_bits(self.max_bits)
    }

    pub fn default(&self) -> f32 {
        f32::from_bits(self.default_bits)
    }
}

/// Best-effort font family catalog for settings UIs.
///
/// This is populated by the runner from the renderer's text backend and is platform-dependent by
/// design.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct FontCatalog {
    pub families: Vec<String>,
    /// Monotonic revision that increments when the effective catalog contents change.
    ///
    /// Refresh attempts that yield the same catalog should not bump this revision, to avoid
    /// spurious invalidation and UI churn.
    pub revision: u64,
}

/// Best-effort metadata for a font family entry.
///
/// This is populated by the runner from the renderer's text backend and is platform-dependent by
/// design. Fields are intentionally coarse and should be treated as hints for settings pickers and
/// diagnostics, not as hard contracts.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct FontCatalogEntry {
    pub family: String,
    /// Whether the family appears to contain at least one variable font (any axis present).
    pub has_variable_axes: bool,
    /// Known variable axis tags (best-effort), e.g. `wght`, `wdth`, `slnt`, `ital`, `opsz`.
    pub known_variable_axes: Vec<String>,
    /// Best-effort variable axis metadata for the family's default face.
    ///
    /// Axis tags beyond the known set may be present (e.g. `GRAD` for Roboto Flex).
    #[serde(default)]
    pub variable_axes: Vec<FontVariableAxisInfo>,
    /// Best-effort monospace hint derived from font tables (typically PostScript `isFixedPitch`).
    pub is_monospace_candidate: bool,
}

/// Best-effort catalog metadata (entries + revision).
///
/// The revision is expected to be monotonic and should generally match `FontCatalog.revision` when
/// both are set by the runner.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct FontCatalogMetadata {
    pub entries: Vec<FontCatalogEntry>,
    /// Monotonic revision that increments when the effective entry list changes.
    pub revision: u64,
}

/// Stable key representing the current effective text font stack / fallback configuration.
///
/// Runners should update this whenever the renderer text backend changes in a way that can affect
/// shaping/metrics: font family overrides, user font loading, web font injection, etc.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextFontStackKey(pub u64);
