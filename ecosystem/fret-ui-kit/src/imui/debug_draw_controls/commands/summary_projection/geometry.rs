use super::super::super::summaries::{DebugDrawCommandKind, DebugDrawCommandSummary};
use super::super::DebugDrawCommand;

pub(super) fn geometry_summary(command: &DebugDrawCommand) -> Option<DebugDrawCommandSummary> {
    let summary = match command {
        DebugDrawCommand::Line { .. } => {
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::Line);
            summary.point_count = 2;
            summary
        }
        DebugDrawCommand::Polyline { points, .. } => {
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::Polyline);
            summary.point_count = points.len();
            summary
        }
        DebugDrawCommand::ConvexPolyFilled { points, .. } => {
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::ConvexPolyFilled);
            summary.point_count = points.len();
            summary
        }
        DebugDrawCommand::ConcavePolyFilled { points, .. } => {
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::ConcavePolyFilled);
            summary.point_count = points.len();
            summary
        }
        DebugDrawCommand::Rect { .. } => {
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::Rect);
            summary.point_count = 4;
            summary
        }
        DebugDrawCommand::RectFilled { .. } => {
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::RectFilled);
            summary.point_count = 4;
            summary
        }
        DebugDrawCommand::RectFilledMultiColor { .. } => {
            let mut summary =
                DebugDrawCommandSummary::new(DebugDrawCommandKind::RectFilledMultiColor);
            summary.point_count = 4;
            summary.vertex_count = 4;
            summary.index_count = 6;
            summary.triangle_count = 2;
            summary
        }
        DebugDrawCommand::Quad { .. } => {
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::Quad);
            summary.point_count = 4;
            summary
        }
        DebugDrawCommand::QuadFilled { .. } => {
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::QuadFilled);
            summary.point_count = 4;
            summary
        }
        DebugDrawCommand::Triangle { .. } => {
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::Triangle);
            summary.point_count = 3;
            summary
        }
        DebugDrawCommand::TriangleFilled { .. } => {
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::TriangleFilled);
            summary.point_count = 3;
            summary.triangle_count = 1;
            summary
        }
        DebugDrawCommand::TriangleMesh { vertices, indices } => {
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::TriangleMesh);
            summary.vertex_count = vertices.len();
            summary.index_count = indices.len();
            summary.triangle_count = indices.len() / 3;
            summary
        }
        DebugDrawCommand::Circle { .. } => {
            DebugDrawCommandSummary::new(DebugDrawCommandKind::Circle)
        }
        DebugDrawCommand::CircleFilled { .. } => {
            DebugDrawCommandSummary::new(DebugDrawCommandKind::CircleFilled)
        }
        DebugDrawCommand::Ngon { segments, .. } => {
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::Ngon);
            summary.point_count = *segments;
            summary
        }
        DebugDrawCommand::NgonFilled { segments, .. } => {
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::NgonFilled);
            summary.point_count = *segments;
            summary
        }
        DebugDrawCommand::Ellipse { segments, .. } => {
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::Ellipse);
            summary.point_count = *segments;
            summary
        }
        DebugDrawCommand::EllipseFilled { segments, .. } => {
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::EllipseFilled);
            summary.point_count = *segments;
            summary
        }
        DebugDrawCommand::BezierQuadratic { .. } => {
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::BezierQuadratic);
            summary.point_count = 3;
            summary
        }
        DebugDrawCommand::BezierCubic { .. } => {
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::BezierCubic);
            summary.point_count = 4;
            summary
        }
        _ => return None,
    };
    Some(summary)
}
