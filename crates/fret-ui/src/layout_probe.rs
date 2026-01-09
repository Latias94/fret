use fret_core::{Axis, Size};

/// Available-size sentinel used by legacy layout probing.
///
/// Some containers historically approximate `MinContent` / `MaxContent` by running a normal
/// layout pass with a very large definite bound (e.g. `Px(1.0e9)`). During those "probe" passes we
/// must avoid consuming one-shot state (like deferred scroll requests) and avoid doing expensive
/// precomputation that would be invalid under the real viewport constraints.
pub(crate) const PROBE_LAYOUT_SENTINEL_PX: f32 = 1.0e8;

pub(crate) fn is_probe_layout_any_axis(available: Size) -> bool {
    available.width.0 >= PROBE_LAYOUT_SENTINEL_PX || available.height.0 >= PROBE_LAYOUT_SENTINEL_PX
}

pub(crate) fn is_probe_layout_axis(axis: Axis, available: Size) -> bool {
    match axis {
        Axis::Vertical => available.height.0 >= PROBE_LAYOUT_SENTINEL_PX,
        Axis::Horizontal => available.width.0 >= PROBE_LAYOUT_SENTINEL_PX,
    }
}
