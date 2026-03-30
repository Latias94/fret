use fret_core::Px;
use fret_ui::{ElementContext, Invalidation, UiHost};

use crate::{MetricRef, PaddingRefinement};

use super::{occlusion_insets_or_zero, safe_area_insets_or_zero};

/// Returns a `PaddingRefinement` that applies a uniform base padding plus runner-committed window
/// insets (safe-area + occlusion) for the current window (ADR 0232).
///
/// This helper is intended to keep app content readable on mobile:
/// - safe-area padding avoids notches / rounded corners,
/// - occlusion padding avoids transient obstructions like the on-screen keyboard (IME).
#[track_caller]
pub fn window_insets_padding_refinement_or_zero<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    invalidation: Invalidation,
    base_padding_px: Px,
) -> PaddingRefinement {
    let safe_area = safe_area_insets_or_zero(cx, invalidation);
    let occlusion = occlusion_insets_or_zero(cx, invalidation);

    let base = base_padding_px.0.max(0.0);
    let top = base + safe_area.top.0.max(0.0) + occlusion.top.0.max(0.0);
    let right = base + safe_area.right.0.max(0.0) + occlusion.right.0.max(0.0);
    let bottom = base + safe_area.bottom.0.max(0.0) + occlusion.bottom.0.max(0.0);
    let left = base + safe_area.left.0.max(0.0) + occlusion.left.0.max(0.0);

    PaddingRefinement {
        top: Some(MetricRef::Px(Px(top))),
        right: Some(MetricRef::Px(Px(right))),
        bottom: Some(MetricRef::Px(Px(bottom))),
        left: Some(MetricRef::Px(Px(left))),
    }
}
