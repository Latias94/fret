use serde::{Deserialize, Serialize};

/// Portable cursor icon set for UI → host requests.
///
/// This is intentionally a small, cross-platform subset (desktop-first). Platforms may map these
/// to their closest native cursors, or treat them as no-ops if unsupported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CursorIcon {
    Default,
    Pointer,
    Text,
    ColResize,
    RowResize,
}

impl Default for CursorIcon {
    fn default() -> Self {
        Self::Default
    }
}
