//! Typed token access for Material 3 progress indicators.

use fret_core::{Color, Corners, Px};
use fret_ui::Theme;

use crate::tokens::progress_indicator_common;

pub(crate) fn track_color(theme: &Theme) -> Color {
    progress_indicator_common::track_color(theme)
}

pub(crate) fn active_color(theme: &Theme) -> Color {
    progress_indicator_common::active_color(theme)
}

pub(crate) fn four_color_palette(theme: &Theme) -> [Color; 4] {
    progress_indicator_common::four_color_palette(theme)
}

pub(crate) fn track_shape(theme: &Theme) -> Corners {
    progress_indicator_common::track_shape(theme)
}

pub(crate) fn active_shape(theme: &Theme) -> Corners {
    progress_indicator_common::active_shape(theme)
}

pub(crate) fn linear_height(theme: &Theme) -> Px {
    progress_indicator_common::linear_height(theme)
}

pub(crate) fn linear_track_thickness(theme: &Theme) -> Px {
    progress_indicator_common::linear_track_thickness(theme)
}

pub(crate) fn linear_active_thickness(theme: &Theme) -> Px {
    progress_indicator_common::linear_active_thickness(theme)
}

pub(crate) fn circular_size(theme: &Theme) -> Px {
    progress_indicator_common::circular_size(theme)
}

pub(crate) fn circular_track_thickness(theme: &Theme) -> Px {
    progress_indicator_common::circular_track_thickness(theme)
}

pub(crate) fn circular_active_thickness(theme: &Theme) -> Px {
    progress_indicator_common::circular_active_thickness(theme)
}
