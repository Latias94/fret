//! Collapsible-style height transitions.
//!
//! Upstream Radix Collapsible/Accordion coordinate mount/unmount via `Presence` and expose measured
//! content dimensions for CSS keyframe animations (e.g. `--radix-collapsible-content-height`).
//!
//! Fret does not use CSS variables. Instead, we cache the last known open height and drive a
//! clipped wrapper height using `Presence`'s eased progress.

use fret_core::Px;
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::primitives::presence::PresenceOutput;
use crate::{LayoutRefinement, MetricRef};

#[derive(Debug, Clone, Copy)]
struct MeasuredHeightState {
    last_height: Px,
}

impl Default for MeasuredHeightState {
    fn default() -> Self {
        Self {
            last_height: Px(0.0),
        }
    }
}

/// Read the last cached open height for a collapsible content subtree.
pub fn last_measured_height_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    state_id: GlobalElementId,
) -> Px {
    cx.with_state_for(state_id, MeasuredHeightState::default, |st| st.last_height)
}

/// Update the cached open height from the previously-laid-out bounds of `wrapper_element_id`.
///
/// This should be called from the same element scope that renders the wrapper (so the wrapper ID
/// is stable), but the cached value can be stored on a separate `state_id` (typically the root).
pub fn update_measured_height_if_open_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    state_id: GlobalElementId,
    wrapper_element_id: GlobalElementId,
    open: bool,
    animating: bool,
) -> Px {
    let last_height = last_measured_height_for(cx, state_id);

    if !open || animating {
        return last_height;
    }

    let Some(bounds) = cx.last_bounds_for_element(wrapper_element_id) else {
        return last_height;
    };

    let h = bounds.size.height;
    if h.0 <= 0.0 || (h.0 - last_height.0).abs() <= 0.5 {
        return last_height;
    }

    cx.with_state_for(state_id, MeasuredHeightState::default, |st| {
        st.last_height = h;
    });
    h
}

/// Compute wrapper mounting and layout patches for a collapsible content subtree.
///
/// When a measurement exists, the wrapper height is driven using `presence.opacity` as an eased
/// 0..1 progress value. Without a measurement, call sites should avoid "close presence" to prevent
/// hidden content from affecting layout.
pub fn collapsible_height_wrapper_refinement(
    open: bool,
    force_mount: bool,
    require_measurement_for_close: bool,
    presence: PresenceOutput,
    measured_height: Px,
) -> (bool, LayoutRefinement) {
    let has_measurement = measured_height.0 > 0.0;
    let progress = presence.opacity.clamp(0.0, 1.0);

    let keep_mounted_for_close =
        presence.present && (!require_measurement_for_close || has_measurement);
    let should_render = force_mount || open || keep_mounted_for_close;

    let wants_height_animation = has_measurement && (presence.animating || !open);

    let mut wrapper = LayoutRefinement::default()
        .w_full()
        .min_w_0()
        .overflow_hidden();
    if wants_height_animation {
        wrapper = wrapper.h_px(MetricRef::Px(Px(measured_height.0 * progress)));
    } else if !open && force_mount {
        wrapper = wrapper.h_px(MetricRef::Px(Px(0.0)));
    }

    (should_render, wrapper)
}
