use std::collections::HashMap;

use fret_core::{
    DockGraph, DockNode, DockNodeId, Scene,
    geometry::{Px, Rect},
};

use super::super::split_geometry::{self, SplitHandle};

#[derive(Debug, Clone)]
pub(in crate::dock) struct SplitHandlePaintInput {
    node: DockNodeId,
    axis: fret_core::Axis,
    bounds: Rect,
    children_len: usize,
    fractions: Vec<f32>,
}

pub(in crate::dock) fn split_handle_paint_inputs(
    graph: &DockGraph,
    layout: &HashMap<DockNodeId, Rect>,
) -> Vec<SplitHandlePaintInput> {
    let mut inputs = Vec::new();
    for (&node, &bounds) in layout.iter() {
        let Some(DockNode::Split {
            axis,
            children,
            fractions,
        }) = graph.node(node)
        else {
            continue;
        };
        if children.len() < 2 {
            continue;
        }
        inputs.push(SplitHandlePaintInput {
            node,
            axis: *axis,
            bounds,
            children_len: children.len(),
            fractions: fractions.clone(),
        });
    }
    inputs
}

pub(in crate::dock) fn paint_split_handle_inputs(
    theme: fret_ui::ThemeSnapshot,
    inputs: &[SplitHandlePaintInput],
    active: Option<DockNodeId>,
    split_handle_gap: Px,
    split_handle_hit_thickness: Px,
    scale_factor: f32,
    scene: &mut Scene,
) {
    for input in inputs {
        let computed = split_geometry::compute_layout(
            input.axis,
            input.bounds,
            input.children_len,
            &input.fractions,
            split_handle_gap,
            split_handle_hit_thickness,
            &[],
        );

        let background = if active == Some(input.node) {
            theme.color_token("ring")
        } else {
            theme.color_token("border")
        };

        let handle = SplitHandle {
            axis: input.axis,
            paint_device_px: 1.0,
        };
        for center in computed.handle_centers {
            handle.paint(
                scene,
                // Keep split handle under component focus rings (typically DrawOrder(1)),
                // while still painting above panel backgrounds (DrawOrder(0)).
                fret_core::DrawOrder(0),
                input.bounds,
                center,
                scale_factor,
                background,
            );
        }
    }
}
