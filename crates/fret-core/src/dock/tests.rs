use super::*;
use slotmap::KeyData;

fn window(id: u64) -> AppWindowId {
    AppWindowId::from(KeyData::from_ffi(id))
}

fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
}

#[derive(Debug, Clone, Copy)]
struct DockForestStats {
    reachable_nodes: usize,
    max_split_depth: usize,
}

fn assert_canonical_window_forest(g: &DockGraph, window: AppWindowId) -> DockForestStats {
    use std::collections::HashSet;

    fn assert_canonical_subtree(
        g: &DockGraph,
        node: DockNodeId,
        split_depth: usize,
        reachable: &mut HashSet<DockNodeId>,
        max_split_depth: &mut usize,
    ) {
        if !reachable.insert(node) {
            panic!("dock graph has a cycle or duplicated edge: node={node:?}");
        }

        let Some(n) = g.node(node) else {
            panic!("dock graph references a missing node: node={node:?}");
        };

        match n {
            DockNode::Tabs { tabs, active } => {
                assert!(!tabs.is_empty(), "Tabs must be non-empty (node={node:?})");
                assert!(
                    *active < tabs.len(),
                    "Tabs.active out of bounds (node={node:?} active={active} len={})",
                    tabs.len()
                );
            }
            DockNode::Floating { child } => {
                assert_canonical_subtree(g, *child, split_depth, reachable, max_split_depth);
            }
            DockNode::Split {
                axis,
                children,
                fractions,
            } => {
                *max_split_depth = (*max_split_depth).max(split_depth.saturating_add(1));

                assert!(
                    children.len() >= 2,
                    "Split must have 2+ children (node={node:?} len={})",
                    children.len()
                );
                assert_eq!(
                    children.len(),
                    fractions.len(),
                    "Split children/fractions length mismatch (node={node:?})"
                );

                let mut sum = 0.0f32;
                for (i, f) in fractions.iter().enumerate() {
                    assert!(
                        f.is_finite(),
                        "Split fraction must be finite (node={node:?} i={i} f={f})"
                    );
                    assert!(
                        *f >= 0.0,
                        "Split fraction must be non-negative (node={node:?} i={i} f={f})"
                    );
                    sum += *f;
                }
                assert!(
                    (sum - 1.0).abs() <= 1e-3,
                    "Split fractions must be normalized (node={node:?} sum={sum})"
                );

                for child in children {
                    if let Some(DockNode::Split {
                        axis: child_axis, ..
                    }) = g.node(*child)
                    {
                        assert!(
                            *child_axis != *axis,
                            "Nested same-axis splits must be flattened (parent={node:?} axis={axis:?} child={child:?})"
                        );
                    }
                    assert_canonical_subtree(
                        g,
                        *child,
                        split_depth.saturating_add(1),
                        reachable,
                        max_split_depth,
                    );
                }
            }
        }
    }

    let mut reachable: HashSet<DockNodeId> = HashSet::new();
    let mut max_split_depth: usize = 0;

    if let Some(root) = g.window_root(window) {
        assert_canonical_subtree(g, root, 0, &mut reachable, &mut max_split_depth);
    }

    for w in g.floating_windows(window) {
        let floating = w.floating;
        let Some(DockNode::Floating { .. }) = g.node(floating) else {
            panic!("floating_windows entry must point to a Floating node: node={floating:?}");
        };
        assert_canonical_subtree(g, floating, 0, &mut reachable, &mut max_split_depth);
    }

    DockForestStats {
        reachable_nodes: reachable.len(),
        max_split_depth,
    }
}

fn assert_canonical_all_windows(g: &DockGraph) -> DockForestStats {
    use std::collections::HashSet;

    let windows = windows_including_floatings(g);
    let mut overall = DockForestStats {
        reachable_nodes: 0,
        max_split_depth: 0,
    };

    let mut seen_panels: HashSet<PanelKey> = HashSet::new();
    for w in windows {
        let stats = assert_canonical_window_forest(g, w);
        overall.reachable_nodes = overall.reachable_nodes.max(stats.reachable_nodes);
        overall.max_split_depth = overall.max_split_depth.max(stats.max_split_depth);

        for panel in g.collect_panels_in_window(w) {
            assert!(
                seen_panels.insert(panel.clone()),
                "panel appears multiple times across windows: {:?}",
                panel.kind.0
            );
            assert!(
                g.find_panel_in_window(w, &panel).is_some(),
                "panel lookup must succeed for collected panels: window={w:?} panel={:?}",
                panel.kind.0
            );
        }
    }

    overall
}

fn windows_including_floatings(g: &DockGraph) -> Vec<AppWindowId> {
    use std::collections::HashSet;

    let mut out: Vec<AppWindowId> = Vec::new();
    let mut seen: HashSet<AppWindowId> = HashSet::new();

    for w in g.window_roots.keys().copied() {
        if seen.insert(w) {
            out.push(w);
        }
    }
    for w in g.window_floatings.keys().copied() {
        if seen.insert(w) {
            out.push(w);
        }
    }

    out.sort_by_key(|w| w.data().as_ffi());
    out
}

