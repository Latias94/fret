use fret_core::dock::DockGraph;
use fret_core::{AppWindowId, DockNode, DockNodeId, PanelKey};

/// Lightweight dock-graph diagnostics helpers.
///
/// These functions are primarily intended for scripted diagnostics gates and debugging tools.
/// They are **not** a stable public contract for persisted layouts.
pub fn dock_graph_stats_for_window(
    graph: &DockGraph,
    window: AppWindowId,
) -> fret_runtime::DockGraphStatsDiagnostics {
    use std::collections::HashSet;

    let mut node_count: u32 = 0;
    let mut tabs_count: u32 = 0;
    let mut split_count: u32 = 0;
    let mut floating_count: u32 = 0;
    let mut max_depth: u32 = 0;
    let mut max_split_depth: u32 = 0;

    let mut canonical_ok = true;
    let mut has_nested_same_axis_splits = false;

    let mut visited: HashSet<DockNodeId> = HashSet::new();
    let mut stack: Vec<(DockNodeId, u32, u32)> = Vec::new();

    if let Some(root) = graph.window_root(window) {
        stack.push((root, 1, 0));
    }
    for f in graph.floating_windows(window) {
        stack.push((f.floating, 1, 0));
    }

    while let Some((node, depth, split_depth)) = stack.pop() {
        if !visited.insert(node) {
            continue;
        }
        node_count = node_count.saturating_add(1);
        max_depth = max_depth.max(depth);
        max_split_depth = max_split_depth.max(split_depth);

        let Some(n) = graph.node(node) else {
            canonical_ok = false;
            continue;
        };

        match n {
            DockNode::Tabs { tabs, .. } => {
                tabs_count = tabs_count.saturating_add(1);
                if tabs.is_empty() {
                    canonical_ok = false;
                }
            }
            DockNode::Floating { child } => {
                floating_count = floating_count.saturating_add(1);
                stack.push((*child, depth.saturating_add(1), split_depth));
            }
            DockNode::Split {
                axis,
                children,
                fractions,
            } => {
                split_count = split_count.saturating_add(1);

                if children.len() < 2 || children.len() != fractions.len() {
                    canonical_ok = false;
                }

                let mut sum: f32 = 0.0;
                for f in fractions {
                    if !f.is_finite() || *f < 0.0 {
                        canonical_ok = false;
                    }
                    sum += *f;
                }
                if !sum.is_finite() || (sum - 1.0).abs() > 1.0e-3 {
                    canonical_ok = false;
                }

                for &child in children {
                    if let Some(DockNode::Split {
                        axis: child_axis, ..
                    }) = graph.node(child)
                        && child_axis == axis
                    {
                        has_nested_same_axis_splits = true;
                        canonical_ok = false;
                    }
                    stack.push((
                        child,
                        depth.saturating_add(1),
                        split_depth.saturating_add(1),
                    ));
                }
            }
        }
    }

    fret_runtime::DockGraphStatsDiagnostics {
        node_count,
        tabs_count,
        split_count,
        floating_count,
        max_depth,
        max_split_depth,
        canonical_ok,
        has_nested_same_axis_splits,
    }
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in bytes {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

pub fn dock_graph_signature_for_window(
    graph: &DockGraph,
    window: AppWindowId,
) -> fret_runtime::DockGraphSignatureDiagnostics {
    use std::collections::HashSet;

    fn panel_key_sig(p: &PanelKey) -> String {
        match &p.instance {
            Some(instance) if !instance.is_empty() => format!("{}#{}", p.kind.0, instance),
            _ => p.kind.0.clone(),
        }
    }

    fn node_sig(graph: &DockGraph, node: DockNodeId, visited: &mut HashSet<DockNodeId>) -> String {
        if !visited.insert(node) {
            return "cycle".to_string();
        }

        let Some(n) = graph.node(node) else {
            return "missing".to_string();
        };

        match n {
            DockNode::Tabs { tabs, active } => {
                let body = tabs.iter().map(panel_key_sig).collect::<Vec<_>>().join(",");
                if tabs.len() > 1 {
                    format!("tabs(a={active}:[{body}])")
                } else {
                    format!("tabs([{body}])")
                }
            }
            DockNode::Floating { child } => {
                let child_sig = node_sig(graph, *child, visited);
                format!("floating({child_sig})")
            }
            DockNode::Split { axis, children, .. } => {
                let axis = match axis {
                    fret_core::Axis::Horizontal => "h",
                    fret_core::Axis::Vertical => "v",
                };
                let child_sigs = children
                    .iter()
                    .map(|c| node_sig(graph, *c, visited))
                    .collect::<Vec<_>>()
                    .join(",");
                format!("split({axis},[{child_sigs}])")
            }
        }
    }

    let root_sig = graph
        .window_root(window)
        .map(|root| node_sig(graph, root, &mut HashSet::new()))
        .unwrap_or_else(|| "none".to_string());

    let mut floating_sigs: Vec<String> = graph
        .floating_windows(window)
        .iter()
        .map(|f| node_sig(graph, f.floating, &mut HashSet::new()))
        .collect();
    floating_sigs.sort();

    let signature = format!(
        "dock(root={root_sig};floatings=[{}])",
        floating_sigs.join(",")
    );
    let fingerprint64 = fnv1a64(signature.as_bytes());

    fret_runtime::DockGraphSignatureDiagnostics {
        signature,
        fingerprint64,
    }
}

