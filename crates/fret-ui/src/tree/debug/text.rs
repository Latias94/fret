use super::super::*;

#[derive(Debug, Clone, Copy, Default)]
pub struct UiDebugTextConstraintsSnapshot {
    pub measured: Option<TextConstraints>,
    pub prepared: Option<TextConstraints>,
}

#[derive(Debug, Clone, Copy)]
pub struct UiDebugPaintTextPrepareHotspot {
    pub node: NodeId,
    pub element: Option<GlobalElementId>,
    pub element_kind: &'static str,
    pub text_len: u32,
    pub constraints: TextConstraints,
    pub reasons_mask: u16,
    pub prepare_time: Duration,
}