#[test]
fn compute_layout_repairs_mismatched_fraction_lengths_without_truncating_children() {
    use std::collections::HashMap;

    let panel_a = PanelKey::new("test.a");
    let panel_b = PanelKey::new("test.b");
    let panel_c = PanelKey::new("test.c");

    let mut g = DockGraph::new();
    let tabs0 = g.insert_node(DockNode::Tabs {
        tabs: vec![panel_a],
        active: 0,
    });
    let tabs1 = g.insert_node(DockNode::Tabs {
        tabs: vec![panel_b],
        active: 0,
    });
    let tabs2 = g.insert_node(DockNode::Tabs {
        tabs: vec![panel_c],
        active: 0,
    });

    let split = g.insert_node(DockNode::Split {
        axis: Axis::Horizontal,
        children: vec![tabs0, tabs1, tabs2],
        // Non-canonical: missing one fraction entry. `compute_layout` should not silently truncate.
        fractions: vec![2.0, 1.0],
    });

    let mut out: HashMap<DockNodeId, Rect> = HashMap::new();
    g.compute_layout(split, rect(0.0, 0.0, 400.0, 100.0), &mut out);

    assert!(out.contains_key(&tabs0));
    assert!(out.contains_key(&tabs1));
    assert!(out.contains_key(&tabs2));

    assert_eq!(out.get(&tabs0).unwrap().size.width, Px(200.0));
    assert_eq!(out.get(&tabs1).unwrap().size.width, Px(100.0));
    assert_eq!(out.get(&tabs2).unwrap().size.width, Px(100.0));
}

#[test]
fn float_panel_in_window_creates_floating_container() {
    let w = window(1);
    let panel_a = PanelKey::new("test.a");
    let panel_b = PanelKey::new("test.b");

    let mut g = DockGraph::new();
    let tabs = g.insert_node(DockNode::Tabs {
        tabs: vec![panel_a.clone(), panel_b.clone()],
        active: 0,
    });
    g.set_window_root(w, tabs);

    let ok = g.apply_op(&DockOp::FloatPanelInWindow {
        source_window: w,
        panel: panel_b.clone(),
        target_window: w,
        rect: rect(10.0, 20.0, 300.0, 200.0),
    });
    assert!(ok);
    assert_eq!(g.collect_panels_in_window(w).len(), 2);
    assert!(
        g.floating_windows(w)
            .iter()
            .any(|f| { g.collect_panels_in_subtree(f.floating).contains(&panel_b) })
    );
    assert!(g.find_panel_in_window(w, &panel_b).is_some());
}

#[test]
fn merge_floating_into_moves_panels_and_removes_floating_entry() {
    let w = window(1);
    let panel_a = PanelKey::new("test.a");
    let panel_b = PanelKey::new("test.b");

    let mut g = DockGraph::new();
    let main_tabs = g.insert_node(DockNode::Tabs {
        tabs: vec![panel_a.clone(), panel_b.clone()],
        active: 0,
    });
    g.set_window_root(w, main_tabs);

    assert!(g.apply_op(&DockOp::FloatPanelInWindow {
        source_window: w,
        panel: panel_b.clone(),
        target_window: w,
        rect: rect(0.0, 0.0, 200.0, 160.0),
    }));

    let floating = g.floating_windows(w).first().unwrap().floating;
    assert!(g.apply_op(&DockOp::MergeFloatingInto {
        window: w,
        floating,
        target_tabs: main_tabs,
    }));

    assert!(g.floating_windows(w).is_empty());
    let (tabs, _i) = g
        .find_panel_in_window(w, &panel_b)
        .expect("panel in window");
    assert_eq!(tabs, main_tabs);
}

#[test]
fn merge_floating_into_rejects_target_inside_same_floating_subtree() {
    let w = window(1);
    let panel_a = PanelKey::new("test.a");
    let panel_b = PanelKey::new("test.b");

    let mut g = DockGraph::new();
    let main_tabs = g.insert_node(DockNode::Tabs {
        tabs: vec![panel_a.clone(), panel_b.clone()],
        active: 0,
    });
    g.set_window_root(w, main_tabs);

    assert!(g.apply_op(&DockOp::FloatPanelInWindow {
        source_window: w,
        panel: panel_b.clone(),
        target_window: w,
        rect: rect(0.0, 0.0, 200.0, 160.0),
    }));

    let floating = g.floating_windows(w).first().unwrap().floating;
    let DockNode::Floating {
        child: floating_tabs,
    } = g.node(floating).unwrap()
    else {
        unreachable!();
    };

    assert!(!g.apply_op(&DockOp::MergeFloatingInto {
        window: w,
        floating,
        target_tabs: *floating_tabs,
    }));
    assert_eq!(g.collect_panels_in_window(w).len(), 2);
    assert_eq!(g.floating_windows(w).len(), 1);
    assert!(g.find_panel_in_window(w, &panel_b).is_some());
}

#[test]
fn merge_window_into_rejects_non_tabs_target_to_avoid_panel_loss() {
    let w1 = window(1);
    let w2 = window(2);
    let panel_a = PanelKey::new("test.a");
    let panel_b = PanelKey::new("test.b");
    let panel_c = PanelKey::new("test.c");

    let mut g = DockGraph::new();

    let source_tabs = g.insert_node(DockNode::Tabs {
        tabs: vec![panel_a.clone(), panel_b.clone()],
        active: 0,
    });
    g.set_window_root(w1, source_tabs);

    let target_tabs = g.insert_node(DockNode::Tabs {
        tabs: vec![panel_c.clone()],
        active: 0,
    });
    let other = g.insert_node(DockNode::Tabs {
        tabs: vec![PanelKey::new("test.other")],
        active: 0,
    });
    let target_root = g.insert_node(DockNode::Split {
        axis: Axis::Horizontal,
        children: vec![target_tabs, other],
        fractions: vec![0.5, 0.5],
    });
    g.set_window_root(w2, target_root);

    assert!(!g.apply_op(&DockOp::MergeWindowInto {
        source_window: w1,
        target_window: w2,
        target_tabs: target_root,
    }));

    assert_eq!(g.collect_panels_in_window(w1).len(), 2);
    assert!(g.find_panel_in_window(w1, &panel_a).is_some());
    assert!(g.find_panel_in_window(w1, &panel_b).is_some());
    assert!(g.window_root(w1).is_some());
    assert_eq!(g.collect_panels_in_window(w2).len(), 2);
}

