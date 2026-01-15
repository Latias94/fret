//! Headless drag-and-drop policy toolbox.
//!
//! This crate is intentionally UI-agnostic: it depends only on `fret-core` geometry and IDs, and
//! provides reusable policy primitives (activation constraints, collision detection, modifiers,
//! and auto-scroll request computation).

pub mod activation;
pub mod collision;
pub mod modifier;
pub mod rect_index;
pub mod registry;
pub mod scroll;

pub use activation::{ActivationConstraint, PointerSensor, SensorEvent, SensorOutput};
pub use collision::{
    CollisionStrategy, DndCollision, closest_center_collisions, pointer_within_collisions,
};
pub use modifier::{Axis, axis_lock, clamp_rect_translation};
pub use rect_index::RectDroppableIndex;
pub use registry::{DndItemId, Draggable, Droppable, RegistrySnapshot};
pub use scroll::{AutoScrollConfig, AutoScrollRequest, compute_autoscroll};
