use fret_runtime::CommandId;

use crate::interaction::NodeGraphConnectionMode;
use crate::ui::commands::{
    CMD_NODE_GRAPH_FRAME_ALL, CMD_NODE_GRAPH_FRAME_SELECTION, CMD_NODE_GRAPH_RESET_VIEW,
    CMD_NODE_GRAPH_TOGGLE_CONNECTION_MODE, CMD_NODE_GRAPH_ZOOM_IN, CMD_NODE_GRAPH_ZOOM_OUT,
};

/// Command dispatch override for a controls overlay action.
#[derive(Debug, Clone, PartialEq)]
pub enum NodeGraphControlsCommandBinding {
    /// Uses the overlay's built-in command mapping.
    Default,
    /// Disables the action (no command dispatch).
    Disabled,
    /// Dispatches a custom command when the action is activated.
    Command(CommandId),
}

/// B-layer wiring knobs for the controls overlay.
///
/// This is intentionally policy-light: it only affects what gets dispatched on activation, and does
/// not change layout, hit-testing, or focus behavior.
#[derive(Debug, Clone)]
pub struct NodeGraphControlsBindings {
    pub toggle_connection_mode: NodeGraphControlsCommandBinding,
    pub zoom_in: NodeGraphControlsCommandBinding,
    pub zoom_out: NodeGraphControlsCommandBinding,
    pub frame_all: NodeGraphControlsCommandBinding,
    pub frame_selection: NodeGraphControlsCommandBinding,
    pub reset_view: NodeGraphControlsCommandBinding,
}

impl Default for NodeGraphControlsBindings {
    fn default() -> Self {
        Self {
            toggle_connection_mode: NodeGraphControlsCommandBinding::Default,
            zoom_in: NodeGraphControlsCommandBinding::Default,
            zoom_out: NodeGraphControlsCommandBinding::Default,
            frame_all: NodeGraphControlsCommandBinding::Default,
            frame_selection: NodeGraphControlsCommandBinding::Default,
            reset_view: NodeGraphControlsCommandBinding::Default,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ControlsButton {
    ToggleConnectionMode,
    ZoomIn,
    ZoomOut,
    FrameAll,
    FrameSelection,
    ResetView,
}

pub(super) fn controls_buttons() -> &'static [ControlsButton] {
    &[
        ControlsButton::ToggleConnectionMode,
        ControlsButton::ZoomIn,
        ControlsButton::ZoomOut,
        ControlsButton::FrameAll,
        ControlsButton::FrameSelection,
        ControlsButton::ResetView,
    ]
}

pub(super) fn resolve_controls_command_id(
    bindings: &NodeGraphControlsBindings,
    button: ControlsButton,
) -> Option<CommandId> {
    match resolve_controls_binding(bindings, button) {
        NodeGraphControlsCommandBinding::Disabled => None,
        NodeGraphControlsCommandBinding::Command(id) => Some(id.clone()),
        NodeGraphControlsCommandBinding::Default => Some(default_controls_command_id(button)),
    }
}

pub(super) fn next_controls_button(current: Option<ControlsButton>, dir: i32) -> ControlsButton {
    let buttons = controls_buttons();
    let idx = current
        .and_then(|current| buttons.iter().position(|button| *button == current))
        .unwrap_or(0);
    let len = buttons.len().max(1);
    let idx_i32 = idx as i32;
    let len_i32 = len as i32;
    let mut next = idx_i32 + dir;
    next = ((next % len_i32) + len_i32) % len_i32;
    buttons[next as usize]
}

pub(super) fn controls_button_a11y_label(button: ControlsButton) -> &'static str {
    match button {
        ControlsButton::ToggleConnectionMode => "Toggle connection mode",
        ControlsButton::ZoomIn => "Zoom in",
        ControlsButton::ZoomOut => "Zoom out",
        ControlsButton::FrameAll => "Frame all",
        ControlsButton::FrameSelection => "Frame selection",
        ControlsButton::ResetView => "Reset view",
    }
}

pub(super) fn controls_button_label(
    button: ControlsButton,
    mode: NodeGraphConnectionMode,
) -> &'static str {
    match button {
        ControlsButton::ToggleConnectionMode => match mode {
            NodeGraphConnectionMode::Strict => "S",
            NodeGraphConnectionMode::Loose => "L",
        },
        ControlsButton::ZoomIn => "+",
        ControlsButton::ZoomOut => "–",
        ControlsButton::FrameAll => "Fit",
        ControlsButton::FrameSelection => "Sel",
        ControlsButton::ResetView => "1:1",
    }
}

