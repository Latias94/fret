use std::sync::Arc;

use super::super::commands::{DebugDrawCommand, DebugDrawMeshCommand};
use super::super::geometry::sequential_triangle_indices;
use super::super::{DebugDrawVertex, ImUiDebugDrawList};

impl ImUiDebugDrawList {
    pub fn add_triangle_list<V>(&mut self, vertices: V)
    where
        V: IntoIterator<Item = DebugDrawVertex>,
    {
        let vertices: Vec<_> = vertices.into_iter().collect();
        let indices = sequential_triangle_indices(vertices.len());
        self.commands
            .push(DebugDrawCommand::Mesh(DebugDrawMeshCommand::TriangleMesh {
                vertices: Arc::from(vertices),
                indices,
            }));
    }

    pub fn add_triangle_mesh<V, I>(&mut self, vertices: V, indices: I)
    where
        V: IntoIterator<Item = DebugDrawVertex>,
        I: IntoIterator<Item = u32>,
    {
        self.commands
            .push(DebugDrawCommand::Mesh(DebugDrawMeshCommand::TriangleMesh {
                vertices: Arc::from(vertices.into_iter().collect::<Vec<_>>()),
                indices: Arc::from(indices.into_iter().collect::<Vec<_>>()),
            }));
    }
}
