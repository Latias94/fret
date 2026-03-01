use fret_core::{Point, Rect};
use fret_ui_headless::tab_strip_surface::classify_tab_strip_surface;

use crate::tab_drag::WorkspaceTabHitRect;

pub(crate) use fret_ui_headless::tab_strip_surface::TabStripSurface as WorkspaceTabStripSurface;

pub(crate) fn classify_workspace_tab_strip_surface(
    position: Point,
    dragged_tab_id: &str,
    tab_rects: &[WorkspaceTabHitRect],
    pinned_boundary_rect: Option<Rect>,
    end_drop_target_rect: Option<Rect>,
    scroll_viewport_rect: Option<Rect>,
    overflow_control_rect: Option<Rect>,
    scroll_left_control_rect: Option<Rect>,
    scroll_right_control_rect: Option<Rect>,
) -> WorkspaceTabStripSurface {
    classify_tab_strip_surface(
        position,
        tab_rects,
        |r| r.rect,
        |r| r.id.as_ref() == dragged_tab_id,
        pinned_boundary_rect,
        end_drop_target_rect,
        scroll_viewport_rect,
        overflow_control_rect,
        scroll_left_control_rect,
        scroll_right_control_rect,
    )
}
