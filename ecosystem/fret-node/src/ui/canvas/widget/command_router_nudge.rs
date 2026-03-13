use super::*;

#[derive(Clone, Copy)]
pub(super) struct NudgeCommandRequest {
    pub(super) dir: CanvasPoint,
    pub(super) fast: bool,
}

pub(super) fn nudge_command_request(command: &str) -> Option<NudgeCommandRequest> {
    match command {
        CMD_NODE_GRAPH_NUDGE_LEFT => Some(NudgeCommandRequest {
            dir: CanvasPoint { x: -1.0, y: 0.0 },
            fast: false,
        }),
        CMD_NODE_GRAPH_NUDGE_RIGHT => Some(NudgeCommandRequest {
            dir: CanvasPoint { x: 1.0, y: 0.0 },
            fast: false,
        }),
        CMD_NODE_GRAPH_NUDGE_UP => Some(NudgeCommandRequest {
            dir: CanvasPoint { x: 0.0, y: -1.0 },
            fast: false,
        }),
        CMD_NODE_GRAPH_NUDGE_DOWN => Some(NudgeCommandRequest {
            dir: CanvasPoint { x: 0.0, y: 1.0 },
            fast: false,
        }),
        CMD_NODE_GRAPH_NUDGE_LEFT_FAST => Some(NudgeCommandRequest {
            dir: CanvasPoint { x: -1.0, y: 0.0 },
            fast: true,
        }),
        CMD_NODE_GRAPH_NUDGE_RIGHT_FAST => Some(NudgeCommandRequest {
            dir: CanvasPoint { x: 1.0, y: 0.0 },
            fast: true,
        }),
        CMD_NODE_GRAPH_NUDGE_UP_FAST => Some(NudgeCommandRequest {
            dir: CanvasPoint { x: 0.0, y: -1.0 },
            fast: true,
        }),
        CMD_NODE_GRAPH_NUDGE_DOWN_FAST => Some(NudgeCommandRequest {
            dir: CanvasPoint { x: 0.0, y: 1.0 },
            fast: true,
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
