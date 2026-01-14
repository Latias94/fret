use fret_core::Rect;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DndItemId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Draggable {
    pub id: DndItemId,
    pub rect: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Droppable {
    pub id: DndItemId,
    pub rect: Rect,
    pub disabled: bool,
    pub z_index: i32,
}

#[derive(Debug, Default, Clone)]
pub struct RegistrySnapshot {
    pub draggables: Vec<Draggable>,
    pub droppables: Vec<Droppable>,
}
