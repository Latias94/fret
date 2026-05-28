use super::super::super::{
    DEFAULT_PATH_ARC_SEGMENTS, DEFAULT_PATH_BEZIER_SEGMENTS, DEFAULT_PATH_ELLIPTICAL_ARC_SEGMENTS,
};

pub(in crate::imui::debug_draw_controls) fn path_arc_segments(segments: usize) -> usize {
    if segments == 0 {
        DEFAULT_PATH_ARC_SEGMENTS
    } else {
        segments
    }
}

pub(in crate::imui::debug_draw_controls) fn path_bezier_segments(segments: usize) -> usize {
    if segments == 0 {
        DEFAULT_PATH_BEZIER_SEGMENTS
    } else {
        segments
    }
}

pub(in crate::imui::debug_draw_controls) fn path_elliptical_arc_segments(segments: usize) -> usize {
    if segments == 0 {
        DEFAULT_PATH_ELLIPTICAL_ARC_SEGMENTS
    } else {
        segments
    }
}
