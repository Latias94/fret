use crate::ui::canvas::widget::*;

impl<H: UiHost> super::cache_plan_adapter::PaintRootCachePlanCx<H> for PaintCx<'_, H> {
    fn paint_root_cache_plan_host(&self) -> &H {
        &*self.app
    }

    fn paint_root_cache_plan_bounds(&self) -> Rect {
        self.bounds
    }

    fn paint_root_cache_plan_scale_factor(&self) -> f32 {
        self.scale_factor
    }
}
