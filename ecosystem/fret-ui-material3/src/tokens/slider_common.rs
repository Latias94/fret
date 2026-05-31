//! Shared default matrices for Material 3 Slider token fallbacks.

use fret_core::{Corners, Px};

pub(crate) fn state_layer_size() -> Px {
    Px(40.0)
}

pub(crate) fn value_indicator_bottom_space() -> Px {
    Px(12.0)
}

pub(crate) fn tick_mark_size() -> Px {
    Px(2.0)
}

pub(crate) fn tick_mark_opacity() -> f32 {
    0.38
}

pub(crate) fn stop_indicator_size() -> Px {
    Px(4.0)
}

pub(crate) fn stop_indicator_trailing_space() -> Px {
    Px(4.0)
}

pub(crate) fn disabled_content_opacity() -> f32 {
    0.38
}

pub(crate) fn disabled_inactive_track_opacity() -> f32 {
    0.12
}

pub(crate) fn full_shape() -> Corners {
    Corners::all(Px(9999.0))
}

pub(crate) fn selected_stop_indicator_opacity() -> f32 {
    1.0
}

pub(crate) fn unselected_stop_indicator_opacity() -> f32 {
    1.0
}

pub(crate) fn track_height() -> Px {
    Px(16.0)
}

pub(crate) fn handle_height() -> Px {
    Px(44.0)
}

pub(crate) fn handle_resting_width() -> Px {
    Px(4.0)
}

pub(crate) fn handle_pressed_width() -> Px {
    Px(2.0)
}

pub(crate) fn handle_focused_width() -> Px {
    Px(2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slider_state_layer_and_value_indicator_defaults_match_material_matrix() {
        assert_eq!(state_layer_size(), Px(40.0));
        assert_eq!(value_indicator_bottom_space(), Px(12.0));
    }

    #[test]
    fn slider_tick_and_stop_indicator_defaults_match_material_matrix() {
        assert_eq!(tick_mark_size(), Px(2.0));
        assert_eq!(tick_mark_opacity(), 0.38);
        assert_eq!(stop_indicator_size(), Px(4.0));
        assert_eq!(stop_indicator_trailing_space(), Px(4.0));
        assert_eq!(selected_stop_indicator_opacity(), 1.0);
        assert_eq!(unselected_stop_indicator_opacity(), 1.0);
    }

    #[test]
    fn slider_track_and_handle_defaults_match_material_matrix() {
        assert_eq!(track_height(), Px(16.0));
        assert_eq!(handle_height(), Px(44.0));
        assert_eq!(handle_resting_width(), Px(4.0));
        assert_eq!(handle_pressed_width(), Px(2.0));
        assert_eq!(handle_focused_width(), Px(2.0));
    }

    #[test]
    fn slider_full_shape_uses_unbounded_radius() {
        assert_eq!(full_shape(), Corners::all(Px(9999.0)));
    }
}
