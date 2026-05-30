use fret_core::{ImageId, Point, Rect, UvPoint, UvRect};

use super::super::super::commands::DebugDrawCommand;
use super::super::super::{DebugDrawImageOptions, DebugDrawImageQuadOptions, ImUiDebugDrawList};

impl ImUiDebugDrawList {
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
}
