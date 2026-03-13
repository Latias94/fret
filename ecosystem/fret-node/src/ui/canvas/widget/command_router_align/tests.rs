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
