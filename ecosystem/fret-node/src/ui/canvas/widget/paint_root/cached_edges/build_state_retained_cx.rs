use fret_ui::{PaintCx, UiHost};

impl<H: UiHost> super::build_state_adapter::PaintRootCachedEdgeBuildStateCx<H> for PaintCx<'_, H> {
    fn paint_root_cached_edge_build_state_host(&self) -> &H {
        &*self.app
    }

    fn paint_root_cached_edge_build_state_step_inputs(
        &mut self,
    ) -> super::build_state_adapter::PaintRootCachedEdgeBuildStateStepInputs<'_, H> {
        let scale_factor = self.scale_factor;
        super::build_state_adapter::PaintRootCachedEdgeBuildStateStepInputs {
            host: &*self.app,
            services: &mut *self.services,
            scale_factor,
        }
    }
}
