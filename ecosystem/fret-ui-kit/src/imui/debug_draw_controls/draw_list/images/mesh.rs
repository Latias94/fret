use std::sync::Arc;

use fret_core::ImageId;

use super::super::super::commands::DebugDrawCommand;
use super::super::super::{DebugDrawImageMeshOptions, DebugDrawVertex, ImUiDebugDrawList};

impl ImUiDebugDrawList {
    pub fn add_image_triangle_mesh<V, I>(&mut self, image: ImageId, vertices: V, indices: I)
    where
        V: IntoIterator<Item = DebugDrawVertex>,
        I: IntoIterator<Item = u32>,
    {
        self.add_image_triangle_mesh_with_options(
            image,
            vertices,
            indices,
            DebugDrawImageMeshOptions::default(),
        );
    }

    pub fn add_image_triangle_mesh_with_options<V, I>(
        &mut self,
        image: ImageId,
        vertices: V,
        indices: I,
        options: DebugDrawImageMeshOptions,
    ) where
        V: IntoIterator<Item = DebugDrawVertex>,
        I: IntoIterator<Item = u32>,
    {
        self.commands.push(DebugDrawCommand::ImageTriangleMesh {
            image,
            vertices: Arc::from(vertices.into_iter().collect::<Vec<_>>()),
            indices: Arc::from(indices.into_iter().collect::<Vec<_>>()),
            options,
        });
    }
}
