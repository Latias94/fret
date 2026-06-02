use std::sync::Arc;

use fret_core::ImageId;

use crate::imui::debug_draw_controls::{DebugDrawImageMeshOptions, DebugDrawVertex};

// This file owns triangle mesh and image triangle mesh debug-draw command payload variants.

#[derive(Debug, Clone)]
pub(in crate::imui::debug_draw_controls) enum DebugDrawMeshCommand {
    TriangleMesh {
        vertices: Arc<[DebugDrawVertex]>,
        indices: Arc<[u32]>,
    },
    ImageTriangleMesh {
        image: ImageId,
        vertices: Arc<[DebugDrawVertex]>,
        indices: Arc<[u32]>,
        options: DebugDrawImageMeshOptions,
    },
}
