mod accessors;
mod classification;
mod mutation;

/// Aggregate source-level metadata for an IMUI debug draw list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct DebugDrawListSummary {
    command_count: usize,
    clip_push_count: usize,
    clip_pop_count: usize,
    max_clip_depth: usize,
    final_clip_depth: usize,
    image_command_count: usize,
    svg_command_count: usize,
    text_command_count: usize,
    point_count: usize,
    vertex_count: usize,
    index_count: usize,
    triangle_count: usize,
}
