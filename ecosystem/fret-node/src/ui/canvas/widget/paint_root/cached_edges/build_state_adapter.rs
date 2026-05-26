//! Paint-root cached edge build-state adapter contract.
//!
//! This module keeps cached edge build-state route inputs behind a named seam. Concrete retained
//! host, service, and scale inputs live next to the cached edge build-state binding.

use fret_core::UiServices;
use fret_ui::UiHost;

pub(super) struct PaintRootCachedEdgeBuildStateStepInputs<'a, H: UiHost> {
    pub(super) host: &'a H,
    pub(super) services: &'a mut dyn UiServices,
    pub(super) scale_factor: f32,
}

pub(super) trait PaintRootCachedEdgeBuildStateCx<H: UiHost> {
    fn paint_root_cached_edge_build_state_host(&self) -> &H;

    fn paint_root_cached_edge_build_state_step_inputs(
        &mut self,
    ) -> PaintRootCachedEdgeBuildStateStepInputs<'_, H>;
}

pub(super) fn paint_root_cached_edge_build_state_host<H>(
    cx: &impl PaintRootCachedEdgeBuildStateCx<H>,
) -> &H
where
    H: UiHost,
{
    cx.paint_root_cached_edge_build_state_host()
}

pub(super) fn paint_root_cached_edge_build_state_step_inputs<H>(
    cx: &mut impl PaintRootCachedEdgeBuildStateCx<H>,
) -> PaintRootCachedEdgeBuildStateStepInputs<'_, H>
where
    H: UiHost,
{
    cx.paint_root_cached_edge_build_state_step_inputs()
}
