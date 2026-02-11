use super::*;
use slotmap::KeyData;

fn window(id: u64) -> AppWindowId {
    AppWindowId::from(KeyData::from_ffi(id))
}

fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
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
