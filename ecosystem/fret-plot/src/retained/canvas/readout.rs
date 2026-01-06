//! Cursor/readout helpers.

use super::super::layers::PlotCursorReadoutRow;
use super::super::style::ReadoutSeriesPolicy;
use crate::series::SeriesId;

pub(super) fn apply_readout_policy(
    rows: &mut Vec<PlotCursorReadoutRow>,
    pinned: Option<SeriesId>,
    legend_hover: Option<SeriesId>,
    policy: ReadoutSeriesPolicy,
) {
    match policy {
        ReadoutSeriesPolicy::PinnedOrAll => {
            if let Some(pinned) = pinned {
                rows.retain(|r| r.series_id == pinned);
            }
        }
        ReadoutSeriesPolicy::PinnedOnly => {
            if let Some(pinned) = pinned {
                rows.retain(|r| r.series_id == pinned);
            } else {
                rows.clear();
            }
        }
        ReadoutSeriesPolicy::PinnedOrLegendHoverOrAll => {
            if let Some(pinned) = pinned {
                rows.retain(|r| r.series_id == pinned);
            } else if let Some(hovered) = legend_hover {
                rows.retain(|r| r.series_id == hovered);
            }
        }
    }
}
