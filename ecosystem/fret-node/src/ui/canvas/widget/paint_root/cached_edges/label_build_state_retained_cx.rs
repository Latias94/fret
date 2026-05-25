use fret_ui::{PaintCx, UiHost};

impl<H: UiHost> super::label_build_state_adapter::PaintRootCachedEdgeLabelBuildStateCx<H>
    for PaintCx<'_, H>
{
    fn paint_root_cached_edge_label_build_state_host(&self) -> &H {
        &*self.app
    }

    fn paint_root_cached_edge_label_build_state_step_inputs(
        &mut self,
    ) -> super::label_build_state_adapter::PaintRootCachedEdgeLabelBuildStateStepInputs<'_, H> {
        let scale_factor = self.scale_factor;
        super::label_build_state_adapter::PaintRootCachedEdgeLabelBuildStateStepInputs {
            host: &*self.app,
            services: &mut *self.services,
            scale_factor,
        }
    }
}
