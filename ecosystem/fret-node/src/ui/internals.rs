//! Derived editor internals output (UI-only).
//!
//! This module mirrors the "internals" concept from XyFlow/ReactFlow: the canonical graph model
//! stays pure data, while derived geometry (node rects, handle bounds, transforms) can be surfaced
//! for editor tooling (overlays, inspectors, debugging) without serializing it into assets.

#[path = "internals/snapshot.rs"]
mod snapshot;
#[path = "internals/store.rs"]
mod store;
#[path = "internals/transform.rs"]
mod transform;

pub use snapshot::{NodeGraphA11ySnapshot, NodeGraphInternalsSnapshot};
pub use store::NodeGraphInternalsStore;
pub use transform::NodeGraphCanvasTransform;
