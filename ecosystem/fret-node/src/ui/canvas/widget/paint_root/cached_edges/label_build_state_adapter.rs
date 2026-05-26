//! Paint-root cached edge-label build-state adapter contract.
//!
//! This module keeps cached edge-label build-state route inputs behind a named seam. Concrete
//! retained host, service, and scale inputs live next to the cached edge-label build-state binding.

use fret_core::UiServices;
use fret_ui::UiHost;

pub(super) struct PaintRootCachedEdgeLabelBuildStateStepInputs<'a, H: UiHost> {
    pub(super) host: &'a H,
    pub(super) services: &'a mut dyn UiServices,
    pub(super) scale_factor: f32,
}

pub(super) trait PaintRootCachedEdgeLabelBuildStateCx<H: UiHost> {
    fn paint_root_cached_edge_label_build_state_host(&self) -> &H;

    fn paint_root_cached_edge_label_build_state_step_inputs(
        &mut self,
    ) -> PaintRootCachedEdgeLabelBuildStateStepInputs<'_, H>;
}

pub(super) fn paint_root_cached_edge_label_build_state_host<H>(
    cx: &impl PaintRootCachedEdgeLabelBuildStateCx<H>,
) -> &H
where
    H: UiHost,
{
    cx.paint_root_cached_edge_label_build_state_host()
}

pub(super) fn paint_root_cached_edge_label_build_state_step_inputs<H>(
    cx: &mut impl PaintRootCachedEdgeLabelBuildStateCx<H>,
) -> PaintRootCachedEdgeLabelBuildStateStepInputs<'_, H>
where
    H: UiHost,
{
    cx.paint_root_cached_edge_label_build_state_step_inputs()
}
