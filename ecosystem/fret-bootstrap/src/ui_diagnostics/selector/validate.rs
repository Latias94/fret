use std::collections::HashSet;

use fret_core::{AppWindowId, SemanticsRole};
use fret_ui::elements::ElementRuntime;
use slotmap::Key as _;

use super::super::{UiDiagnosticsConfig, UiSelectorV1, UiSemanticsNodeV1};
use super::{
    SemanticsIndex, is_redacted_string, parse_semantics_role, selector_ancestors_for,
    semantics_role_label,
};

fn candidate_selectors_for_node(
    snapshot: &fret_core::SemanticsSnapshot,
    raw_node: &fret_core::SemanticsNode,
    exported_node: &UiSemanticsNodeV1,
    element: Option<u64>,
    cfg: &UiDiagnosticsConfig,
) -> Vec<UiSelectorV1> {
    let mut out = Vec::new();

    if let Some(id) = raw_node.test_id.as_deref() {
        out.push(UiSelectorV1::TestId {
            id: id.to_string(),
            root_z_index: None,
        });
    }

    let role = semantics_role_label(raw_node.role).to_string();
    if let Some(name) = exported_node.label.as_deref()
        && !(cfg.redact_text && is_redacted_string(name))
    {
        // Prefer shorter paths when possible; the validated selection path will pick the
        // shortest unique candidate.
        let ancestors = selector_ancestors_for(snapshot, raw_node);
        for suffix_len in 1..=ancestors.len() {
            let suffix = ancestors[ancestors.len() - suffix_len..].to_vec();
            out.push(UiSelectorV1::RoleAndPath {
                role: role.clone(),
                name: name.to_string(),
                ancestors: suffix,
                root_z_index: None,
            });
        }
        out.push(UiSelectorV1::RoleAndName {
            role: role.clone(),
            name: name.to_string(),
            root_z_index: None,
        });
    }

    if let Some(element) = element {
        out.push(UiSelectorV1::GlobalElementId {
            element,
            root_z_index: None,
        });
    }

    out.push(UiSelectorV1::NodeId {
        node: raw_node.id.data().as_ffi(),
        root_z_index: None,
    });
    out
}

#[derive(Debug, Clone, Copy)]
struct SelectorEvalSummary {
    match_count: u32,
    chosen_node_id: Option<u64>,
    note: Option<&'static str>,
}

