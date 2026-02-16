use super::super::*;

#[derive(Debug, Clone, Copy)]
pub(in crate::tree) struct DebugLayoutStackFrame {
    pub(in crate::tree) child_inclusive_time: Duration,
}

#[derive(Debug, Clone, Copy)]
pub(in crate::tree) struct DebugWidgetMeasureStackFrame {
    pub(in crate::tree) child_inclusive_time: Duration,
}

#[derive(Debug, Clone, Copy)]
pub(in crate::tree) struct DebugPaintStackFrame {
    pub(in crate::tree) child_inclusive_time: Duration,
    pub(in crate::tree) child_inclusive_scene_ops_delta: u32,
}
