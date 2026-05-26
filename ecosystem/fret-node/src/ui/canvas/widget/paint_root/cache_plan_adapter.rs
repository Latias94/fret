//! Paint-root cache-plan adapter contract.
//!
//! This module keeps cache-plan route inputs behind a named seam. Concrete lifecycle-context
//! bindings live next to the retained paint root.

use fret_core::Rect;
use fret_ui::UiHost;

pub(super) trait PaintRootCachePlanCx<H: UiHost> {
    fn paint_root_cache_plan_host(&self) -> &H;
    fn paint_root_cache_plan_bounds(&self) -> Rect;
    fn paint_root_cache_plan_scale_factor(&self) -> f32;
}