fn eval_selector_scoped(
    snapshot: &fret_core::SemanticsSnapshot,
    window: AppWindowId,
    element_runtime: Option<&ElementRuntime>,
    selector: &UiSelectorV1,
    scope_root: Option<u64>,
) -> SelectorEvalSummary {
    let index = SemanticsIndex::new(snapshot);
    let mut matches: Vec<&fret_core::SemanticsNode> = Vec::new();
    let mut note: Option<&'static str> = None;
    let want_root_z_index = match selector {
        UiSelectorV1::RoleAndName { root_z_index, .. } => *root_z_index,
        UiSelectorV1::RoleAndPath { root_z_index, .. } => *root_z_index,
        UiSelectorV1::TestId { root_z_index, .. } => *root_z_index,
        UiSelectorV1::GlobalElementId { root_z_index, .. } => *root_z_index,
        UiSelectorV1::NodeId { root_z_index, .. } => *root_z_index,
    };

    let in_scope = |id: u64| -> bool {
        scope_root
            .map(|root| index.is_descendant_of_or_self(id, root))
            .unwrap_or(true)
    };
    let matches_root_z = |id: u64| -> bool {
        want_root_z_index
            .map(|z| index.root_z_for(id) == z)
            .unwrap_or(true)
    };

    match selector {
        UiSelectorV1::NodeId { node, .. } => {
            if let Some(n) = index.by_id.get(node).copied().filter(|n| {
                let id = n.id.data().as_ffi();
                index.is_selectable(id) && in_scope(id) && matches_root_z(id)
            }) {
                matches.push(n);
            }
        }
        UiSelectorV1::RoleAndName { role, name, .. } => {
            let Some(role) = parse_semantics_role(role) else {
                return SelectorEvalSummary {
                    match_count: 0,
                    chosen_node_id: None,
                    note: Some("invalid_role"),
                };
            };

            matches.extend(snapshot.nodes.iter().filter(|n| {
                let id = n.id.data().as_ffi();
                index.is_selectable(id)
                    && in_scope(id)
                    && matches_root_z(id)
                    && n.role == role
                    && n.label.as_deref() == Some(name)
            }));
        }
        UiSelectorV1::RoleAndPath {
            role,
            name,
            ancestors,
            ..
        } => {
            let Some(role) = parse_semantics_role(role) else {
                return SelectorEvalSummary {
                    match_count: 0,
                    chosen_node_id: None,
                    note: Some("invalid_role"),
                };
            };

            let mut parsed_ancestors: Vec<(SemanticsRole, &str)> =
                Vec::with_capacity(ancestors.len());
            for a in ancestors {
                let Some(r) = parse_semantics_role(&a.role) else {
                    return SelectorEvalSummary {
                        match_count: 0,
                        chosen_node_id: None,
                        note: Some("invalid_ancestor_role"),
                    };
                };
                parsed_ancestors.push((r, a.name.as_str()));
            }

            matches.extend(snapshot.nodes.iter().filter(|n| {
                let id = n.id.data().as_ffi();
                index.is_selectable(id)
                    && in_scope(id)
                    && matches_root_z(id)
                    && n.role == role
                    && n.label.as_deref() == Some(name)
                    && index.ancestors_match_subsequence(n.parent, &parsed_ancestors)
            }));
        }
        UiSelectorV1::TestId { id, .. } => {
            matches.extend(snapshot.nodes.iter().filter(|n| {
                let node_id = n.id.data().as_ffi();
                index.is_selectable(node_id)
                    && in_scope(node_id)
                    && matches_root_z(node_id)
                    && n.test_id.as_deref() == Some(id)
            }));
            if matches.is_empty()
                && super::extend_test_id_chrome_fallback(
                    snapshot,
                    &index,
                    id,
                    &in_scope,
                    &matches_root_z,
                    &mut matches,
                )
            {
                note = Some("fallback_chrome_suffix");
            }
            if matches.is_empty() {
                // Fallback for debugging: allow selecting hidden nodes if no visible match exists.
                note = Some("fallback_hidden_nodes");
                matches.extend(snapshot.nodes.iter().filter(|n| {
                    let node_id = n.id.data().as_ffi();
                    in_scope(node_id) && matches_root_z(node_id) && n.test_id.as_deref() == Some(id)
                }));
            }
        }
        UiSelectorV1::GlobalElementId { element, .. } => {
            let Some(node) = element_runtime.and_then(|runtime| {
                runtime.node_for_element(window, fret_ui::elements::GlobalElementId(*element))
            }) else {
                return SelectorEvalSummary {
                    match_count: 0,
                    chosen_node_id: None,
                    note: Some("element_runtime_missing"),
                };
            };
            let node_id = node.data().as_ffi();
            if let Some(n) = index.by_id.get(&node_id).copied().filter(|n| {
                let id = n.id.data().as_ffi();
                index.is_selectable(id) && in_scope(id) && matches_root_z(id)
            }) {
                matches.push(n);
            }
        }
    }

    let match_count = matches.len().min(u32::MAX as usize) as u32;
    let chosen = super::super::pick::pick_best_match(matches.iter().copied(), &index);
    let chosen_node_id = chosen.map(|n| n.id.data().as_ffi());

    SelectorEvalSummary {
        match_count,
        chosen_node_id,
        note,
    }
}

pub(in super::super) fn best_selector_for_node_validated(
    snapshot: &fret_core::SemanticsSnapshot,
    window: AppWindowId,
    element_runtime: Option<&ElementRuntime>,
    raw_node: &fret_core::SemanticsNode,
    element: Option<u64>,
    cfg: &UiDiagnosticsConfig,
) -> Option<UiSelectorV1> {
    let exported =
        UiSemanticsNodeV1::from_node(raw_node, cfg.redact_text, cfg.max_debug_string_bytes);
    let candidates = candidate_selectors_for_node(snapshot, raw_node, &exported, element, cfg);
    let target_node_id = raw_node.id.data().as_ffi();
    let index = SemanticsIndex::new(snapshot);
    let target_root_z_index = index.root_z_for(target_node_id);

    for selector in candidates {
        // `NodeId` is always unique but is intentionally an in-run-only reference.
        // Keep it out of the "validated" fast path so callers can fall back to more stable selectors.
        if matches!(selector, UiSelectorV1::NodeId { .. }) {
            continue;
        }
        let eval = eval_selector_scoped(snapshot, window, element_runtime, &selector, None);
        if eval.match_count == 1 && eval.chosen_node_id == Some(target_node_id) {
            return Some(selector);
        }
    }

    // If nothing is unique across multiple roots, try again with root-z gating.
    let candidates = candidate_selectors_for_node(snapshot, raw_node, &exported, element, cfg);
    for selector in candidates {
        if matches!(selector, UiSelectorV1::NodeId { .. }) {
            continue;
        }

        let gated = match selector {
            UiSelectorV1::RoleAndName {
                role,
                name,
                root_z_index: None,
            } => UiSelectorV1::RoleAndName {
                role,
                name,
                root_z_index: Some(target_root_z_index),
            },
            UiSelectorV1::RoleAndPath {
                role,
                name,
                ancestors,
                root_z_index: None,
            } => UiSelectorV1::RoleAndPath {
                role,
                name,
                ancestors,
                root_z_index: Some(target_root_z_index),
            },
            UiSelectorV1::TestId {
                id,
                root_z_index: None,
            } => UiSelectorV1::TestId {
                id,
                root_z_index: Some(target_root_z_index),
            },
            UiSelectorV1::GlobalElementId {
                element,
                root_z_index: None,
            } => UiSelectorV1::GlobalElementId {
                element,
                root_z_index: Some(target_root_z_index),
            },
            other => other,
        };

        let eval = eval_selector_scoped(snapshot, window, element_runtime, &gated, None);
        if eval.match_count == 1 && eval.chosen_node_id == Some(target_node_id) {
            return Some(gated);
        }
    }

    None
}

