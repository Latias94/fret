use fret_core::{Color, Px, Rect};

use super::super::super::super::commands::DebugDrawCommand;
use super::super::super::super::{DebugDrawStrokeStyle, ImUiDebugDrawList};

impl ImUiDebugDrawList {
    pub fn add_rect(&mut self, rect: Rect, color: Color, thickness: Px) {
        self.add_rect_with_style(rect, color, thickness);
    }

    pub fn add_rect_with_style(
        &mut self,
        rect: Rect,
        color: Color,
        style: impl Into<DebugDrawStrokeStyle>,
    ) {
        self.commands.push(DebugDrawCommand::Rect {
            rect,
            color,
            style: style.into(),
        });
    }

    pub fn add_rect_filled(&mut self, rect: Rect, color: Color) {
        self.commands
            .push(DebugDrawCommand::RectFilled { rect, color });
    }

    pub fn add_rect_filled_multi_color(
        &mut self,
        rect: Rect,
        upper_left: Color,
        upper_right: Color,
        bottom_right: Color,
        bottom_left: Color,
    ) {
        self.commands.push(DebugDrawCommand::RectFilledMultiColor {
            rect,
            upper_left,
            upper_right,
            bottom_right,
            bottom_left,
        });
    }
}
