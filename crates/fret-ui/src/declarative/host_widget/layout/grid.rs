use super::super::ElementHostWidget;
use crate::declarative::layout_helpers::clamp_to_constraints;
use crate::declarative::prelude::*;
use crate::layout_constraints::AvailableSpace as RuntimeAvailableSpace;
use crate::layout_engine::layout_children_from_engine_if_solved;
use crate::widget::MeasureCx;

impl ElementHostWidget {
    pub(super) fn layout_grid_impl<H: UiHost>(
        &mut self,
        cx: &mut LayoutCx<'_, H>,
        window: AppWindowId,
        props: crate::element::GridProps,
    ) -> Size {
        self.layout_grid_impl_engine(cx, window, props)
    }

    fn layout_grid_impl_engine<H: UiHost>(
        &mut self,
        cx: &mut LayoutCx<'_, H>,
        _window: AppWindowId,
        props: crate::element::GridProps,
    ) -> Size {
        if cx.pass_kind == crate::layout_pass::LayoutPassKind::Probe {
            let mut constraints = cx.probe_constraints_for_size(cx.available);
            if cx.available.width.0 <= 0.0 {
                constraints.available.width = RuntimeAvailableSpace::MaxContent;
            }
            if cx.available.height.0 <= 0.0 {
                constraints.available.height = RuntimeAvailableSpace::MaxContent;
            }

            // Avoid re-entrant `with_widget_mut(cx.node)` by measuring the current widget directly.
            let mut measure_cx = MeasureCx {
                app: cx.app,
                tree: cx.tree,
                node: cx.node,
                window: cx.window,
                focus: cx.focus,
                children: cx.children,
                constraints,
                scale_factor: cx.scale_factor,
                services: cx.services,
                observe_model: cx.observe_model,
                observe_global: cx.observe_global,
            };
            return self.measure_impl(&mut measure_cx);
        }

        if cx.children.is_empty() {
            return clamp_to_constraints(cx.available, props.layout, cx.available);
        }

        if layout_children_from_engine_if_solved(cx).is_none() {
            cx.tree.record_layout_engine_widget_fallback_solve(
                cx.app,
                _window,
                cx.node,
                "grid",
                cx.children.first().copied(),
            );
            cx.tree.solve_barrier_flow_root(
                cx.app,
                cx.services,
                cx.node,
                cx.bounds,
                cx.scale_factor,
            );
            let _ = layout_children_from_engine_if_solved(cx);
        }

        clamp_to_constraints(cx.available, props.layout, cx.available)
    }
}
