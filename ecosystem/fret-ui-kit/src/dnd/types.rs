use super::{AutoScrollRequest, DndCollision, DndItemId, SensorOutput};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DndScopeId(pub u64);

pub const DND_SCOPE_DEFAULT: DndScopeId = DndScopeId(0);

#[derive(Debug, Clone)]
pub struct DndUpdate {
    pub sensor: SensorOutput,
    pub collisions: Vec<DndCollision>,
    pub over: Option<DndItemId>,
    pub autoscroll: Option<AutoScrollRequest>,
}

impl Default for DndUpdate {
    fn default() -> Self {
        Self {
            sensor: SensorOutput::Pending,
            collisions: Vec::new(),
            over: None,
            autoscroll: None,
        }
    }
}

impl DndUpdate {
    pub fn pending() -> Self {
        Self::default()
    }
}
