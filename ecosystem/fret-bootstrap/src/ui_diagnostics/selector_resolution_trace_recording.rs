fn select_semantics_node_with_trace<'a>(
    snapshot: &'a fret_core::SemanticsSnapshot,
    window: AppWindowId,
    element_runtime: Option<&ElementRuntime>,
    selector: &UiSelectorV1,
    step_index: u32,
    redact_text: bool,
    trace: &mut Vec<UiSelectorResolutionTraceEntryV1>,
) -> Option<&'a fret_core::SemanticsNode> {
    let index = SemanticsIndex::new(snapshot);
    let mut matches: Vec<&'a fret_core::SemanticsNode> = Vec::new();
    let mut note: Option<String> = None;

    match selector {
        UiSelectorV1::NodeId { node } => {
            if let Some(n) = index
                .by_id
                .get(node)
                .copied()
                .filter(|n| index.is_selectable(n.id.data().as_ffi()))
            {
                matches.push(n);
            }
        }
        UiSelectorV1::RoleAndName { role, name } => {
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
                index.is_selectable(id) && n.role == role && n.label.as_deref() == Some(name)
            }));
        }
        UiSelectorV1::RoleAndPath {
            role,
            name,
            ancestors,
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
                    && n.role == role
                    && n.label.as_deref() == Some(name)
                    && index.ancestors_match_subsequence(n.parent, &parsed_ancestors)
            }));
        }
        UiSelectorV1::TestId { id } => {
            matches.extend(snapshot.nodes.iter().filter(|n| {
                let node_id = n.id.data().as_ffi();
                index.is_selectable(node_id) && n.test_id.as_deref() == Some(id)
            }));
            if matches.is_empty() {
                // Fallback for debugging: allow selecting hidden nodes if no visible match exists.
                note = Some("fallback_hidden_nodes".to_string());
                matches.extend(
                    snapshot
                        .nodes
                        .iter()
                        .filter(|n| n.test_id.as_deref() == Some(id)),
                );
            }
        }
        UiSelectorV1::GlobalElementId { element } => {
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
            if let Some(n) = index
                .by_id
                .get(&node_id)
                .copied()
                .filter(|n| index.is_selectable(n.id.data().as_ffi()))
            {
                matches.push(n);
            }
        }
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
