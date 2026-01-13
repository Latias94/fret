use super::super::ElementHostWidget;
use crate::declarative::layout_helpers::clamp_to_constraints;
use crate::declarative::prelude::*;
use crate::declarative::taffy_layout::*;
use crate::layout_constraints::LayoutConstraints;
use crate::layout_constraints::{AvailableSpace as RuntimeAvailableSpace, LayoutSize};
use crate::layout_engine::{ParentLayoutKind, layout_children_from_engine_if_solved};
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
        window: AppWindowId,
        props: crate::element::GridProps,
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

        if let Some(size) = layout_children_from_engine_if_solved(cx) {
            return size;
        }

        let pad_left = props.padding.left.0.max(0.0);
        let pad_right = props.padding.right.0.max(0.0);
        let pad_top = props.padding.top.0.max(0.0);
        let pad_bottom = props.padding.bottom.0.max(0.0);
        let pad_w = pad_left + pad_right;
        let pad_h = pad_top + pad_bottom;
        let inner_origin = fret_core::Point::new(
            Px(cx.bounds.origin.x.0 + pad_left),
            Px(cx.bounds.origin.y.0 + pad_top),
        );

        let outer_avail_w = match props.layout.size.width {
            Length::Px(px) => Px(px.0.min(cx.available.width.0.max(0.0))),
            Length::Fill | Length::Auto => cx.available.width,
        };
        let outer_avail_h = match props.layout.size.height {
            Length::Px(px) => Px(px.0.min(cx.available.height.0.max(0.0))),
            Length::Fill | Length::Auto => cx.available.height,
        };

        let inner_avail = Size::new(
            Px((outer_avail_w.0 - pad_w).max(0.0)),
            Px((outer_avail_h.0 - pad_h).max(0.0)),
        );

        let sf = if cx.scale_factor.is_finite() && cx.scale_factor > 0.0 {
            cx.scale_factor
        } else {
            1.0
        };

        let root_style = TaffyStyle {
            display: Display::Grid,
            justify_content: Some(taffy_justify(props.justify)),
            align_items: Some(taffy_align_items(props.align)),
            gap: TaffySize {
                width: LengthPercentage::length(props.gap.0.max(0.0) * sf),
                height: LengthPercentage::length(props.gap.0.max(0.0) * sf),
            },
            size: TaffySize {
                width: match props.layout.size.width {
                    Length::Px(px) => Dimension::length((px.0 - pad_w).max(0.0) * sf),
                    Length::Fill => Dimension::length(inner_avail.width.0.max(0.0) * sf),
                    Length::Auto => Dimension::auto(),
                },
                height: match props.layout.size.height {
                    Length::Px(px) => Dimension::length((px.0 - pad_h).max(0.0) * sf),
                    Length::Fill => Dimension::length(inner_avail.height.0.max(0.0) * sf),
                    Length::Auto => Dimension::auto(),
                },
            },
            grid_template_columns: taffy::style_helpers::evenly_sized_tracks(props.cols),
            grid_template_rows: props
                .rows
                .map(taffy::style_helpers::evenly_sized_tracks)
                .unwrap_or_default(),
            ..Default::default()
        };

        let available = LayoutSize::new(
            RuntimeAvailableSpace::Definite(inner_avail.width),
            RuntimeAvailableSpace::Definite(inner_avail.height),
        );

        let (root_layout, child_layouts) = cx.tree.solve_flow_island_with_root_style(
            cx.app,
            cx.services,
            window,
            cx.node,
            root_style,
            cx.children,
            ParentLayoutKind::Grid,
            available,
            sf,
        );
        let inner_size = root_layout.size;

        for (child, layout_rect) in child_layouts {
            let rect = Rect::new(
                fret_core::Point::new(
                    Px(inner_origin.x.0 + layout_rect.origin.x.0),
                    Px(inner_origin.y.0 + layout_rect.origin.y.0),
                ),
                layout_rect.size,
            );
            let _ = cx.layout_in(child, rect);
        }

        let desired = Size::new(
            Px((inner_size.width.0 + pad_w).max(0.0)),
            Px((inner_size.height.0 + pad_h).max(0.0)),
        );
        clamp_to_constraints(desired, props.layout, cx.available)
    }
}
