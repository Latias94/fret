//! Headless drag-and-drop policy toolbox.
//!
//! This crate is intentionally UI-agnostic: it depends only on `fret-core` geometry and IDs, and
//! provides reusable policy primitives (activation constraints, collision detection, modifiers,
//! and auto-scroll request computation).

pub mod activation;
pub mod collision;
pub mod frame;
pub mod modifier;
pub mod rect_index;
pub mod registry;
pub mod scroll;
pub mod sortable;

pub use activation::{ActivationConstraint, PointerSensor, SensorEvent, SensorOutput};
pub use collision::{
    CollisionStrategy, DndCollision, closest_center_collisions, closest_center_over,
    pointer_within_collisions, pointer_within_over,
};
pub use frame::compute_dnd_over;
pub use frame::{DndFrameOutput, compute_dnd_frame};
pub use modifier::{Axis, axis_lock, clamp_rect_translation};
pub use rect_index::RectDroppableIndex;
pub use registry::{DndItemId, Draggable, Droppable, RegistrySnapshot};
pub use scroll::{AutoScrollConfig, AutoScrollRequest, compute_autoscroll};
pub use sortable::{InsertionSide, insertion_side_for_pointer, sortable_insertion};
