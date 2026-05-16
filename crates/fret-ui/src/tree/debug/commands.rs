use super::super::*;

#[derive(Debug, Clone)]
pub struct UiDebugCommandAvailabilityHotspot {
    pub command: CommandId,
    pub route: &'static str,
    pub start_node: NodeId,
    pub resolved_node: Option<NodeId>,
    pub outcome: CommandAvailability,
    pub elapsed: Duration,
    pub start_element: Option<GlobalElementId>,
    pub resolved_element: Option<GlobalElementId>,
}
