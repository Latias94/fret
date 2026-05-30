//! Shared helpers for component shape token access.

use fret_core::Corners;
use fret_ui::Theme;

pub(crate) fn uniform_corners_from_metric(theme: &Theme, key: &str) -> Option<Corners> {
    theme.metric_by_key(key).map(Corners::all)
}

pub(crate) fn corners_or_metric(theme: &Theme, key: &str) -> Option<Corners> {
    theme
        .corners_by_key(key)
        .or_else(|| uniform_corners_from_metric(theme, key))
}
