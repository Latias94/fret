use fret_core::{ImageId, Px, Rect, UvRect};

use super::super::super::commands::DebugDrawCommand;
use super::super::super::{DebugDrawImageOptions, DebugDrawRoundCorners, ImUiDebugDrawList};

impl ImUiDebugDrawList {
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
