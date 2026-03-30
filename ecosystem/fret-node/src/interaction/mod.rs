//! Editor interaction configuration types.
//!
//! These types are shared across:
//! - persisted view state (`crate::io`),
//! - the UI substrate (`crate::ui`),
//! - headless rules/policies (`crate::rules`, `crate::profile`).

use fret_core::Modifiers;
use serde::{Deserialize, Serialize};

/// Connection mode for selecting/validating target ports during connection gestures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum NodeGraphConnectionMode {
    #[default]
    Strict,
    Loose,
}


/// Where node dragging can start from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum NodeGraphDragHandleMode {
    /// Start dragging from anywhere inside the node bounds.
    #[default]
    Any,
    /// Start dragging only from the node header region.
    Header,
}


/// Modifier requirement for interaction activation (XyFlow mental model).
///
/// This is used for zoom activation (`zoomActivationKey`), selection activation (`selectionKeyCode`),
/// and multi-selection (`multiSelectionKeyCode`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum NodeGraphModifierKey {
    /// Wheel zoom is always active (no activation modifier required).
    None,
    /// Wheel zoom is active only while Ctrl or Meta is held (recommended default).
    #[default]
    CtrlOrMeta,
    /// Wheel zoom is active only while Shift is held.
    Shift,
    /// Wheel zoom is active only while Alt is held.
    Alt,
}

/// Backward-compat alias for older API surface.
pub type NodeGraphZoomActivationKey = NodeGraphModifierKey;

impl NodeGraphModifierKey {
    pub fn is_pressed(self, modifiers: Modifiers) -> bool {
        match self {
            Self::None => true,
            Self::CtrlOrMeta => modifiers.ctrl || modifiers.meta,
            Self::Shift => modifiers.shift,
            Self::Alt => modifiers.alt || modifiers.alt_gr,
        }
    }
}