fn resolve_controls_binding(
    bindings: &NodeGraphControlsBindings,
    button: ControlsButton,
) -> &NodeGraphControlsCommandBinding {
    match button {
        ControlsButton::ToggleConnectionMode => &bindings.toggle_connection_mode,
        ControlsButton::ZoomIn => &bindings.zoom_in,
        ControlsButton::ZoomOut => &bindings.zoom_out,
        ControlsButton::FrameAll => &bindings.frame_all,
        ControlsButton::FrameSelection => &bindings.frame_selection,
        ControlsButton::ResetView => &bindings.reset_view,
    }
}

fn default_controls_command_id(button: ControlsButton) -> CommandId {
    match button {
        ControlsButton::ToggleConnectionMode => {
            CommandId::from(CMD_NODE_GRAPH_TOGGLE_CONNECTION_MODE)
        }
        ControlsButton::ZoomIn => CommandId::from(CMD_NODE_GRAPH_ZOOM_IN),
        ControlsButton::ZoomOut => CommandId::from(CMD_NODE_GRAPH_ZOOM_OUT),
        ControlsButton::FrameAll => CommandId::from(CMD_NODE_GRAPH_FRAME_ALL),
        ControlsButton::FrameSelection => CommandId::from(CMD_NODE_GRAPH_FRAME_SELECTION),
        ControlsButton::ResetView => CommandId::from(CMD_NODE_GRAPH_RESET_VIEW),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ControlsButton, NodeGraphControlsBindings, NodeGraphControlsCommandBinding,
        controls_button_a11y_label, controls_button_label, controls_buttons, next_controls_button,
        resolve_controls_command_id,
    };
    use crate::interaction::NodeGraphConnectionMode;
    use fret_runtime::CommandId;

    #[test]
    fn controls_buttons_order_stays_stable_for_layout_and_keyboard_navigation() {
        assert_eq!(
            controls_buttons(),
            &[
                ControlsButton::ToggleConnectionMode,
                ControlsButton::ZoomIn,
                ControlsButton::ZoomOut,
                ControlsButton::FrameAll,
                ControlsButton::FrameSelection,
                ControlsButton::ResetView,
            ]
        );
    }

    #[test]
    fn next_controls_button_wraps_in_both_directions() {
        assert_eq!(
            next_controls_button(Some(ControlsButton::ResetView), 1),
            ControlsButton::ToggleConnectionMode
        );
        assert_eq!(
            next_controls_button(Some(ControlsButton::ToggleConnectionMode), -1),
            ControlsButton::ResetView
        );
    }

    #[test]
    fn controls_policy_resolves_default_and_override_commands() {
        let mut bindings = NodeGraphControlsBindings::default();
        bindings.zoom_in =
            NodeGraphControlsCommandBinding::Command(CommandId::from("node_graph.custom.zoom_in"));
        bindings.reset_view = NodeGraphControlsCommandBinding::Disabled;

        assert_eq!(
            resolve_controls_command_id(&bindings, ControlsButton::ZoomIn),
            Some(CommandId::from("node_graph.custom.zoom_in"))
        );
        assert_eq!(
            resolve_controls_command_id(&bindings, ControlsButton::ResetView),
            None
        );
        assert_eq!(
            controls_button_a11y_label(ControlsButton::FrameSelection),
            "Frame selection"
        );
        assert_eq!(
            controls_button_label(
                ControlsButton::ToggleConnectionMode,
                NodeGraphConnectionMode::Strict
            ),
            "S"
        );
        assert_eq!(
            controls_button_label(
                ControlsButton::ToggleConnectionMode,
                NodeGraphConnectionMode::Loose
            ),
            "L"
        );
    }
}
