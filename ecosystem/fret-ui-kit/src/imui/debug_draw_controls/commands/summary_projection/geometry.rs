use super::super::super::summaries::DebugDrawCommandSummary;
use super::super::DebugDrawCommand;

mod beziers;
mod linear;
mod mesh;
mod round;

pub(super) fn geometry_summary(command: &DebugDrawCommand) -> Option<DebugDrawCommandSummary> {
    if let Some(summary) = linear::linear_geometry_summary(command) {
        return Some(summary);
    }
    if let Some(summary) = mesh::mesh_geometry_summary(command) {
        return Some(summary);
    }
    if let Some(summary) = round::round_geometry_summary(command) {
        return Some(summary);
    }
    beziers::bezier_geometry_summary(command)
}
