use super::super::ElementHostWidget;
use crate::declarative::frame::layout_style_for_node;
use crate::declarative::layout_helpers::clamp_to_constraints;
use crate::declarative::prelude::*;
use crate::layout_constraints::LayoutConstraints;
use crate::layout_constraints::{AvailableSpace as RuntimeAvailableSpace, LayoutSize};
use crate::widget::MeasureCx;

impl ElementHostWidget {
    pub(super) fn layout_flex_impl<H: UiHost>(
        &mut self,
        cx: &mut LayoutCx<'_, H>,
        window: AppWindowId,
        props: FlexProps,
    ) -> Size {
        self.layout_flex_impl_engine(cx, window, props)
    }

    fn layout_flex_impl_engine<H: UiHost>(
        &mut self,
        cx: &mut LayoutCx<'_, H>,
        window: AppWindowId,
        props: FlexProps,
    ) -> Size {
        if cx.pass_kind == crate::layout_pass::LayoutPassKind::Probe {
            let constraints = LayoutConstraints::new(
                LayoutSize::new(None, None),
                LayoutSize::new(
                    RuntimeAvailableSpace::Definite(cx.available.width),
                    RuntimeAvailableSpace::Definite(cx.available.height),
                ),
            );

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

        let needs_engine_solve = cx.children.iter().copied().any(|child| {
            cx.tree
                .layout_engine_child_local_rect(cx.node, child)
                .is_none()
        });
        if needs_engine_solve {
            let missing_child = cx.children.iter().copied().find(|&child| {
                cx.tree
                    .layout_engine_child_local_rect(cx.node, child)
                    .is_none()
            });
            cx.tree.record_layout_engine_widget_fallback_solve(
                cx.app,
                window,
                cx.node,
                "flex",
                missing_child,
            );

            // Prefer a window-scoped solve, but fall back to a barrier-style solve for this root
            // when the subtree is not already covered by an outer viewport/root compute.
            cx.tree.solve_barrier_flow_root(
                cx.app,
                cx.services,
                cx.node,
                cx.bounds,
                cx.scale_factor,
            );
        }

        let pad_left = props.padding.left.0.max(0.0);
        let pad_right = props.padding.right.0.max(0.0);
        let pad_top = props.padding.top.0.max(0.0);
        let pad_bottom = props.padding.bottom.0.max(0.0);
        let pad_w = pad_left + pad_right;
        let pad_h = pad_top + pad_bottom;
        let inner_size = Size::new(
            Px((cx.available.width.0 - pad_w).max(0.0)),
            Px((cx.available.height.0 - pad_h).max(0.0)),
        );
        let auto_margin_inner_size = inner_size;

        let mut ml_auto_tail_group_start: Option<usize> = None;
        let mut ml_auto_tail_shift_x = 0.0f32;
        if props.direction == fret_core::Axis::Horizontal && cx.children.len() > 1 {
            for (idx, &child) in cx.children.iter().enumerate() {
                let child_style = layout_style_for_node(cx.app, window, child);
                if matches!(child_style.margin.left, crate::element::MarginEdge::Auto) {
                    if idx + 1 < cx.children.len() {
                        ml_auto_tail_group_start = Some(idx);
                    }
                    break;
                }
            }
        }

        if let Some(start) = ml_auto_tail_group_start {
            let mut tail_right = 0.0f32;
            for &child in &cx.children[start..] {
                let Some(layout) = cx.tree.layout_engine_child_local_rect(cx.node, child) else {
                    continue;
                };
                let child_style = layout_style_for_node(cx.app, window, child);
                let right_margin = match child_style.margin.right {
                    crate::element::MarginEdge::Px(px) => px.0,
                    crate::element::MarginEdge::Auto => 0.0,
                };
                let x = layout.origin.x.0 - pad_left;
                tail_right = tail_right.max(x + layout.size.width.0 + right_margin);
            }
            ml_auto_tail_shift_x = (auto_margin_inner_size.width.0 - tail_right).max(0.0);
        }

        for (child_index, &child) in cx.children.iter().enumerate() {
            let Some(layout) = cx.tree.layout_engine_child_local_rect(cx.node, child) else {
                continue;
            };
            let child_style = layout_style_for_node(cx.app, window, child);
            let single_child = cx.children.len() == 1;

            // The layout engine reports child rects in the parent's local coordinate space.
            // The auto-margin adjustment logic below is expressed relative to the parent's inner
            // content origin (after padding), so subtract the padding offset before applying the
            // centering rules.
            let mut x = layout.origin.x.0 - pad_left;
            let mut y = layout.origin.y.0 - pad_top;

            let margin_left_auto =
                matches!(child_style.margin.left, crate::element::MarginEdge::Auto);
            let margin_right_auto =
                matches!(child_style.margin.right, crate::element::MarginEdge::Auto);
            let margin_top_auto =
                matches!(child_style.margin.top, crate::element::MarginEdge::Auto);
            let margin_bottom_auto =
                matches!(child_style.margin.bottom, crate::element::MarginEdge::Auto);

            let margin_px = |edge: crate::element::MarginEdge| match edge {
                crate::element::MarginEdge::Px(px) => px.0,
                crate::element::MarginEdge::Auto => 0.0,
            };

            match props.direction {
                fret_core::Axis::Horizontal => {
                    if ml_auto_tail_group_start == Some(child_index) {
                        // Preserve the explicit `gap` between the auto-margin item and the next
                        // sibling. Some layout-engine outcomes collapse that gap when `ml-auto`
                        // is present, but web flexbox keeps it intact.
                        if let Some(&next_child) = cx.children.get(child_index + 1) {
                            if let Some(next_layout) =
                                cx.tree.layout_engine_child_local_rect(cx.node, next_child)
                            {
                                let next_x = next_layout.origin.x.0 - pad_left;
                                let desired = (next_x - props.gap.0 - layout.size.width.0).max(0.0);
                                x = x.min(desired);
                            }
                        }
                    }

                    if margin_left_auto || margin_right_auto {
                        // Partial support for CSS-like auto margins in flex rows.
                        //
                        // - `ml-auto` (left auto) pushes the item to the end of the main axis
                        //   while keeping the current sequential layout position as a floor (so we
                        //   don't overlap previous siblings when space is tight).
                        // - `mx-auto` (left+right auto) is treated as a centering hint, but only
                        //   when there is a single child.
                        let left = if margin_left_auto {
                            0.0
                        } else {
                            margin_px(child_style.margin.left)
                        };
                        let right = if margin_right_auto {
                            0.0
                        } else {
                            margin_px(child_style.margin.right)
                        };

                        if margin_left_auto && margin_right_auto {
                            if single_child {
                                let free = auto_margin_inner_size.width.0
                                    - layout.size.width.0
                                    - left
                                    - right;
                                x = (left + (free.max(0.0) / 2.0)).max(0.0);
                            }
                        } else if margin_left_auto {
                            if ml_auto_tail_group_start == Some(child_index) {
                                // When `margin-left: auto` appears on an item with following
                                // siblings, treat it as a flexible spacer that right-aligns the
                                // whole trailing group (like web flexbox does).
                            } else {
                                // Align to end (like `margin-left: auto`).
                                let desired = (auto_margin_inner_size.width.0
                                    - layout.size.width.0
                                    - margin_px(child_style.margin.right))
                                .max(0.0);
                                x = x.max(desired);
                            }
                        } else if margin_right_auto {
                            if single_child {
                                let free = auto_margin_inner_size.width.0
                                    - layout.size.width.0
                                    - left
                                    - right;
                                x = left.max(0.0).min((left + free.max(0.0)).max(0.0));
                            }
                        }
                    }

                    if margin_top_auto || margin_bottom_auto {
                        let top = if margin_top_auto {
                            0.0
                        } else {
                            margin_px(child_style.margin.top)
                        };
                        let bottom = if margin_bottom_auto {
                            0.0
                        } else {
                            margin_px(child_style.margin.bottom)
                        };
                        let free =
                            auto_margin_inner_size.height.0 - layout.size.height.0 - top - bottom;
                        if margin_top_auto && margin_bottom_auto {
                            y = (top + (free.max(0.0) / 2.0)).max(0.0);
                        } else if margin_top_auto {
                            y = (top + free.max(0.0)).max(0.0);
                        } else if margin_bottom_auto {
                            y = top.max(0.0);
                        }
                    }
                }
                fret_core::Axis::Vertical => {
                    if single_child && (margin_top_auto || margin_bottom_auto) {
                        let top = if margin_top_auto {
                            0.0
                        } else {
                            margin_px(child_style.margin.top)
                        };
                        let bottom = if margin_bottom_auto {
                            0.0
                        } else {
                            margin_px(child_style.margin.bottom)
                        };
                        let free =
                            auto_margin_inner_size.height.0 - layout.size.height.0 - top - bottom;
                        if margin_top_auto && margin_bottom_auto {
                            y = (top + (free.max(0.0) / 2.0)).max(0.0);
                        } else if margin_top_auto {
                            y = (top + free.max(0.0)).max(0.0);
                        } else if margin_bottom_auto {
                            y = top.max(0.0);
                        }
                    }

                    if margin_left_auto || margin_right_auto {
                        let left = if margin_left_auto {
                            0.0
                        } else {
                            margin_px(child_style.margin.left)
                        };
                        let right = if margin_right_auto {
                            0.0
                        } else {
                            margin_px(child_style.margin.right)
                        };
                        let free =
                            auto_margin_inner_size.width.0 - layout.size.width.0 - left - right;
                        if margin_left_auto && margin_right_auto {
                            x = (left + (free.max(0.0) / 2.0)).max(0.0);
                        } else if margin_left_auto {
                            x = (left + free.max(0.0)).max(0.0);
                        } else if margin_right_auto {
                            x = left.max(0.0);
                        }
                    }
                }
            }

            if ml_auto_tail_shift_x > 0.0
                && ml_auto_tail_group_start.is_some_and(|start| child_index >= start)
            {
                x += ml_auto_tail_shift_x;
            }

            let local_x = x + pad_left;
            let local_y = y + pad_top;
            let rect = Rect::new(
                fret_core::Point::new(
                    Px(cx.bounds.origin.x.0 + local_x),
                    Px(cx.bounds.origin.y.0 + local_y),
                ),
                layout.size,
            );

            let _ = cx.layout_in(child, rect);
        }

        clamp_to_constraints(cx.available, props.layout, cx.available)
    }
}
