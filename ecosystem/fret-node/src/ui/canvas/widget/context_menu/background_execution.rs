mod activate;
mod apply;
mod plan;
mod retained_cx;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;

use crate::ui::canvas::widget::*;

#[derive(Debug)]
pub(super) enum BackgroundInsertMenuPlan {
    Apply(Vec<GraphOp>),
    Reject(DiagnosticSeverity, Arc<str>),
    Ignore,
}

pub(in crate::ui::canvas::widget) trait BackgroundInsertMenuCx<H: UiHost> {
    fn host(&mut self) -> &mut H;
    fn window(&self) -> Option<AppWindowId>;
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn plan_background_insert_menu_candidate<H: UiHost>(
        &mut self,
        host: &mut H,
        candidate: &InsertNodeCandidate,
        at: CanvasPoint,
    ) -> BackgroundInsertMenuPlan {
        plan::plan_background_insert_menu_candidate(self, host, candidate, at)
    }

    pub(super) fn apply_background_insert_menu_plan<H: UiHost>(
        &mut self,
        cx: &mut impl BackgroundInsertMenuCx<H>,
        plan: BackgroundInsertMenuPlan,
    ) {
        apply::apply_background_insert_menu_plan(self, cx, plan)
    }

    pub(super) fn activate_background_context_action<H: UiHost>(
        &mut self,
        cx: &mut impl BackgroundInsertMenuCx<H>,
        at: CanvasPoint,
        action: NodeGraphContextMenuAction,
        menu_candidates: &[InsertNodeCandidate],
    ) -> bool {
        activate::activate_background_context_action(self, cx, at, action, menu_candidates)
    }
}
