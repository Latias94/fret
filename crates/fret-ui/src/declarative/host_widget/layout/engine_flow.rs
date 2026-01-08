use crate::declarative::frame::{ElementInstance, element_record_for_node, layout_style_for_node};
use crate::declarative::prelude::*;
use crate::declarative::taffy_layout::*;
use crate::layout_engine::TaffyLayoutEngine;
use crate::widget::LayoutCx;
use fret_core::{AppWindowId, NodeId, Size};

#[derive(Debug, Clone, Copy)]
pub(super) enum ParentLayoutKind {
    Flex { direction: fret_core::Axis },
    Grid,
}

pub(super) fn layout_children_from_engine_if_solved<H: UiHost>(
    cx: &mut LayoutCx<'_, H>,
) -> Option<Size> {
    let mut child_bounds: Vec<(NodeId, Rect)> = Vec::with_capacity(cx.children.len());
    for &child in cx.children {
        let bounds = cx.layout_engine_child_bounds(child)?;
        child_bounds.push((child, bounds));
    }
    for (child, bounds) in child_bounds {
        let _ = cx.layout_in(child, bounds);
    }
    Some(cx.available)
}

pub(super) fn configure_flow_subtree<H: UiHost>(
    engine: &mut TaffyLayoutEngine,
    cx: &mut LayoutCx<'_, H>,
    window: AppWindowId,
    parent_kind: ParentLayoutKind,
    node: NodeId,
) {
    let _ = engine.request_layout_node(node);
    if let Some(child) = passthrough_wrapper_child(cx, window, node) {
        let mut style = style_for_item_in_parent(cx, window, parent_kind, node, Display::Flex);
        style.flex_direction = FlexDirection::Column;
        style.align_items = Some(TaffyAlignItems::Stretch);
        style.justify_content = Some(JustifyContent::FlexStart);

        if let Some(props) = element_record_for_node(cx.app, window, node).and_then(|r| {
            if let ElementInstance::Container(p) = r.instance {
                Some(p)
            } else {
                None
            }
        }) {
            style.padding = taffy::geometry::Rect {
                left: LengthPercentage::length(props.padding.left.0.max(0.0)),
                right: LengthPercentage::length(props.padding.right.0.max(0.0)),
                top: LengthPercentage::length(props.padding.top.0.max(0.0)),
                bottom: LengthPercentage::length(props.padding.bottom.0.max(0.0)),
            };
        }

        engine.set_style(node, style);
        engine.set_children(node, &[child]);
        engine.set_measured(node, false);
        configure_flow_subtree(
            engine,
            cx,
            window,
            ParentLayoutKind::Flex {
                direction: fret_core::Axis::Vertical,
            },
            child,
        );
        return;
    }

    let instance = element_record_for_node(cx.app, window, node).map(|r| r.instance);
    match instance {
        Some(ElementInstance::Flex(props)) => {
            let mut style = style_for_item_in_parent(cx, window, parent_kind, node, Display::Flex);
            style.flex_direction = match props.direction {
                fret_core::Axis::Horizontal => FlexDirection::Row,
                fret_core::Axis::Vertical => FlexDirection::Column,
            };
            style.flex_wrap = if props.wrap {
                FlexWrap::Wrap
            } else {
                FlexWrap::NoWrap
            };
            style.justify_content = Some(taffy_justify(props.justify));
            style.align_items = Some(taffy_align_items(props.align));
            style.gap = taffy::geometry::Size {
                width: LengthPercentage::length(props.gap.0.max(0.0)),
                height: LengthPercentage::length(props.gap.0.max(0.0)),
            };
            style.padding = taffy::geometry::Rect {
                left: LengthPercentage::length(props.padding.left.0.max(0.0)),
                right: LengthPercentage::length(props.padding.right.0.max(0.0)),
                top: LengthPercentage::length(props.padding.top.0.max(0.0)),
                bottom: LengthPercentage::length(props.padding.bottom.0.max(0.0)),
            };

            let children = cx.tree.children(node).to_vec();
            engine.set_style(node, style);
            engine.set_children(node, &children);
            engine.set_measured(node, false);
            for child in children {
                configure_flow_subtree(
                    engine,
                    cx,
                    window,
                    ParentLayoutKind::Flex {
                        direction: props.direction,
                    },
                    child,
                );
            }
        }
        Some(ElementInstance::Grid(props)) => {
            let mut style = style_for_item_in_parent(cx, window, parent_kind, node, Display::Grid);
            style.justify_content = Some(taffy_justify(props.justify));
            style.align_items = Some(taffy_align_items(props.align));
            style.gap = taffy::geometry::Size {
                width: LengthPercentage::length(props.gap.0.max(0.0)),
                height: LengthPercentage::length(props.gap.0.max(0.0)),
            };
            style.padding = taffy::geometry::Rect {
                left: LengthPercentage::length(props.padding.left.0.max(0.0)),
                right: LengthPercentage::length(props.padding.right.0.max(0.0)),
                top: LengthPercentage::length(props.padding.top.0.max(0.0)),
                bottom: LengthPercentage::length(props.padding.bottom.0.max(0.0)),
            };
            style.grid_template_columns = taffy::style_helpers::evenly_sized_tracks(props.cols);
            style.grid_template_rows = props
                .rows
                .map(taffy::style_helpers::evenly_sized_tracks)
                .unwrap_or_default();

            let children = cx.tree.children(node).to_vec();
            engine.set_style(node, style);
            engine.set_children(node, &children);
            engine.set_measured(node, false);
            for child in children {
                configure_flow_subtree(engine, cx, window, ParentLayoutKind::Grid, child);
            }
        }
        _ => {
            let style = style_for_item_in_parent(cx, window, parent_kind, node, Display::Block);
            engine.set_style(node, style);
            engine.set_children(node, &[]);
            engine.set_measured(node, true);
        }
    }
}

