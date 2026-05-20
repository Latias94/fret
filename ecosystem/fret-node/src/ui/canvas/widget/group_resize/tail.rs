use crate::core::CanvasRect;
use crate::ui::canvas::state::GroupResize;
use crate::ui::canvas::widget::*;

pub(super) fn finish_group_resize_move<H, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl super::super::widget_tail::WidgetPaintInvalidationCx<H>,
    resize: &mut GroupResize,
    new_rect: CanvasRect,
) {
    update_resize_preview_state(resize, new_rect);
    canvas.interaction.group_resize = Some(resize.clone());
    super::super::widget_tail::invalidate_widget_paint(cx);
}

fn update_resize_preview_state(resize: &mut GroupResize, new_rect: CanvasRect) {
    if resize.current_rect != new_rect {
        resize.current_rect = new_rect;
        resize.preview_rev = resize.preview_rev.wrapping_add(1);
    }
}

#[cfg(test)]
mod tests {
    use fret_core::{Point, Px};

    use super::*;
    use crate::core::{CanvasPoint, CanvasSize, GroupId};

    fn rect(x: f32, y: f32, width: f32, height: f32) -> CanvasRect {
        CanvasRect {
            origin: CanvasPoint { x, y },
            size: CanvasSize { width, height },
        }
    }

    #[test]
    fn update_resize_preview_state_updates_rect_and_preview_rev() {
        let start_rect = rect(10.0, 20.0, 100.0, 80.0);
        let new_rect = rect(10.0, 20.0, 120.0, 90.0);
        let mut resize = GroupResize {
            group: GroupId::new(),
            start_pos: Point::new(Px(0.0), Px(0.0)),
            start_rect,
            current_rect: start_rect,
            preview_rev: 0,
        };

        update_resize_preview_state(&mut resize, new_rect);

        assert_eq!(resize.current_rect, new_rect);
        assert_eq!(resize.preview_rev, 1);
    }

    #[test]
    fn update_resize_preview_state_skips_noop_preview_rev() {
        let start_rect = rect(10.0, 20.0, 100.0, 80.0);
        let mut resize = GroupResize {
            group: GroupId::new(),
            start_pos: Point::new(Px(0.0), Px(0.0)),
            start_rect,
            current_rect: start_rect,
            preview_rev: 7,
        };

        update_resize_preview_state(&mut resize, start_rect);

        assert_eq!(resize.preview_rev, 7);
    }
}
