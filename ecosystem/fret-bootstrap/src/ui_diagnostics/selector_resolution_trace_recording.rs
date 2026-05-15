fn select_semantics_node_with_trace<'a>(
    snapshot: &'a fret_core::SemanticsSnapshot,
    window: AppWindowId,
    element_runtime: Option<&ElementRuntime>,
    selector: &UiSelectorV1,
    scope_root: Option<u64>,
    step_index: u32,
    redact_text: bool,
    trace: &mut Vec<UiSelectorResolutionTraceEntryV1>,
) -> Option<&'a fret_core::SemanticsNode> {
    let index = SemanticsIndex::new(snapshot);
    let mut matches: Vec<&'a fret_core::SemanticsNode> = Vec::new();
    let mut note: Option<String> = None;
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
                note = Some("invalid_role".to_string());
                push_selector_resolution_trace(
                    trace,
                    UiSelectorResolutionTraceEntryV1 {
                        step_index,
                        selector: selector.clone(),
                        match_count: 0,
                        chosen_node_id: None,
                        candidates: Vec::new(),
                        note,
                    },
                );
                return None;
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
                note = Some("invalid_role".to_string());
                push_selector_resolution_trace(
                    trace,
                    UiSelectorResolutionTraceEntryV1 {
                        step_index,
                        selector: selector.clone(),
                        match_count: 0,
                        chosen_node_id: None,
                        candidates: Vec::new(),
                        note,
                    },
                );
                return None;
            };

            let mut parsed_ancestors: Vec<(SemanticsRole, &str)> =
                Vec::with_capacity(ancestors.len());
            for a in ancestors {
                let Some(r) = parse_semantics_role(&a.role) else {
                    note = Some("invalid_ancestor_role".to_string());
                    push_selector_resolution_trace(
                        trace,
                        UiSelectorResolutionTraceEntryV1 {
                            step_index,
                            selector: selector.clone(),
                            match_count: 0,
                            chosen_node_id: None,
                            candidates: Vec::new(),
                            note,
                        },
                    );
                    return None;
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
                && selector::extend_test_id_chrome_fallback(
                    snapshot,
                    &index,
                    id,
                    &in_scope,
                    &matches_root_z,
                    &mut matches,
                )
            {
                note = Some("fallback_chrome_suffix".to_string());
            }
        }
        UiSelectorV1::GlobalElementId { element, .. } => {
            let Some(node) = element_runtime.and_then(|runtime| {
                runtime.node_for_element(window, fret_ui::elements::GlobalElementId(*element))
            }) else {
                note = Some("element_runtime_missing".to_string());
                push_selector_resolution_trace(
                    trace,
                    UiSelectorResolutionTraceEntryV1 {
                        step_index,
                        selector: selector.clone(),
                        match_count: 0,
                        chosen_node_id: None,
                        candidates: Vec::new(),
                        note,
                    },
                );
                return None;
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

    if matches.is_empty() && note.is_none() {
        note =
            filtered_selector_match_note(snapshot, &index, selector, scope_root, want_root_z_index);
    }

    let match_count = matches.len().min(u32::MAX as usize) as u32;
    let chosen = pick_best_match(matches.iter().copied(), &index);
    let chosen_node_id = chosen.map(|n| n.id.data().as_ffi());

    let mut ranked: Vec<((u32, u32, u64), &'a fret_core::SemanticsNode)> = matches
        .iter()
        .copied()
        .map(|n| {
            let id = n.id.data().as_ffi();
            ((index.root_z_for(id), index.depth_for(id), id), n)
        })
        .collect();
    ranked.sort_by(|(a, _), (b, _)| b.cmp(a));

    let candidates: Vec<UiSelectorResolutionCandidateV1> = ranked
        .into_iter()
        .take(MAX_SELECTOR_TRACE_CANDIDATES)
        .map(|(_rank, n)| UiSelectorResolutionCandidateV1 {
            node_id: n.id.data().as_ffi(),
            role: semantics_role_label(n.role).to_string(),
            name: if redact_text { None } else { n.label.clone() },
            test_id: n.test_id.clone(),
        })
        .collect();

    push_selector_resolution_trace(
        trace,
        UiSelectorResolutionTraceEntryV1 {
            step_index,
            selector: selector.clone(),
            match_count,
            chosen_node_id,
            candidates,
            note,
        },
    );

    chosen
}

fn filtered_selector_match_note(
    snapshot: &fret_core::SemanticsSnapshot,
    index: &SemanticsIndex<'_>,
    selector: &UiSelectorV1,
    scope_root: Option<u64>,
    want_root_z_index: Option<u32>,
) -> Option<String> {
    let UiSelectorV1::TestId { id, .. } = selector else {
        return None;
    };

    let in_scope = |node_id: u64| -> bool {
        scope_root
            .map(|root| index.is_descendant_of_or_self(node_id, root))
            .unwrap_or(true)
    };
    let matches_root_z = |node_id: u64| -> bool {
        want_root_z_index
            .map(|z| index.root_z_for(node_id) == z)
            .unwrap_or(true)
    };

    let raw_matches: Vec<&fret_core::SemanticsNode> = snapshot
        .nodes
        .iter()
        .filter(|node| {
            let node_id = node.id.data().as_ffi();
            in_scope(node_id)
                && matches_root_z(node_id)
                && node.test_id.as_deref() == Some(id.as_str())
        })
        .collect();
    if raw_matches.is_empty() {
        return None;
    }

    let hidden_count = raw_matches
        .iter()
        .filter(|node| {
            index
                .nearest_semantics_hidden_ancestor_or_self(node.id.data().as_ffi())
                .is_some()
        })
        .count();
    let outside_visible_root_count = raw_matches
        .iter()
        .filter(|node| !index.is_in_visible_root(node.id.data().as_ffi()))
        .count();
    let barrier_root = snapshot.barrier_root.map(|root| root.data().as_ffi());
    let barrier_root_z = barrier_root.map(|root| index.root_z_for(root));
    let below_barrier_count = barrier_root_z
        .map(|barrier_z| {
            raw_matches
                .iter()
                .filter(|node| index.root_z_for(node.id.data().as_ffi()) < barrier_z)
                .count()
        })
        .unwrap_or(0);

    Some(format!(
        "raw_match_count={} filtered_by_selectable hidden_count={} outside_visible_root_count={} below_barrier_count={} barrier_root={:?} barrier_root_z={:?}",
        raw_matches.len(),
        hidden_count,
        outside_visible_root_count,
        below_barrier_count,
        barrier_root,
        barrier_root_z,
    ))
}