#[test]
fn float_tabs_in_window_creates_floating_container_with_tabs() {
    let w = window(1);
    let panel_a = PanelKey::new("test.a");
    let panel_b = PanelKey::new("test.b");
    let panel_c = PanelKey::new("test.c");

    let mut g = DockGraph::new();
    let tabs = g.insert_node(DockNode::Tabs {
        tabs: vec![panel_a.clone(), panel_b.clone(), panel_c.clone()],
        active: 1,
    });
    g.set_window_root(w, tabs);

    assert!(g.apply_op(&DockOp::FloatTabsInWindow {
        source_window: w,
        source_tabs: tabs,
        target_window: w,
        rect: rect(10.0, 20.0, 300.0, 200.0),
    }));

    assert!(
        g.window_root(w).is_none(),
        "expected dock root to be removed"
    );
    let floatings = g.floating_windows(w);
    assert_eq!(floatings.len(), 1);
    assert_eq!(floatings[0].rect, rect(10.0, 20.0, 300.0, 200.0));
    assert_eq!(
        g.collect_panels_in_subtree(floatings[0].floating),
        vec![panel_a, panel_b, panel_c]
    );
    assert!(
        g.find_panel_in_window(w, &PanelKey::new("test.b"))
            .is_some(),
        "expected panel lookup to work even when the window has no root"
    );
}

#[test]
fn move_tabs_merges_into_target_tabs_and_preserves_active() {
    let w = window(1);
    let panel_a = PanelKey::new("test.a");
    let panel_b = PanelKey::new("test.b");
    let panel_c = PanelKey::new("test.c");

    let mut g = DockGraph::new();
    let source_tabs = g.insert_node(DockNode::Tabs {
        tabs: vec![panel_a.clone(), panel_b.clone()],
        active: 1,
    });
    let target_tabs = g.insert_node(DockNode::Tabs {
        tabs: vec![panel_c.clone()],
        active: 0,
    });
    let root = g.insert_node(DockNode::Split {
        axis: Axis::Horizontal,
        children: vec![source_tabs, target_tabs],
        fractions: vec![0.5, 0.5],
    });
    g.set_window_root(w, root);

    assert!(g.apply_op(&DockOp::MoveTabs {
        source_window: w,
        source_tabs,
        target_window: w,
        target_tabs,
        zone: DropZone::Center,
        insert_index: Some(0),
    }));

    assert_eq!(g.window_root(w), Some(target_tabs));
    let DockNode::Tabs { tabs, active } = g.node(target_tabs).expect("target tabs exists") else {
        unreachable!();
    };
    assert_eq!(
        tabs,
        &vec![panel_a.clone(), panel_b.clone(), panel_c.clone()]
    );
    assert_eq!(*active, 1);
}

#[test]
fn layout_roundtrips_floatings_with_rect_and_order() {
    let w = window(1);
    let panel_a = PanelKey::new("test.a");
    let panel_b = PanelKey::new("test.b");
    let panel_c = PanelKey::new("test.c");

    let mut g = DockGraph::new();
    let main_tabs = g.insert_node(DockNode::Tabs {
        tabs: vec![panel_a.clone()],
        active: 0,
    });
    g.set_window_root(w, main_tabs);

    let f0_tabs = g.insert_node(DockNode::Tabs {
        tabs: vec![panel_b.clone()],
        active: 0,
    });
    let f0 = g.insert_node(DockNode::Floating { child: f0_tabs });
    g.floating_windows_mut(w).push(DockFloatingWindow {
        floating: f0,
        rect: rect(10.0, 20.0, 300.0, 200.0),
    });

    let f1_tabs = g.insert_node(DockNode::Tabs {
        tabs: vec![panel_c.clone()],
        active: 0,
    });
    let f1 = g.insert_node(DockNode::Floating { child: f1_tabs });
    g.floating_windows_mut(w).push(DockFloatingWindow {
        floating: f1,
        rect: rect(40.0, 60.0, 200.0, 120.0),
    });

    let windows = vec![(w, "main".to_string())];
    let layout = g.export_layout(&windows);

    let mut g2 = DockGraph::new();
    assert!(g2.import_layout_for_windows(&layout, &windows));
    assert_eq!(
        g2.collect_panels_in_window(w),
        vec![panel_a, panel_b, panel_c]
    );
    let floatings = g2.floating_windows(w);
    assert_eq!(floatings.len(), 2);
    assert_eq!(floatings[0].rect, rect(10.0, 20.0, 300.0, 200.0));
    assert_eq!(floatings[1].rect, rect(40.0, 60.0, 200.0, 120.0));
}

#[test]
fn dock_layout_json_roundtrips_and_validates() {
    let w = window(1);
    let panel_a = PanelKey::new("test.a");
    let panel_b = PanelKey::new("test.b");

    let mut g = DockGraph::new();
    let main_tabs = g.insert_node(DockNode::Tabs {
        tabs: vec![panel_a.clone()],
        active: 0,
    });
    g.set_window_root(w, main_tabs);

    let floating_tabs = g.insert_node(DockNode::Tabs {
        tabs: vec![panel_b.clone()],
        active: 0,
    });
    let floating = g.insert_node(DockNode::Floating {
        child: floating_tabs,
    });
    g.floating_windows_mut(w).push(DockFloatingWindow {
        floating,
        rect: rect(10.0, 20.0, 300.0, 200.0),
    });

    let windows = vec![(w, "main".to_string())];
    let layout = g.export_layout(&windows);

    let json = serde_json::to_string(&layout).expect("serialize DockLayout");
    let roundtripped: crate::DockLayout =
        serde_json::from_str(&json).expect("deserialize DockLayout");
    roundtripped.validate().expect("DockLayout validates");

    let mut g2 = DockGraph::new();
    assert!(g2.import_layout_for_windows(&roundtripped, &windows));
    assert_eq!(g2.collect_panels_in_window(w), vec![panel_a, panel_b]);
}

