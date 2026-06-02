use super::super::super::super::summaries::{DebugDrawCommandKind, DebugDrawCommandSummary};
use super::super::super::{DebugDrawCommand, DebugDrawMeshCommand};

pub(super) fn mesh_geometry_summary(command: &DebugDrawCommand) -> Option<DebugDrawCommandSummary> {
    let summary = match command {
        DebugDrawCommand::RectFilledMultiColor { .. } => {
            let mut summary =
                DebugDrawCommandSummary::new(DebugDrawCommandKind::RectFilledMultiColor);
            summary.point_count = 4;
            summary.vertex_count = 4;
            summary.index_count = 6;
            summary.triangle_count = 2;
            summary
        }
        DebugDrawCommand::Mesh(DebugDrawMeshCommand::TriangleMesh { vertices, indices }) => {
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::TriangleMesh);
            summary.vertex_count = vertices.len();
            summary.index_count = indices.len();
            summary.triangle_count = indices.len() / 3;
            summary
        }
        _ => return None,
    };
    Some(summary)
}
