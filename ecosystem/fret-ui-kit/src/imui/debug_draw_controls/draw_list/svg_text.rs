use std::sync::Arc;

use fret_core::{Color, Point, Px, Rect};
use fret_ui::SvgSource;

use super::super::commands::DebugDrawCommand;
use super::super::{DebugDrawSvgOptions, ImUiDebugDrawList};

impl ImUiDebugDrawList {
    pub fn add_svg_image(&mut self, rect: Rect, svg: SvgSource) {
        self.add_svg_image_with_options(rect, svg, DebugDrawSvgOptions::default());
    }

    pub fn add_svg_image_with_options(
        &mut self,
        rect: Rect,
        svg: SvgSource,
        options: DebugDrawSvgOptions,
    ) {
        self.commands
            .push(DebugDrawCommand::SvgImage { rect, svg, options });
    }

    pub fn add_svg_mask_icon(&mut self, rect: Rect, svg: SvgSource, color: Color) {
        self.add_svg_mask_icon_with_options(rect, svg, color, DebugDrawSvgOptions::default());
    }

    pub fn add_svg_mask_icon_with_options(
        &mut self,
        rect: Rect,
        svg: SvgSource,
        color: Color,
        options: DebugDrawSvgOptions,
    ) {
        self.commands.push(DebugDrawCommand::SvgMaskIcon {
            rect,
            svg,
            color,
            options,
        });
    }

    pub fn add_text(&mut self, origin: Point, text: impl Into<Arc<str>>, color: Color, size: Px) {
        self.commands.push(DebugDrawCommand::Text {
            origin,
            text: text.into(),
            color,
            size,
        });
    }
}