pub(in super::super) fn inspect_selector_candidates_report(
    snapshot: &fret_core::SemanticsSnapshot,
    window: AppWindowId,
    element_runtime: Option<&ElementRuntime>,
    raw_node: &fret_core::SemanticsNode,
    element: Option<u64>,
    cfg: &UiDiagnosticsConfig,
) -> String {
    let exported =
        UiSemanticsNodeV1::from_node(raw_node, cfg.redact_text, cfg.max_debug_string_bytes);
    let mut candidates = candidate_selectors_for_node(snapshot, raw_node, &exported, element, cfg);

    // Dedupe by stable JSON encoding (selectors can repeat for short ancestor paths).
    let mut seen: HashSet<String> = HashSet::new();
    candidates.retain(|sel| {
        let Ok(json) = serde_json::to_string(sel) else {
            return false;
        };
        seen.insert(json)
    });

    let target_node_id = raw_node.id.data().as_ffi();

    let mut scored: Vec<(i32, String)> = Vec::new();
    for selector in candidates {
        let eval = eval_selector_scoped(snapshot, window, element_runtime, &selector, None);
        let chosen_is_target = eval.chosen_node_id == Some(target_node_id);

        let (kind, base): (&'static str, i32) = match &selector {
            UiSelectorV1::TestId { .. } => ("test_id", 900),
            UiSelectorV1::RoleAndPath { .. } => ("role_path", 850),
            UiSelectorV1::RoleAndName { .. } => ("role_name", 700),
            UiSelectorV1::GlobalElementId { .. } => ("global_element_id", 650),
            UiSelectorV1::NodeId { .. } => ("node_id", 100),
        };

        let mut score = base;
        if eval.match_count == 0 {
            score -= 500;
        } else if eval.match_count == 1 {
            score += 300;
        } else {
            score -= (eval.match_count.saturating_sub(1) as i32) * 50;
        }

        if chosen_is_target {
            score += 500;
        } else {
            score -= 1000;
        }

        if let UiSelectorV1::RoleAndPath { ancestors, .. } = &selector {
            score -= (ancestors.len() as i32) * 5;
        }

        let selector_json =
            serde_json::to_string(&selector).unwrap_or_else(|_| "<unserializable>".to_string());
        let note = eval.note.unwrap_or("-");
        let chosen = eval
            .chosen_node_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "-".to_string());

        let line = format!(
            "- score={score} ok={chosen_is_target} matches={} chosen={chosen} kind={kind} note={note} selector={selector_json}",
            eval.match_count
        );
        scored.push((score, line));
    }

    scored.sort_by(|(a, _), (b, _)| b.cmp(a));
    scored
        .into_iter()
        .take(12)
        .map(|(_, line)| line)
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{
        NodeId, Point, Px, Rect, SemanticsActions, SemanticsFlags, SemanticsNode, SemanticsRoot,
        SemanticsSnapshot, Size,
    };
    use slotmap::KeyData;

    fn node_id(id: u64) -> NodeId {
        NodeId::from(KeyData::from_ffi(id))
    }

    fn window_id(id: u64) -> AppWindowId {
        AppWindowId::from(KeyData::from_ffi(id))
    }

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
    }

    fn semantics_node(
        id: u64,
        parent: Option<u64>,
        role: SemanticsRole,
        bounds: Rect,
        label: &str,
        test_id: Option<&str>,
    ) -> SemanticsNode {
        SemanticsNode {
            id: node_id(id),
            parent: parent.map(node_id),
            role,
            bounds,
            flags: SemanticsFlags::default(),
            test_id: test_id.map(|s| s.to_string()),
            active_descendant: None,
            pos_in_set: None,
            set_size: None,
            label: Some(label.to_string()),
            value: None,
            extra: Default::default(),
            text_selection: None,
            text_composition: None,
            actions: SemanticsActions::default(),
            labelled_by: Vec::new(),
            described_by: Vec::new(),
            controls: Vec::new(),
            inline_spans: Vec::new(),
        }
    }

    #[test]
    fn best_selector_validated_prefers_unique_role_path_when_test_id_ambiguous() {
        let window = window_id(1);

        let snapshot = SemanticsSnapshot {
            window,
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![
                semantics_node(
                    1,
                    None,
                    SemanticsRole::Window,
                    rect(0.0, 0.0, 100.0, 100.0),
                    "root",
                    None,
                ),
                semantics_node(
                    10,
                    Some(1),
                    SemanticsRole::Group,
                    rect(0.0, 0.0, 100.0, 10.0),
                    "toolbar",
                    None,
                ),
                semantics_node(
                    11,
                    Some(1),
                    SemanticsRole::Group,
                    rect(0.0, 90.0, 100.0, 10.0),
                    "footer",
                    None,
                ),
                semantics_node(
                    2,
                    Some(10),
                    SemanticsRole::Button,
                    rect(0.0, 0.0, 10.0, 10.0),
                    "OK",
                    Some("dup"),
                ),
                semantics_node(
                    3,
                    Some(11),
                    SemanticsRole::Button,
                    rect(0.0, 90.0, 10.0, 10.0),
                    "OK",
                    Some("dup"),
                ),
            ],
        };

        let mut cfg = UiDiagnosticsConfig::default();
        cfg.redact_text = false;
        let node = &snapshot.nodes[3];

        let sel = best_selector_for_node_validated(&snapshot, window, None, node, None, &cfg)
            .expect("expected a validated selector");

        match sel {
            UiSelectorV1::RoleAndPath { ancestors, .. } => {
                assert!(
                    ancestors.iter().any(|a| a.name == "toolbar"),
                    "expected the selector path to include the unique ancestor"
                );
            }
            other => panic!("expected role+path selector, got {other:?}"),
        }
    }

    #[test]
    fn selector_root_z_index_gates_test_id_resolution() {
        let window = window_id(1);

        let snapshot = SemanticsSnapshot {
            window,
            roots: vec![
                SemanticsRoot {
                    root: node_id(1),
                    visible: true,
                    blocks_underlay_input: false,
                    hit_testable: true,
                    z_index: 0,
                },
                SemanticsRoot {
                    root: node_id(100),
                    visible: true,
                    blocks_underlay_input: false,
                    hit_testable: true,
                    z_index: 10,
                },
            ],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![
                semantics_node(
                    1,
                    None,
                    SemanticsRole::Window,
                    rect(0.0, 0.0, 100.0, 100.0),
                    "root-a",
                    None,
                ),
                semantics_node(
                    100,
                    None,
                    SemanticsRole::Window,
                    rect(0.0, 0.0, 100.0, 100.0),
                    "root-b",
                    None,
                ),
                semantics_node(
                    2,
                    Some(1),
                    SemanticsRole::Button,
                    rect(0.0, 0.0, 10.0, 10.0),
                    "OK",
                    Some("dup"),
                ),
                semantics_node(
                    3,
                    Some(100),
                    SemanticsRole::Button,
                    rect(0.0, 0.0, 10.0, 10.0),
                    "OK",
                    Some("dup"),
                ),
            ],
        };

        let selector = UiSelectorV1::TestId {
            id: "dup".to_string(),
            root_z_index: Some(0),
        };
        let picked =
            super::super::select_semantics_node_scoped(&snapshot, window, None, &selector, None)
                .unwrap();
        assert_eq!(picked.id, node_id(2));

        let selector = UiSelectorV1::TestId {
            id: "dup".to_string(),
            root_z_index: Some(10),
        };
        let picked =
            super::super::select_semantics_node_scoped(&snapshot, window, None, &selector, None)
                .unwrap();
        assert_eq!(picked.id, node_id(3));
    }
}
