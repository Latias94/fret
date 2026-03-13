use super::*;

#[derive(Default)]
pub(super) struct FrameScratchState {
    viewport_uniform_bytes_scratch: Vec<u8>,
    render_space_bytes_scratch: Vec<u8>,
    plan_quad_vertices_scratch: Vec<ViewportVertex>,
    plan_quad_vertex_bases_scratch: Vec<Option<u32>>,
}

impl FrameScratchState {
    pub(super) fn viewport_uniform_bytes_mut(&mut self) -> &mut Vec<u8> {
        &mut self.viewport_uniform_bytes_scratch
    }

    pub(super) fn render_space_bytes_mut(&mut self, render_space_bytes_len: usize) -> &mut Vec<u8> {
        self.render_space_bytes_scratch.clear();
        self.render_space_bytes_scratch
            .resize(render_space_bytes_len, 0u8);
        &mut self.render_space_bytes_scratch
    }

    pub(super) fn take_plan_quad_scratch(&mut self) -> (Vec<ViewportVertex>, Vec<Option<u32>>) {
        (
            std::mem::take(&mut self.plan_quad_vertices_scratch),
            std::mem::take(&mut self.plan_quad_vertex_bases_scratch),
        )
    }

    pub(super) fn finish_plan_quad_vertices(&mut self, mut vertices: Vec<ViewportVertex>) {
        vertices.clear();
        self.plan_quad_vertices_scratch = vertices;
    }

    pub(super) fn store_plan_quad_bases(&mut self, bases: Vec<Option<u32>>) {
        self.plan_quad_vertex_bases_scratch = bases;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quad_scratch_roundtrips_vertices_and_bases() {
        let mut state = FrameScratchState::default();
        state.store_plan_quad_bases(vec![Some(7)]);

        let (mut vertices, bases) = state.take_plan_quad_scratch();
        assert_eq!(bases, vec![Some(7)]);
        vertices.push(ViewportVertex {
            pos_px: [1.0, 2.0],
            uv: [0.0, 1.0],
            opacity: 0.5,
            _pad: [0.0; 3],
        });

        state.finish_plan_quad_vertices(vertices);

        let (vertices, bases) = state.take_plan_quad_scratch();
        assert!(vertices.is_empty());
        assert!(vertices.capacity() >= 1);
        assert!(bases.is_empty());
    }
}