fn style_for_item_in_parent<H: UiHost>(
    cx: &mut LayoutCx<'_, H>,
    window: AppWindowId,
    parent_kind: ParentLayoutKind,
    node: NodeId,
    display: Display,
) -> TaffyStyle {
    let layout_style = layout_style_for_node(cx.app, window, node);

    let mut min_w = layout_style.size.min_width.map(|p| p.0);
    let mut min_h = layout_style.size.min_height.map(|p| p.0);
    if let ParentLayoutKind::Flex { direction } = parent_kind {
        let spacer_min = element_record_for_node(cx.app, window, node).and_then(|r| {
            if let ElementInstance::Spacer(p) = r.instance {
                Some(p.min)
            } else {
                None
            }
        });
        if let Some(min) = spacer_min {
            let min = min.0.max(0.0);
            match direction {
                fret_core::Axis::Horizontal => {
                    min_w = Some(min_w.unwrap_or(0.0).max(min));
                }
                fret_core::Axis::Vertical => {
                    min_h = Some(min_h.unwrap_or(0.0).max(min));
                }
            }
        }
    }

    let mut style = TaffyStyle {
        display,
        position: taffy_position(layout_style.position),
        inset: taffy_rect_lpa_from_inset(layout_style.position, layout_style.inset),
        size: taffy::geometry::Size {
            width: taffy_dimension(layout_style.size.width),
            height: taffy_dimension(layout_style.size.height),
        },
        aspect_ratio: layout_style.aspect_ratio,
        min_size: taffy::geometry::Size {
            width: min_w.map(Dimension::length).unwrap_or_else(Dimension::auto),
            height: min_h.map(Dimension::length).unwrap_or_else(Dimension::auto),
        },
        max_size: taffy::geometry::Size {
            width: layout_style
                .size
                .max_width
                .map(|p| Dimension::length(p.0))
                .unwrap_or_else(Dimension::auto),
            height: layout_style
                .size
                .max_height
                .map(|p| Dimension::length(p.0))
                .unwrap_or_else(Dimension::auto),
        },
        margin: taffy_rect_lpa_from_margin_edges(layout_style.margin),
        ..Default::default()
    };

    match parent_kind {
        ParentLayoutKind::Flex { .. } => {
            style.flex_grow = layout_style.flex.grow;
            style.flex_shrink = layout_style.flex.shrink;
            style.flex_basis = taffy_dimension(layout_style.flex.basis);
            style.align_self = layout_style.flex.align_self.map(taffy_align_self);
        }
        ParentLayoutKind::Grid => {
            style.grid_column = taffy_grid_line(layout_style.grid.column);
            style.grid_row = taffy_grid_line(layout_style.grid.row);
        }
    }

    style
}

fn passthrough_wrapper_child<H: UiHost>(
    cx: &mut LayoutCx<'_, H>,
    window: AppWindowId,
    node: NodeId,
) -> Option<NodeId> {
    let layout_style = layout_style_for_node(cx.app, window, node);
    if layout_style.position != crate::element::PositionStyle::Static {
        return None;
    }
    if layout_style.inset.left.is_some()
        || layout_style.inset.right.is_some()
        || layout_style.inset.top.is_some()
        || layout_style.inset.bottom.is_some()
    {
        return None;
    }

    let children = cx.tree.children(node);
    if children.len() != 1 {
        return None;
    }
    let child = children[0];
    let child_style = layout_style_for_node(cx.app, window, child);
    if child_style.position != crate::element::PositionStyle::Static {
        return None;
    }

    let instance = element_record_for_node(cx.app, window, node).map(|r| r.instance)?;
    match instance {
        ElementInstance::Container(_)
        | ElementInstance::Pressable(_)
        | ElementInstance::Opacity(_)
        | ElementInstance::VisualTransform(_)
        | ElementInstance::Semantics(_)
        | ElementInstance::FocusScope(_)
        | ElementInstance::Stack(_) => Some(child),
        _ => None,
    }
}
