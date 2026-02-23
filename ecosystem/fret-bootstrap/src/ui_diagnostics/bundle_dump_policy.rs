use super::*;

pub(super) struct DumpSemanticsPolicy {
    pub max_nodes: usize,
    pub test_ids_closure: bool,
}

fn env_usize_override(key: &str) -> Option<usize> {
    std::env::var(key)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .and_then(|v| v.parse::<usize>().ok())
}

fn env_flag_override(key: &str) -> Option<bool> {
    std::env::var(key)
        .ok()
        .map(|v| v.trim().to_ascii_lowercase())
        .filter(|v| !v.is_empty())
        .map(|v| !matches!(v.as_str(), "0" | "false" | "no" | "off"))
}

pub(super) fn resolve_dump_semantics_policy(
    cfg: &UiDiagnosticsConfig,
    is_script_dump: bool,
) -> DumpSemanticsPolicy {
    const DEFAULT_NON_SCRIPT_DUMP_MAX_SEMANTICS_NODES: usize = 10_000;

    let max_nodes = env_usize_override("FRET_DIAG_BUNDLE_DUMP_MAX_SEMANTICS_NODES")
        .unwrap_or_else(|| {
            if is_script_dump {
                cfg.max_semantics_nodes
            } else {
                cfg.max_semantics_nodes
                    .min(DEFAULT_NON_SCRIPT_DUMP_MAX_SEMANTICS_NODES)
            }
        })
        .clamp(0, 500_000);

    let test_ids_closure =
        env_flag_override("FRET_DIAG_BUNDLE_DUMP_SEMANTICS_TEST_IDS_ONLY").unwrap_or(false);

    DumpSemanticsPolicy {
        max_nodes,
        test_ids_closure,
    }
}

pub(super) fn apply_dump_semantics_policy_to_windows(
    windows: &mut [UiDiagnosticsWindowBundleV1],
    policy: &DumpSemanticsPolicy,
) {
    if policy.test_ids_closure {
        filter_bundle_semantics_nodes_to_test_ids_closure(windows);
    }
    clamp_bundle_semantics_nodes(windows, policy.max_nodes);
}

fn clamp_bundle_semantics_nodes(windows: &mut [UiDiagnosticsWindowBundleV1], max_nodes: usize) {
    for w in windows {
        for s in &mut w.snapshots {
            let Some(semantics) = s.debug.semantics.as_mut() else {
                continue;
            };
            if semantics.nodes.len() > max_nodes {
                semantics.nodes.truncate(max_nodes);
            }
        }
    }
}

fn filter_bundle_semantics_nodes_to_test_ids_closure(windows: &mut [UiDiagnosticsWindowBundleV1]) {
    use std::collections::{HashMap, HashSet};

    for w in windows {
        for s in &mut w.snapshots {
            let Some(semantics) = s.debug.semantics.as_mut() else {
                continue;
            };

            let mut parent_by_id: HashMap<u64, u64> = HashMap::new();
            for n in &semantics.nodes {
                if let Some(parent) = n.parent {
                    parent_by_id.insert(n.id, parent);
                }
            }

            let mut include: HashSet<u64> = HashSet::new();
            let mut stack: Vec<u64> = semantics
                .nodes
                .iter()
                .filter(|n| n.test_id.is_some())
                .map(|n| n.id)
                .collect();
            while let Some(id) = stack.pop() {
                if !include.insert(id) {
                    continue;
                }
                if let Some(parent) = parent_by_id.get(&id).copied() {
                    stack.push(parent);
                }
            }

            if include.is_empty() {
                semantics.nodes.clear();
                continue;
            }

            semantics.nodes.retain(|n| include.contains(&n.id));
        }
    }
}
