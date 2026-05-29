use super::DebugDrawListSummary;

impl DebugDrawListSummary {
    pub fn command_count(self) -> usize {
        self.command_count
    }

    pub fn clip_push_count(self) -> usize {
        self.clip_push_count
    }

    pub fn clip_pop_count(self) -> usize {
        self.clip_pop_count
    }

    pub fn max_clip_depth(self) -> usize {
        self.max_clip_depth
    }

    pub fn final_clip_depth(self) -> usize {
        self.final_clip_depth
    }

    pub fn image_command_count(self) -> usize {
        self.image_command_count
    }

    pub fn svg_command_count(self) -> usize {
        self.svg_command_count
    }

    pub fn text_command_count(self) -> usize {
        self.text_command_count
    }

    pub fn point_count(self) -> usize {
        self.point_count
    }

    pub fn vertex_count(self) -> usize {
        self.vertex_count
    }

    pub fn index_count(self) -> usize {
        self.index_count
    }

    pub fn triangle_count(self) -> usize {
        self.triangle_count
    }
}
