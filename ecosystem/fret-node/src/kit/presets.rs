//! Interaction presets for editor-like node graphs.
//!
//! These are convenience configurations layered on top of the stable substrate. They are intended
//! as starting points (one-liners) for building a graph editor with familiar behavior.

use crate::io::{NodeGraphConnectionMode, NodeGraphInteractionState, NodeGraphPanOnDragButtons};

/// High-level editor interaction preset (mechanism remains the same).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeGraphInteractionPreset {
    /// ReactFlow/XyFlow-style defaults.
    XyFlow,
    /// Unity ShaderGraph-style affordances (2D editor).
    ShaderGraph,
}

impl NodeGraphInteractionPreset {
    /// Returns a fresh interaction state configured for this preset.
    ///
    /// This is intentionally a full `NodeGraphInteractionState` snapshot (not a delta) so callers
    /// can treat it as a "default baseline" and then override domain-specific knobs.
    pub fn interaction_state(self) -> NodeGraphInteractionState {
        let mut s = NodeGraphInteractionState::default();

        match self {
            Self::XyFlow => {
                // Keep substrate defaults aligned with XyFlow expectations.
            }
            Self::ShaderGraph => {
                // Panning: middle-mouse drag is the primary affordance.
                s.pan_on_drag = NodeGraphPanOnDragButtons {
                    left: false,
                    middle: true,
                    right: false,
                };
                s.space_to_pan = false;
                s.pan_activation_key_code = None;

                // Selection: drag on background selects without modifiers (common in node editors).
                s.selection_on_drag = true;

                // Connection semantics: strict wiring, but still typed via the rules layer.
                s.connection_mode = NodeGraphConnectionMode::Strict;
                s.connect_on_click = false;

                // Edge ergonomics: double-click on a wire inserts a reroute (optional, opt-in).
                s.reroute_on_edge_double_click = true;

                // Avoid conflicts with "double click to insert" workflows.
                s.zoom_on_double_click = false;
            }
        }

        s
    }

    /// Applies this preset to an existing interaction state.
    ///
    /// This keeps some domain-owned surfaces intact (e.g. extents and snap grid size), but sets
    /// the core interaction affordances consistently.
    pub fn apply_to(self, state: &mut NodeGraphInteractionState) {
        let keep_translate = state.translate_extent;
        let keep_node_extent = state.node_extent;
        let keep_snap_grid = state.snap_grid;

        *state = self.interaction_state();
        state.translate_extent = keep_translate;
        state.node_extent = keep_node_extent;
        state.snap_grid = keep_snap_grid;
    }
}

#[cfg(test)]
mod tests {
    use super::NodeGraphInteractionPreset;

    #[test]
    fn shadergraph_preset_enables_reroute_double_click_and_middle_mouse_pan() {
        let s = NodeGraphInteractionPreset::ShaderGraph.interaction_state();
        assert!(s.reroute_on_edge_double_click);
        assert!(s.pan_on_drag.middle);
        assert!(!s.pan_on_drag.left);
        assert!(!s.space_to_pan);
        assert!(s.selection_on_drag);
        assert!(matches!(
            s.connection_mode,
            crate::interaction::NodeGraphConnectionMode::Strict
        ));
        assert!(!s.zoom_on_double_click);
    }

    #[test]
    fn apply_to_preserves_extents_and_snap_grid() {
        let mut s = crate::io::NodeGraphInteractionState::default();
        s.snap_grid = crate::core::CanvasSize {
            width: 42.0,
            height: 24.0,
        };
        s.translate_extent = Some(crate::core::CanvasRect {
            origin: crate::core::CanvasPoint { x: -10.0, y: -20.0 },
            size: crate::core::CanvasSize {
                width: 100.0,
                height: 200.0,
            },
        });

        NodeGraphInteractionPreset::ShaderGraph.apply_to(&mut s);
        assert_eq!(s.snap_grid.width, 42.0);
        assert!(s.translate_extent.is_some());
    }
}
