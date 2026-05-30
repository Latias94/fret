//! Node graph aliases for shared 2D resize handle vocabulary.
//!
//! The generic handle enum and bitset live in `fret-canvas`; this module keeps node-graph naming at
//! the public presenter boundary without re-owning the mechanism.

pub use fret_canvas::interaction::{
    ResizeHandle2D as NodeResizeHandle, ResizeHandleSet2D as NodeResizeHandleSet,
};
