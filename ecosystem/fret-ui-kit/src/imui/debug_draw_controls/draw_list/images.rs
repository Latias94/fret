use std::sync::Arc;

use fret_core::{ImageId, Point, Px, Rect, UvPoint, UvRect};

use super::super::commands::DebugDrawCommand;
use super::super::{
    DebugDrawImageMeshOptions, DebugDrawImageOptions, DebugDrawImageQuadOptions,
    DebugDrawRoundCorners, DebugDrawVertex, ImUiDebugDrawList,
};

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

    pub fn add_image(&mut self, rect: Rect, image: ImageId) {
        self.add_image_with_options(rect, image, DebugDrawImageOptions::default());
    }

    pub fn add_image_with_options(
        &mut self,
        rect: Rect,
        image: ImageId,
        options: DebugDrawImageOptions,
    ) {
        self.commands.push(DebugDrawCommand::Image {
            rect,
            image,
            options,
        });
    }

    pub fn add_image_region(
        &mut self,
        rect: Rect,
        image: ImageId,
        uv: UvRect,
        options: DebugDrawImageOptions,
    ) {
        self.commands.push(DebugDrawCommand::ImageRegion {
            rect,
            image,
            uv,
            options,
        });
    }

    pub fn add_image_quad(&mut self, image: ImageId, points: [Point; 4], uvs: [UvPoint; 4]) {
        self.add_image_quad_with_options(image, points, uvs, DebugDrawImageQuadOptions::default());
    }

    pub fn add_image_quad_with_options(
        &mut self,
        image: ImageId,
        points: [Point; 4],
        uvs: [UvPoint; 4],
        options: DebugDrawImageQuadOptions,
    ) {
        self.commands.push(DebugDrawCommand::ImageQuad {
            image,
            points,
            uvs,
            options,
        });
    }

    pub fn add_image_rounded(
        &mut self,
        rect: Rect,
        image: ImageId,
        rounding: Px,
        corners: DebugDrawRoundCorners,
    ) {
        self.add_image_rounded_with_options(
            rect,
            image,
            DebugDrawImageOptions::default(),
            rounding,
            corners,
        );
    }

    pub fn add_image_rounded_with_options(
        &mut self,
        rect: Rect,
        image: ImageId,
        options: DebugDrawImageOptions,
        rounding: Px,
        corners: DebugDrawRoundCorners,
    ) {
        self.commands.push(DebugDrawCommand::ImageRounded {
            rect,
            image,
            options,
            rounding,
            corners,
        });
    }

    pub fn add_image_region_rounded(
        &mut self,
        rect: Rect,
        image: ImageId,
        uv: UvRect,
        options: DebugDrawImageOptions,
        rounding: Px,
        corners: DebugDrawRoundCorners,
    ) {
        self.commands.push(DebugDrawCommand::ImageRegionRounded {
            rect,
            image,
            uv,
            options,
            rounding,
            corners,
        });
    }
}
