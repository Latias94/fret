use fret_core::{Color, ImageId, Point, Px, Rect, UvPoint, UvRect};
use fret_ui::SvgSource;

use crate::imui::debug_draw_controls::{
    DebugDrawImageOptions, DebugDrawImageQuadOptions, DebugDrawRoundCorners, DebugDrawSvgOptions,
};

// This file owns raster, rounded-image, and SVG debug-draw command payload variants.

#[derive(Debug, Clone)]
pub(in crate::imui::debug_draw_controls) enum DebugDrawMediaCommand {
    Image {
        rect: Rect,
        image: ImageId,
        options: DebugDrawImageOptions,
    },
    ImageRegion {
        rect: Rect,
        image: ImageId,
        uv: UvRect,
        options: DebugDrawImageOptions,
    },
    ImageQuad {
        image: ImageId,
        points: [Point; 4],
        uvs: [UvPoint; 4],
        options: DebugDrawImageQuadOptions,
    },
    ImageRounded {
        rect: Rect,
        image: ImageId,
        options: DebugDrawImageOptions,
        rounding: Px,
        corners: DebugDrawRoundCorners,
    },
    ImageRegionRounded {
        rect: Rect,
        image: ImageId,
        uv: UvRect,
        options: DebugDrawImageOptions,
        rounding: Px,
        corners: DebugDrawRoundCorners,
    },
    SvgImage {
        rect: Rect,
        svg: SvgSource,
        options: DebugDrawSvgOptions,
    },
    SvgMaskIcon {
        rect: Rect,
        svg: SvgSource,
        color: Color,
        options: DebugDrawSvgOptions,
    },
}
