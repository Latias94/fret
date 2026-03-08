use super::*;

pub(super) fn align_or_distribute_mode(command: &str) -> Option<AlignDistributeMode> {
    match command {
        CMD_NODE_GRAPH_ALIGN_LEFT => Some(AlignDistributeMode::AlignLeft),
        CMD_NODE_GRAPH_ALIGN_RIGHT => Some(AlignDistributeMode::AlignRight),
        CMD_NODE_GRAPH_ALIGN_TOP => Some(AlignDistributeMode::AlignTop),
        CMD_NODE_GRAPH_ALIGN_BOTTOM => Some(AlignDistributeMode::AlignBottom),
        CMD_NODE_GRAPH_ALIGN_CENTER_X => Some(AlignDistributeMode::AlignCenterX),
        CMD_NODE_GRAPH_ALIGN_CENTER_Y => Some(AlignDistributeMode::AlignCenterY),
        CMD_NODE_GRAPH_DISTRIBUTE_X => Some(AlignDistributeMode::DistributeX),
        CMD_NODE_GRAPH_DISTRIBUTE_Y => Some(AlignDistributeMode::DistributeY),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn align_or_distribute_mode_maps_all_supported_commands() {
        assert_eq!(
            align_or_distribute_mode(CMD_NODE_GRAPH_ALIGN_LEFT),
            Some(AlignDistributeMode::AlignLeft)
        );
        assert_eq!(
            align_or_distribute_mode(CMD_NODE_GRAPH_ALIGN_CENTER_Y),
            Some(AlignDistributeMode::AlignCenterY)
        );
        assert_eq!(
            align_or_distribute_mode(CMD_NODE_GRAPH_DISTRIBUTE_X),
            Some(AlignDistributeMode::DistributeX)
        );
        assert_eq!(
            align_or_distribute_mode(CMD_NODE_GRAPH_DISTRIBUTE_Y),
            Some(AlignDistributeMode::DistributeY)
        );
    }

    #[test]
    fn align_or_distribute_mode_rejects_other_commands() {
        assert_eq!(
            align_or_distribute_mode(CMD_NODE_GRAPH_DELETE_SELECTION),
            None
        );
    }
}
