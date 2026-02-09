use serde::{Deserialize, Serialize};

/// Quality of cursor/position updates during external OS drag sessions (e.g. file drag hover).
///
/// This is used to express *degradation*, not just availability:
/// a backend may support external drops but not provide reliable per-frame hover coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExternalDragPositionQuality {
    /// External drag is unsupported (or position updates are unavailable).
    None,
    /// The backend provides external drag events, but pointer positions are best-effort / may be
    /// stale or missing (e.g. macOS winit file DnD hover limitations).
    BestEffort,
    /// The backend provides reliable pointer position updates during external drag hover.
    #[default]
    Continuous,
}

impl ExternalDragPositionQuality {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::BestEffort => "best_effort",
            Self::Continuous => "continuous",
        }
    }

    pub fn clamp_to_available(self, available: Self) -> Self {
        use ExternalDragPositionQuality::*;
        match (self, available) {
            (None, _) => None,
            (_, None) => None,
            (BestEffort, BestEffort | Continuous) => BestEffort,
            (Continuous, Continuous) => Continuous,
            (Continuous, BestEffort) => BestEffort,
        }
    }
}

/// Windowing quality signal: whether the backend can reliably determine which window is under the
/// cursor.
///
/// This is used as a degradation signal for editor-grade multi-window UX (e.g. docking tear-off
/// hover target selection under overlap).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WindowHoverDetectionQuality {
    /// The backend cannot reliably determine window-under-cursor (or cannot provide global cursor
    /// position updates needed to infer it).
    None,
    /// Best-effort: selection may be stale/missing or ambiguous under overlap.
    BestEffort,
    /// Reliable enough for editor-grade hover selection.
    #[default]
    Reliable,
}

impl WindowHoverDetectionQuality {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::BestEffort => "best_effort",
            Self::Reliable => "reliable",
        }
    }

    pub fn clamp_to_available(self, available: Self) -> Self {
        use WindowHoverDetectionQuality::*;
        match (self, available) {
            (None, _) => None,
            (_, None) => None,
            (BestEffort, BestEffort | Reliable) => BestEffort,
            (Reliable, Reliable) => Reliable,
            (Reliable, BestEffort) => BestEffort,
        }
    }
}

/// Windowing quality signal: whether programmatic window movement via outer-position requests is
/// reliable enough for "follow cursor" UX.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WindowSetOuterPositionQuality {
    None,
    BestEffort,
    #[default]
    Reliable,
}

impl WindowSetOuterPositionQuality {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::BestEffort => "best_effort",
            Self::Reliable => "reliable",
        }
    }

    pub fn clamp_to_available(self, available: Self) -> Self {
        use WindowSetOuterPositionQuality::*;
        match (self, available) {
            (None, _) => None,
            (_, None) => None,
            (BestEffort, BestEffort | Reliable) => BestEffort,
            (Reliable, Reliable) => Reliable,
            (Reliable, BestEffort) => BestEffort,
        }
    }
}

/// Windowing quality signal: whether OS z-level requests (e.g. AlwaysOnTop during drags) behave
/// predictably.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WindowZLevelQuality {
    None,
    BestEffort,
    #[default]
    Reliable,
}

impl WindowZLevelQuality {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::BestEffort => "best_effort",
            Self::Reliable => "reliable",
        }
    }

    pub fn clamp_to_available(self, available: Self) -> Self {
        use WindowZLevelQuality::*;
        match (self, available) {
            (None, _) => None,
            (_, None) => None,
            (BestEffort, BestEffort | Reliable) => BestEffort,
            (Reliable, Reliable) => Reliable,
            (Reliable, BestEffort) => BestEffort,
        }
    }
}