#[derive(Debug, Clone, serde::Deserialize)]
struct DockOpSequenceSuite {
    schema_version: u32,
    cases: Vec<DockOpSequenceCase>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct DockOpSequenceCase {
    id: String,
    #[serde(flatten)]
    spec: DockOpSequenceCaseSpec,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum DockOpSequenceInitialLayout {
    #[default]
    SplitNary,
    RootTabs,
}

impl DockOpSequenceInitialLayout {
    fn from_optional_string(raw: Option<&str>) -> Self {
        let Some(raw) = raw else {
            return Self::SplitNary;
        };
        match raw.trim().to_ascii_lowercase().as_str() {
            "" | "split_nary" | "split-nary" | "default" => Self::SplitNary,
            "root_tabs" | "root-tabs" => Self::RootTabs,
            _ => Self::SplitNary,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum DockOpSequenceCaseSpec {
    Explicit {
        #[serde(default)]
        initial: Option<String>,
        panels: Vec<String>,
        steps: Vec<DockOpSequenceStep>,
        expect: DockOpSequenceExpect,
    },
    Random {
        random: DockOpSequenceRandom,
        expect: DockOpSequenceExpect,
    },
}

#[derive(Debug, Clone, serde::Deserialize)]
struct DockOpSequenceRandom {
    seed: u64,
    steps: u32,
    panel_count: u32,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct DockOpSequenceExpect {
    max_reachable_nodes: usize,
    max_split_depth: usize,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum DockOpSequenceStep {
    FloatTabsInWindow {
        panel: String,
        rect: [f32; 4],
    },
    MovePanel {
        panel: String,
        target_panel: String,
        zone: String,
    },
}

#[test]
fn dock_op_sequence_fixtures_hold_canonical_invariants() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/dock/fixtures/dock_op_sequences_v1.json"
    ));
    let suite: DockOpSequenceSuite =
        serde_json::from_str(raw).expect("parse dock op sequence fixtures");
    assert_eq!(suite.schema_version, 1);

    for case in suite.cases {
        run_dock_op_sequence_case(&case);
    }
}

fn run_dock_op_sequence_case(case: &DockOpSequenceCase) {
    match &case.spec {
        DockOpSequenceCaseSpec::Explicit {
            initial,
            panels,
            steps,
            expect,
        } => {
            let initial = DockOpSequenceInitialLayout::from_optional_string(initial.as_deref());
            let (mut g, main_window, _panels) = make_initial_graph(panels, initial);
            assert_canonical_all_windows(&g);

            for (ix, step) in steps.iter().enumerate() {
                let ok = apply_fixture_step(&mut g, main_window, step);
                assert!(
                    ok,
                    "fixture step failed (case_id={} step_index={} step={:?})",
                    case.id, ix, step
                );
                let stats = assert_canonical_all_windows(&g);
                assert!(
                    stats.reachable_nodes <= expect.max_reachable_nodes,
                    "reachable_nodes exceeded bound (case_id={} step_index={} reachable_nodes={} max={})",
                    case.id,
                    ix,
                    stats.reachable_nodes,
                    expect.max_reachable_nodes
                );
                assert!(
                    stats.max_split_depth <= expect.max_split_depth,
                    "max_split_depth exceeded bound (case_id={} step_index={} max_split_depth={} max={})",
                    case.id,
                    ix,
                    stats.max_split_depth,
                    expect.max_split_depth
                );
            }
        }
        DockOpSequenceCaseSpec::Random { random, expect } => {
            let panels: Vec<String> = (0..random.panel_count)
                .map(|ix| format!("test.p{ix}"))
                .collect();
            let (mut g, _main_window, _panels) = make_initial_nary_split_graph(&panels);
            assert_canonical_all_windows(&g);

            let mut rng = XorShift64::new(random.seed);
            let mut successful_steps: u32 = 0;
            let mut attempts: u32 = 0;
            let max_attempts = random.steps.saturating_mul(25).max(1);
            let mut failed_ops: std::collections::BTreeMap<&'static str, u32> =
                std::collections::BTreeMap::new();
            let mut last_failed_op: Option<DockOp> = None;
            let mut last_ops: std::collections::VecDeque<DockOp> =
                std::collections::VecDeque::new();

            while successful_steps < random.steps && attempts < max_attempts {
                attempts = attempts.saturating_add(1);

                let total_panels_before: usize = windows_including_floatings(&g)
                    .iter()
                    .map(|w| g.collect_panels_in_window(*w).len())
                    .sum();
                assert!(
                    total_panels_before > 0,
                    "random case lost all panels unexpectedly (case_id={} successful_steps={} attempts={})",
                    case.id,
                    successful_steps,
                    attempts
                );

                let op = generate_random_op(&mut rng, &g);
                let ok = g.apply_op(&op);
                if last_ops.len() >= 32 {
                    let _ = last_ops.pop_front();
                }
                last_ops.push_back(op.clone());
                let stats = assert_canonical_all_windows(&g);
                assert!(
                    stats.reachable_nodes <= expect.max_reachable_nodes,
                    "reachable_nodes exceeded bound (case_id={} successful_steps={} reachable_nodes={} max={})",
                    case.id,
                    successful_steps,
                    stats.reachable_nodes,
                    expect.max_reachable_nodes
                );
                assert!(
                    stats.max_split_depth <= expect.max_split_depth,
                    "max_split_depth exceeded bound (case_id={} successful_steps={} max_split_depth={} max={})",
                    case.id,
                    successful_steps,
                    stats.max_split_depth,
                    expect.max_split_depth
                );

                let total_panels_after: usize = windows_including_floatings(&g)
                    .iter()
                    .map(|w| g.collect_panels_in_window(*w).len())
                    .sum();
                assert_eq!(
                    total_panels_after,
                    total_panels_before,
                    "random case unexpectedly changed panel count (case_id={} successful_steps={} attempts={} before={} after={} op={:?} last_ops={:?})",
                    case.id,
                    successful_steps,
                    attempts,
                    total_panels_before,
                    total_panels_after,
                    op,
                    last_ops,
                );
                if ok {
                    successful_steps = successful_steps.saturating_add(1);
                } else {
                    let kind: &'static str = match &op {
                        DockOp::SetActiveTab { .. } => "set_active_tab",
                        DockOp::ClosePanel { .. } => "close_panel",
                        DockOp::MovePanel { .. } => "move_panel",
                        DockOp::MoveTabs { .. } => "move_tabs",
                        DockOp::FloatPanelToWindow { .. } => "float_panel_to_window",
                        DockOp::RequestFloatPanelToNewWindow { .. } => {
                            "request_float_panel_to_new_window"
                        }
                        DockOp::FloatPanelInWindow { .. } => "float_panel_in_window",
                        DockOp::FloatTabsInWindow { .. } => "float_tabs_in_window",
                        DockOp::SetFloatingRect { .. } => "set_floating_rect",
                        DockOp::RaiseFloating { .. } => "raise_floating",
                        DockOp::MergeFloatingInto { .. } => "merge_floating_into",
                        DockOp::MergeWindowInto { .. } => "merge_window_into",
                        DockOp::SetSplitFractions { .. } => "set_split_fractions",
                        DockOp::SetSplitFractionsMany { .. } => "set_split_fractions_many",
                        DockOp::SetSplitFractionTwo { .. } => "set_split_fraction_two",
                    };
                    *failed_ops.entry(kind).or_default() += 1;
                    last_failed_op = Some(op);
                }
            }

            if successful_steps != random.steps {
                panic!(
                    "random case did not reach desired step count (case_id={} successful_steps={} desired_steps={} attempts={} max_attempts={} failed_ops={:?} last_failed_op={:?})",
                    case.id,
                    successful_steps,
                    random.steps,
                    attempts,
                    max_attempts,
                    failed_ops,
                    last_failed_op,
                );
            }
        }
    }
}

fn make_initial_nary_split_graph(panels: &[String]) -> (DockGraph, AppWindowId, Vec<PanelKey>) {
    let w = window(1);
    let panel_keys: Vec<PanelKey> = panels.iter().map(|s| PanelKey::new(s)).collect();

    let mut g = DockGraph::new();
    if panel_keys.is_empty() {
        return (g, w, panel_keys);
    }

    let children: Vec<DockNodeId> = panel_keys
        .iter()
        .map(|p| {
            g.insert_node(DockNode::Tabs {
                tabs: vec![p.clone()],
                active: 0,
            })
        })
        .collect();

    let fractions = vec![1.0 / children.len() as f32; children.len()];
    let root = g.insert_node(DockNode::Split {
        axis: Axis::Horizontal,
        children,
        fractions,
    });
    g.set_window_root(w, root);
    g.simplify_window_forest(w);

    (g, w, panel_keys)
}

fn make_initial_root_tabs_graph(panels: &[String]) -> (DockGraph, AppWindowId, Vec<PanelKey>) {
    let w = window(1);
    let panel_keys: Vec<PanelKey> = panels.iter().map(|s| PanelKey::new(s)).collect();

    let mut g = DockGraph::new();
    if panel_keys.is_empty() {
        return (g, w, panel_keys);
    }

    let root = g.insert_node(DockNode::Tabs {
        tabs: panel_keys.clone(),
        active: 0,
    });
    g.set_window_root(w, root);
    g.simplify_window_forest(w);

    (g, w, panel_keys)
}

fn make_initial_graph(
    panels: &[String],
    initial: DockOpSequenceInitialLayout,
) -> (DockGraph, AppWindowId, Vec<PanelKey>) {
    match initial {
        DockOpSequenceInitialLayout::SplitNary => make_initial_nary_split_graph(panels),
        DockOpSequenceInitialLayout::RootTabs => make_initial_root_tabs_graph(panels),
    }
}

fn apply_fixture_step(
    g: &mut DockGraph,
    _main_window: AppWindowId,
    step: &DockOpSequenceStep,
) -> bool {
    match step {
        DockOpSequenceStep::FloatTabsInWindow {
            panel,
            rect: rect_xywh,
        } => {
            let panel = PanelKey::new(panel);
            let windows = windows_including_floatings(g);
            let (source_window, (source_tabs, _)) =
                find_panel_any_window(g, &windows, &panel).expect("panel exists");

            g.apply_op(&DockOp::FloatTabsInWindow {
                source_window,
                source_tabs,
                target_window: source_window,
                rect: rect(rect_xywh[0], rect_xywh[1], rect_xywh[2], rect_xywh[3]),
            })
        }
        DockOpSequenceStep::MovePanel {
            panel,
            target_panel,
            zone,
        } => {
            let panel = PanelKey::new(panel);
            let target_panel = PanelKey::new(target_panel);
            let windows = windows_including_floatings(g);
            let (source_window, _) =
                find_panel_any_window(g, &windows, &panel).expect("panel exists");
            let (target_window, (target_tabs, _)) =
                find_panel_any_window(g, &windows, &target_panel).expect("target panel exists");

            let zone = parse_drop_zone(zone).unwrap_or(DropZone::Center);
            g.apply_op(&DockOp::MovePanel {
                source_window,
                panel,
                target_window,
                target_tabs,
                zone,
                insert_index: None,
            })
        }
    }
}

fn find_panel_any_window(
    g: &DockGraph,
    windows: &[AppWindowId],
    panel: &PanelKey,
) -> Option<(AppWindowId, (DockNodeId, usize))> {
    windows
        .iter()
        .find_map(|w| g.find_panel_in_window(*w, panel).map(|found| (*w, found)))
}

fn parse_drop_zone(raw: &str) -> Option<DropZone> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "left" => Some(DropZone::Left),
        "right" => Some(DropZone::Right),
        "top" => Some(DropZone::Top),
        "bottom" => Some(DropZone::Bottom),
        "center" => Some(DropZone::Center),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy)]
struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    fn new(seed: u64) -> Self {
        Self { state: seed.max(1) }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    fn next_f32(&mut self) -> f32 {
        let v = self.next_u64() >> 40;
        (v as f32) / ((1u64 << 24) as f32)
    }

    fn gen_range_usize(&mut self, upper: usize) -> usize {
        if upper <= 1 {
            return 0;
        }
        (self.next_u64() as usize) % upper
    }
}

fn generate_random_op(rng: &mut XorShift64, g: &DockGraph) -> DockOp {
    let windows = windows_including_floatings(g);
    if windows.is_empty() {
        return DockOp::SetSplitFractionsMany {
            updates: Vec::new(),
        };
    }

    let existing_panels: Vec<PanelKey> = windows
        .iter()
        .flat_map(|w| g.collect_panels_in_window(*w))
        .collect();
    if existing_panels.is_empty() {
        return DockOp::SetSplitFractionsMany {
            updates: Vec::new(),
        };
    }

    let pick_existing_panel = |rng: &mut XorShift64| -> PanelKey {
        existing_panels[rng.gen_range_usize(existing_panels.len())].clone()
    };

    fn collect_tabs_in_subtree(g: &DockGraph, node: DockNodeId, out: &mut Vec<DockNodeId>) {
        let Some(n) = g.node(node) else {
            return;
        };
        match n {
            DockNode::Tabs { .. } => out.push(node),
            DockNode::Split { children, .. } => {
                for child in children {
                    collect_tabs_in_subtree(g, *child, out);
                }
            }
            DockNode::Floating { child } => collect_tabs_in_subtree(g, *child, out),
        }
    }

    fn tabs_nodes_in_window_forest(g: &DockGraph, window: AppWindowId) -> Vec<DockNodeId> {
        let mut out: Vec<DockNodeId> = Vec::new();
        if let Some(root) = g.window_root(window) {
            collect_tabs_in_subtree(g, root, &mut out);
        }
        for f in g.floating_windows(window) {
            collect_tabs_in_subtree(g, f.floating, &mut out);
        }
        out.sort_by_key(|id| id.data().as_ffi());
        out.dedup();
        out
    }

    let action = rng.next_f32();
    if action < 0.60 {
        // MovePanel (mostly edge docks).
        let panel = pick_existing_panel(rng);
        let target_panel = pick_existing_panel(rng);

        let Some((source_window, _)) = find_panel_any_window(g, &windows, &panel) else {
            return DockOp::SetSplitFractionsMany {
                updates: Vec::new(),
            };
        };
        let Some((target_window, (target_tabs, _))) =
            find_panel_any_window(g, &windows, &target_panel)
        else {
            return DockOp::SetSplitFractionsMany {
                updates: Vec::new(),
            };
        };

        let zone = if action < 0.45 {
            if rng.next_f32() < 0.5 {
                DropZone::Left
            } else {
                DropZone::Right
            }
        } else if rng.next_f32() < 0.2 {
            if rng.next_f32() < 0.5 {
                DropZone::Top
            } else {
                DropZone::Bottom
            }
        } else {
            DropZone::Center
        };

        return DockOp::MovePanel {
            source_window,
            panel,
            target_window,
            target_tabs,
            zone,
            insert_index: None,
        };
    }

    if action < 0.78 {
        // MoveTabs (center only): higher-level operation, but keep it conservative for randomized
        // invariants gates.
        let panel = pick_existing_panel(rng);
        let target_panel = pick_existing_panel(rng);

        let Some((source_window, (source_tabs, _))) = find_panel_any_window(g, &windows, &panel)
        else {
            return DockOp::SetSplitFractionsMany {
                updates: Vec::new(),
            };
        };
        let Some((target_window, (target_tabs, _))) =
            find_panel_any_window(g, &windows, &target_panel)
        else {
            return DockOp::SetSplitFractionsMany {
                updates: Vec::new(),
            };
        };

        return DockOp::MoveTabs {
            source_window,
            source_tabs,
            target_window,
            target_tabs,
            zone: DropZone::Center,
            insert_index: None,
        };
    }

    if action < 0.88 {
        // FloatPanelInWindow (in-window floatings).
        let panel = pick_existing_panel(rng);
        let Some((source_window, _)) = find_panel_any_window(g, &windows, &panel) else {
            return DockOp::SetSplitFractionsMany {
                updates: Vec::new(),
            };
        };
        let x = (rng.next_f32() * 240.0).max(0.0);
        let y = (rng.next_f32() * 200.0).max(0.0);
        let w = 240.0 + rng.next_f32() * 280.0;
        let h = 160.0 + rng.next_f32() * 240.0;
        return DockOp::FloatPanelInWindow {
            source_window,
            panel,
            target_window: source_window,
            rect: rect(x, y, w, h),
        };
    }

    if action < 0.96 {
        // MergeFloatingInto / raise / move rect, if available.
        let win = windows[rng.gen_range_usize(windows.len())];
        let floatings = g.floating_windows(win);
        if floatings.is_empty() {
            return DockOp::SetSplitFractionsMany {
                updates: Vec::new(),
            };
        }
        let floating = floatings[rng.gen_range_usize(floatings.len())].floating;

        // Merge only when we can find a target tabs node outside the source floating subtree.
        if rng.next_f32() < 0.35 {
            let tabs = tabs_nodes_in_window_forest(g, win);
            let mut candidates: Vec<DockNodeId> = Vec::new();
            for t in tabs {
                if g.root_for_node_in_window_forest(win, t) != Some(floating) {
                    candidates.push(t);
                }
            }
            if !candidates.is_empty() {
                let target_tabs = candidates[rng.gen_range_usize(candidates.len())];
                return DockOp::MergeFloatingInto {
                    window: win,
                    floating,
                    target_tabs,
                };
            }
        }

        if rng.next_f32() < 0.5 {
            let x = rng.next_f32() * 300.0;
            let y = rng.next_f32() * 200.0;
            let width = 260.0 + rng.next_f32() * 320.0;
            let h = 180.0 + rng.next_f32() * 260.0;
            return DockOp::SetFloatingRect {
                window: win,
                floating,
                rect: rect(x, y, width, h),
            };
        }
        return DockOp::RaiseFloating {
            window: win,
            floating,
        };
    }

    // Fallback: a center move (should always succeed for an existing panel).
    let panel = pick_existing_panel(rng);
    let target_panel = pick_existing_panel(rng);
    let Some((source_window, _)) = find_panel_any_window(g, &windows, &panel) else {
        return DockOp::SetSplitFractionsMany {
            updates: Vec::new(),
        };
    };
    let Some((target_window, (target_tabs, _))) = find_panel_any_window(g, &windows, &target_panel)
    else {
        return DockOp::SetSplitFractionsMany {
            updates: Vec::new(),
        };
    };
    DockOp::MovePanel {
        source_window,
        panel,
        target_window,
        target_tabs,
        zone: DropZone::Center,
        insert_index: None,
    }
}

#[test]
fn import_layout_simplifies_nested_same_axis_splits() {
    let w = window(1);

    let layout = crate::DockLayout::new(
        vec![crate::DockLayoutWindow {
            logical_window_id: "main".to_string(),
            root: 5,
            placement: None,
            floatings: Vec::new(),
        }],
        vec![
            crate::DockLayoutNode::Tabs {
                id: 1,
                tabs: vec![PanelKey::new("test.a")],
                active: 0,
            },
            crate::DockLayoutNode::Tabs {
                id: 2,
                tabs: vec![PanelKey::new("test.b")],
                active: 0,
            },
            crate::DockLayoutNode::Tabs {
                id: 3,
                tabs: vec![PanelKey::new("test.c")],
                active: 0,
            },
            crate::DockLayoutNode::Split {
                id: 4,
                axis: Axis::Horizontal,
                children: vec![1, 2],
                fractions: vec![0.5, 0.5],
            },
            crate::DockLayoutNode::Split {
                id: 5,
                axis: Axis::Horizontal,
                children: vec![4, 3],
                fractions: vec![0.5, 0.5],
            },
        ],
    );

    let mut g = DockGraph::new();
    assert!(g.import_layout_for_windows(&layout, &[(w, "main".to_string())]));

    let root = g.window_root(w).expect("expected window root");
    let Some(DockNode::Split {
        axis,
        children,
        fractions,
    }) = g.node(root)
    else {
        panic!("expected imported window root to be a split");
    };
    assert_eq!(*axis, Axis::Horizontal);
    assert_eq!(children.len(), 3);
    assert_eq!(children.len(), fractions.len());

    let panels: Vec<PanelKey> = children
        .iter()
        .filter_map(|&id| match g.node(id) {
            Some(DockNode::Tabs { tabs, .. }) => tabs.first().cloned(),
            _ => None,
        })
        .collect();
    assert_eq!(
        panels,
        vec![
            PanelKey::new("test.a"),
            PanelKey::new("test.b"),
            PanelKey::new("test.c"),
        ],
        "expected same-axis nested splits flattened on import"
    );

    let expected = [0.25, 0.25, 0.5];
    for (got, exp) in fractions.iter().copied().zip(expected) {
        assert!(
            (got - exp).abs() <= 1.0e-6,
            "expected {expected:?}, got {fractions:?}"
        );
    }
}

#[test]
fn import_layout_degrades_unmapped_windows_into_floating_containers() {
    let window_a = window(1);
    let panel_a = PanelKey::new("test.a");
    let panel_b = PanelKey::new("test.b");

    let layout = crate::DockLayout::new(
        vec![
            crate::DockLayoutWindow {
                logical_window_id: "main".to_string(),
                root: 1,
                placement: None,
                floatings: Vec::new(),
            },
            crate::DockLayoutWindow {
                logical_window_id: "extra".to_string(),
                root: 2,
                placement: Some(crate::DockWindowPlacement {
                    width: 400,
                    height: 300,
                    x: None,
                    y: None,
                    monitor_hint: None,
                }),
                floatings: Vec::new(),
            },
        ],
        vec![
            crate::DockLayoutNode::Tabs {
                id: 1,
                tabs: vec![panel_a.clone()],
                active: 0,
            },
            crate::DockLayoutNode::Tabs {
                id: 2,
                tabs: vec![panel_b.clone()],
                active: 0,
            },
        ],
    );

    let mut g = DockGraph::new();
    assert!(g.import_layout_for_windows_with_fallback_floatings(
        &layout,
        &[(window_a, "main".to_string())],
        window_a
    ));

    assert!(g.find_panel_in_window(window_a, &panel_a).is_some());
    assert!(g.find_panel_in_window(window_a, &panel_b).is_some());
    assert_eq!(g.floating_windows(window_a).len(), 1);
}

#[test]
fn close_panel_before_active_preserves_active_panel() {
    let w = window(1);
    let panel_a = PanelKey::new("test.a");
    let panel_b = PanelKey::new("test.b");
    let panel_c = PanelKey::new("test.c");

    let mut g = DockGraph::new();
    let tabs = g.insert_node(DockNode::Tabs {
        tabs: vec![panel_a.clone(), panel_b.clone(), panel_c.clone()],
        active: 1,
    });
    g.set_window_root(w, tabs);

    assert!(g.apply_op(&DockOp::ClosePanel {
        window: w,
        panel: panel_a.clone(),
    }));

    let DockNode::Tabs { tabs: list, active } = g.node(tabs).expect("tabs node must exist") else {
        unreachable!();
    };

    assert_eq!(list, &vec![panel_b.clone(), panel_c.clone()]);
    assert_eq!(*active, 0, "expected active panel (b) to remain active");
    assert_eq!(
        g.find_panel_in_window(w, &panel_b),
        Some((tabs, 0)),
        "expected the previously-active panel to remain selected"
    );
}

#[test]
fn close_panel_prunes_empty_tabs_in_nary_split() {
    let w = window(1);
    let panel_a = PanelKey::new("test.a");
    let panel_b = PanelKey::new("test.b");
    let panel_c = PanelKey::new("test.c");

    let mut g = DockGraph::new();
    let tabs_a = g.insert_node(DockNode::Tabs {
        tabs: vec![panel_a.clone()],
        active: 0,
    });
    let tabs_b = g.insert_node(DockNode::Tabs {
        tabs: vec![panel_b.clone()],
        active: 0,
    });
    let tabs_c = g.insert_node(DockNode::Tabs {
        tabs: vec![panel_c.clone()],
        active: 0,
    });

    let root = g.insert_node(DockNode::Split {
        axis: Axis::Horizontal,
        children: vec![tabs_a, tabs_b, tabs_c],
        fractions: vec![0.2, 0.3, 0.5],
    });
    g.set_window_root(w, root);

    assert!(g.apply_op(&DockOp::ClosePanel {
        window: w,
        panel: panel_b.clone(),
    }));

    let root = g.window_root(w).expect("window root exists");
    let DockNode::Split {
        children,
        fractions,
        ..
    } = g.node(root).expect("split root")
    else {
        panic!("expected split root after pruning");
    };

    assert_eq!(children.len(), 2, "expected empty tabs node to be pruned");
    assert_eq!(fractions.len(), 2, "expected fractions to stay aligned");

    let sum: f32 = fractions.iter().copied().sum();
    assert!((sum - 1.0).abs() < 1e-4, "expected normalized fractions");

    assert!(g.find_panel_in_window(w, &panel_a).is_some());
    assert!(g.find_panel_in_window(w, &panel_c).is_some());
    assert!(g.find_panel_in_window(w, &panel_b).is_none());
}

#[test]
fn edge_dock_inserts_into_existing_same_axis_split_and_splits_share() {
    let w = window(1);
    let panel_a = PanelKey::new("test.a");
    let panel_b = PanelKey::new("test.b");
    let panel_c = PanelKey::new("test.c");
    let panel_d = PanelKey::new("test.d");

    let mut g = DockGraph::new();
    let tabs_left = g.insert_node(DockNode::Tabs {
        tabs: vec![panel_a.clone()],
        active: 0,
    });
    let tabs_right = g.insert_node(DockNode::Tabs {
        tabs: vec![panel_b.clone(), panel_c.clone(), panel_d.clone()],
        active: 0,
    });
    let root = g.insert_node(DockNode::Split {
        axis: Axis::Horizontal,
        children: vec![tabs_left, tabs_right],
        fractions: vec![0.2, 0.8],
    });
    g.set_window_root(w, root);

    assert!(g.apply_op(&DockOp::MovePanel {
        source_window: w,
        panel: panel_c.clone(),
        target_window: w,
        target_tabs: tabs_right,
        zone: DropZone::Left,
        insert_index: None,
    }));

    let root = g.window_root(w).expect("window root exists");
    let DockNode::Split {
        children,
        fractions,
        ..
    } = g.node(root).expect("root must exist")
    else {
        unreachable!();
    };

    assert_eq!(children.len(), 3);
    assert_eq!(fractions.len(), 3);
    assert!((fractions[0] - 0.2).abs() < 1e-4);
    assert!((fractions[1] - 0.4).abs() < 1e-4);
    assert!((fractions[2] - 0.4).abs() < 1e-4);

    assert!(g.find_panel_in_window(w, &panel_c).is_some());
    assert!(g.find_panel_in_window(w, &panel_a).is_some());
    assert!(g.find_panel_in_window(w, &panel_b).is_some());
    assert!(g.find_panel_in_window(w, &panel_d).is_some());
}

#[test]
fn repeated_edge_dock_keeps_same_axis_splits_flat() {
    let w = window(1);
    let panel_a = PanelKey::new("test.a");
    let panel_b = PanelKey::new("test.b");
    let panel_c = PanelKey::new("test.c");
    let panel_d = PanelKey::new("test.d");

    let mut g = DockGraph::new();
    let tabs = g.insert_node(DockNode::Tabs {
        tabs: vec![
            panel_a.clone(),
            panel_b.clone(),
            panel_c.clone(),
            panel_d.clone(),
        ],
        active: 0,
    });
    g.set_window_root(w, tabs);

    for panel in [panel_b.clone(), panel_c.clone(), panel_d.clone()] {
        assert!(g.apply_op(&DockOp::MovePanel {
            source_window: w,
            panel,
            target_window: w,
            target_tabs: tabs,
            zone: DropZone::Left,
            insert_index: None,
        }));
    }

    fn max_same_axis_depth(graph: &DockGraph, node: DockNodeId, axis: Axis) -> usize {
        let Some(n) = graph.node(node) else {
            return 0;
        };
        match n {
            DockNode::Tabs { .. } => 0,
            DockNode::Floating { child } => max_same_axis_depth(graph, *child, axis),
            DockNode::Split {
                axis: split_axis,
                children,
                ..
            } => {
                let child_max = children
                    .iter()
                    .copied()
                    .map(|c| max_same_axis_depth(graph, c, axis))
                    .max()
                    .unwrap_or(0);
                if *split_axis == axis {
                    child_max + 1
                } else {
                    child_max
                }
            }
        }
    }

    let root = g.window_root(w).expect("window root exists");
    let depth = max_same_axis_depth(&g, root, Axis::Horizontal);
    assert_eq!(depth, 1, "expected no nested same-axis splits");
}
