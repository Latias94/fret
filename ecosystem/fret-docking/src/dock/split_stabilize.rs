// This file is part of the docking UI implementation.
//
// It is intentionally `pub(super)` only; the public API lives in `dock/mod.rs`.

use super::layout::compute_layout_map;
use super::prelude_core::*;
use fret_core::Axis;
use fret_ui::retained_bridge::resizable_panel_group as resizable;

#[derive(Debug, Clone, Copy)]
pub(super) struct SplitSizeLock {
    pub(super) split: DockNodeId,
    /// Which child subtree should keep its axis size stable (0 or 1).
    pub(super) preserve_child_ix: usize,
    /// Axis size (logical px) captured at drag start.
    pub(super) preserve_px: f32,
}

#[derive(Debug, Clone, Copy)]
enum BoundarySide {
    Start,
    End,
}

fn axis_len(rect: Rect, axis: Axis) -> f32 {
    match axis {
        Axis::Horizontal => rect.size.width.0,
        Axis::Vertical => rect.size.height.0,
    }
}

fn preserve_child_ix(side: BoundarySide) -> usize {
    match side {
        // Resizing the "end" edge should keep the "start" subtree stable.
        BoundarySide::End => 0,
        BoundarySide::Start => 1,
    }
}

fn boundary_child_ix(side: BoundarySide) -> usize {
    match side {
        BoundarySide::End => 1,
        BoundarySide::Start => 0,
    }
}

fn collect_same_axis_locks_in_subtree(
    graph: &DockGraph,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
    node: DockNodeId,
    axis: Axis,
    boundary: BoundarySide,
    out: &mut Vec<SplitSizeLock>,
) {
    let Some(n) = graph.node(node) else {
        return;
    };

    match n {
        DockNode::Tabs { .. } => {}
        DockNode::Floating { child } => {
            collect_same_axis_locks_in_subtree(graph, layout, *child, axis, boundary, out);
        }
        DockNode::Split {
            axis: split_axis,
            children,
            ..
        } => {
            if children.len() != 2 {
                // This stabilization currently only supports binary splits.
                return;
            }

            if *split_axis == axis {
                let preserve_ix = preserve_child_ix(boundary);
                let preserve = children[preserve_ix];
                let Some(preserve_rect) = layout.get(&preserve).copied() else {
                    return;
                };

                out.push(SplitSizeLock {
                    split: node,
                    preserve_child_ix: preserve_ix,
                    preserve_px: axis_len(preserve_rect, axis),
                });

                let boundary_ix = boundary_child_ix(boundary);
                collect_same_axis_locks_in_subtree(
                    graph,
                    layout,
                    children[boundary_ix],
                    axis,
                    boundary,
                    out,
                );
            } else {
                for &child in children {
                    collect_same_axis_locks_in_subtree(graph, layout, child, axis, boundary, out);
                }
            }
        }
    }
}

/// Compute nested-split stabilization locks for a binary split handle.
///
/// When resizing a splitter, we want only the *touching* leaf nodes to change size.
/// Same-axis nested splits should keep their "inner siblings" stable to avoid linked splitters.
pub(super) fn compute_same_axis_locks_for_split_drag(
    graph: &DockGraph,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
    split: DockNodeId,
    axis: Axis,
    handle_ix: usize,
) -> Vec<SplitSizeLock> {
    if handle_ix != 0 {
        // This stabilization is currently defined only for binary splits, so only handle 0 is supported.
        return Vec::new();
    }

    let Some(DockNode::Split {
        axis: a, children, ..
    }) = graph.node(split)
    else {
        return Vec::new();
    };
    if *a != axis || children.len() != 2 {
        return Vec::new();
    }

    let mut locks = Vec::new();
    collect_same_axis_locks_in_subtree(
        graph,
        layout,
        children[0],
        axis,
        BoundarySide::End,
        &mut locks,
    );
    collect_same_axis_locks_in_subtree(
        graph,
        layout,
        children[1],
        axis,
        BoundarySide::Start,
        &mut locks,
    );
    locks
}

pub(super) fn apply_same_axis_locks(
    graph: &mut DockGraph,
    layout_root: DockNodeId,
    layout_bounds: Rect,
    axis: Axis,
    split_handle_gap: Px,
    split_handle_hit_thickness: Px,
    locks: &[SplitSizeLock],
) {
    if locks.is_empty() {
        return;
    }

    for lock in locks {
        let layout = compute_layout_map(
            graph,
            layout_root,
            layout_bounds,
            split_handle_gap,
            split_handle_hit_thickness,
        );
        let Some(&bounds) = layout.get(&lock.split) else {
            continue;
        };

        let Some(DockNode::Split {
            axis: split_axis,
            children,
            fractions,
        }) = graph.node(lock.split)
        else {
            continue;
        };
        if *split_axis != axis || children.len() != 2 || fractions.len() != 2 {
            continue;
        }

        let computed = resizable::compute_layout(
            axis,
            bounds,
            2,
            fractions,
            split_handle_gap,
            split_handle_hit_thickness,
            &[],
        );
        if computed.avail <= 0.0 || computed.mins.len() != 2 {
            continue;
        }

        let preserve_ix = lock.preserve_child_ix;
        if preserve_ix > 1 {
            continue;
        }
        let other_ix = 1 - preserve_ix;

        let min_preserve = computed.mins[preserve_ix];
        let min_other = computed.mins[other_ix];
        let max_preserve = (computed.avail - min_other).clamp(0.0, computed.avail);

        // `mins` are already scaled so `min_preserve + min_other <= avail`.
        let preserve_size = lock.preserve_px.clamp(min_preserve, max_preserve);
        let preserve_fraction = (preserve_size / computed.avail).clamp(0.0, 1.0);
        let other_fraction = (1.0 - preserve_fraction).clamp(0.0, 1.0);

        let mut next = vec![0.0, 0.0];
        next[preserve_ix] = preserve_fraction;
        next[other_ix] = other_fraction;
        let _ = graph.update_split_fractions(lock.split, next);
    }
}
