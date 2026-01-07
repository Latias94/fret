use crate::ids::Revision;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EngineStats {
    pub revision: Revision,
    pub stage_data_runs: u64,
    pub stage_layout_runs: u64,
    pub stage_visual_runs: u64,
    pub stage_marks_runs: u64,
    pub marks_emitted: u64,
    pub points_emitted: u64,
}
