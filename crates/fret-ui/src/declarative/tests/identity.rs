use super::*;
use crate::GlobalElementId;
use fret_runtime::ModelId;

const CX_RS_SOURCE: &str = include_str!("../../elements/cx.rs");

#[track_caller]
fn bump_root_scoped_counter<H: crate::UiHost>(cx: &mut crate::ElementContext<'_, H>) -> u32 {
    cx.root_state(u32::default, |value| {
        *value = value.saturating_add(1);
        *value
    })
}

#[track_caller]
fn bump_callsite_scoped_counter<H: crate::UiHost>(cx: &mut crate::ElementContext<'_, H>) -> u32 {
    cx.slot_state(u32::default, |value| {
        *value = value.saturating_add(1);
        *value
    })
}

fn two_root_scoped_counters<H: crate::UiHost>(cx: &mut crate::ElementContext<'_, H>) -> (u32, u32) {
    let a = bump_root_scoped_counter(cx);
    let b = bump_root_scoped_counter(cx);
    (a, b)
}

fn two_callsite_scoped_counters<H: crate::UiHost>(
    cx: &mut crate::ElementContext<'_, H>,
) -> (u32, u32) {
    let a = bump_callsite_scoped_counter(cx);
    let b = bump_callsite_scoped_counter(cx);
    (a, b)
}

#[track_caller]
fn remember_keyed_child_helper_slot<H: crate::UiHost>(
    cx: &mut crate::ElementContext<'_, H>,
    item: u64,
) -> u64 {
    cx.slot_state(|| item, |remembered| *remembered)
}

#[track_caller]
fn keyed_child_helper_local_model_id<H: crate::UiHost>(
    cx: &mut crate::ElementContext<'_, H>,
    item: u64,
) -> ModelId {
    cx.local_model(|| item).id()
}

#[cfg(debug_assertions)]
#[track_caller]
fn repeated_call_diagnostics_pair<H: crate::UiHost>(
    cx: &mut crate::ElementContext<'_, H>,
) -> (bool, bool) {
    let loc = std::panic::Location::caller();
    (
        cx.note_repeated_call_in_render_evaluation_at(loc),
        cx.note_repeated_call_in_render_evaluation_at(loc),
    )
}

#[cfg(debug_assertions)]
#[track_caller]
fn repeated_call_diagnostics_triple<H: crate::UiHost>(
    cx: &mut crate::ElementContext<'_, H>,
) -> (bool, bool, bool) {
    let loc = std::panic::Location::caller();
    (
        cx.note_repeated_call_in_render_evaluation_at(loc),
        cx.note_repeated_call_in_render_evaluation_at(loc),
        cx.note_repeated_call_in_render_evaluation_at(loc),
    )
}

#[test]
fn named_scopes_produce_stable_element_ids_across_frames() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let leaf_ids = Arc::new(std::sync::Mutex::new(
        Vec::<crate::elements::GlobalElementId>::new(),
    ));

    let mut root: Option<NodeId> = None;
    for frame in 0..2 {
        let leaf_ids = leaf_ids.clone();
        let root_node = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "named-scopes-stable",
            move |cx| {
                let leaf = cx.named("sidebar", |cx| cx.text("leaf"));
                leaf_ids.lock().unwrap().push(leaf.id);
                vec![leaf]
            },
        );

        root.get_or_insert(root_node);
        if frame == 0 {
            ui.set_root(root_node);
        }
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        app.advance_frame();
    }

    let ids = leaf_ids.lock().unwrap();
    assert_eq!(ids.len(), 2);
    assert_eq!(ids[0], ids[1]);
}

#[test]
fn scope_only_authoring_identity_is_live_for_current_frame() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let mut owner_identity: Option<GlobalElementId> = None;
    let mut show_owner = true;

    for frame in 0..2 {
        let root_node = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "scope-only-authoring-identity-liveness",
            |cx| {
                let mut out = Vec::new();
                if show_owner {
                    out.push(cx.keyed("owner", |cx| {
                        owner_identity = Some(cx.root_id());
                        cx.text("owner")
                    }));
                } else {
                    out.push(cx.text("other"));
                }
                out
            },
        );

        ui.set_root(root_node);
        let owner_identity = owner_identity.expect("owner identity");
        assert_eq!(
            crate::elements::live_node_for_element(&mut app, window, owner_identity),
            None,
            "scope-only owner should not require a mounted node"
        );
        assert_eq!(
            crate::elements::element_identity_is_live_in_current_frame(
                &mut app,
                window,
                owner_identity,
            ),
            frame == 0,
            "scope-only authoring identity liveness should follow render participation"
        );

        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        app.advance_frame();
        show_owner = false;
    }
}

#[test]
fn keyed_list_reorder_preserves_element_identity_for_state() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let results = Arc::new(std::sync::Mutex::new(Vec::<Vec<(u64, u64)>>::new()));
    let orders: [Vec<u64>; 2] = [vec![1, 2, 3], vec![3, 2, 1]];

    let mut root: Option<NodeId> = None;
    for (frame, items) in orders.into_iter().enumerate() {
        let results = results.clone();
        let root_node = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "keyed-reorder-identity",
            move |cx| {
                let mut elements = Vec::new();
                let mut frame_results = Vec::new();
                for item in items {
                    let el = cx.keyed(item, |cx| cx.text("row"));
                    let remembered = cx.state_for(el.id, || item, |st: &mut u64| *st);
                    frame_results.push((item, remembered));
                    elements.push(el);
                }
                results.lock().unwrap().push(frame_results);
                elements
            },
        );

        root.get_or_insert(root_node);
        if frame == 0 {
            ui.set_root(root_node);
        }
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        app.advance_frame();
    }

    let results = results.lock().unwrap();
    assert_eq!(results.len(), 2);
    for &(item, remembered) in &results[1] {
        assert_eq!(
            item, remembered,
            "expected keyed items to preserve element identity across reorder"
        );
    }
}

#[test]
fn unkeyed_list_reorder_does_not_preserve_element_identity_for_state() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let results = Arc::new(std::sync::Mutex::new(Vec::<Vec<(u64, u64)>>::new()));
    let orders: [Vec<u64>; 2] = [vec![1, 2, 3], vec![3, 2, 1]];

    let mut root: Option<NodeId> = None;
    for (frame, items) in orders.into_iter().enumerate() {
        let results = results.clone();
        let root_node = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "unkeyed-reorder-identity",
            move |cx| {
                let mut elements = Vec::new();
                let mut frame_results = Vec::new();
                cx.for_each_unkeyed(&items, |cx, _idx, item| {
                    let el = cx.text("row");
                    let remembered = cx.state_for(el.id, || *item, |st: &mut u64| *st);
                    frame_results.push((*item, remembered));
                    elements.push(el);
                });
                results.lock().unwrap().push(frame_results);
                elements
            },
        );

        root.get_or_insert(root_node);
        if frame == 0 {
            ui.set_root(root_node);
        }
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        app.advance_frame();
    }

    let results = results.lock().unwrap();
    assert_eq!(results.len(), 2);

    let reordered = &results[1];
    assert!(
        reordered
            .iter()
            .any(|(item, remembered)| item != remembered),
        "expected unkeyed reorder to reuse element identity by index, not item key"
    );
}

#[test]
#[cfg(feature = "diagnostics")]
fn identity_diagnostics_record_unkeyed_list_reorder() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let orders: [Vec<u64>; 2] = [vec![1, 2, 3], vec![3, 2, 1]];
    let mut root: Option<NodeId> = None;

    for (frame, items) in orders.into_iter().enumerate() {
        let root_node = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "unkeyed-reorder-identity-diagnostics",
            move |cx| {
                let mut elements = Vec::new();
                cx.for_each_unkeyed(&items, |cx, _idx, _item| {
                    elements.push(cx.text("row"));
                });
                elements
            },
        );

        root.get_or_insert(root_node);
        if frame == 0 {
            ui.set_root(root_node);
        }
        app.advance_frame();
    }

    let warnings = app.with_global_mut(crate::elements::ElementRuntime::new, |runtime, _| {
        runtime
            .diagnostics_snapshot(window)
            .expect("diagnostics snapshot")
            .identity_warnings
    });

    let Some(record) = warnings.iter().find_map(|record| match record {
        crate::elements::IdentityDiagnosticsRecord::UnkeyedListOrderChanged {
            previous_len,
            next_len,
            file,
            ..
        } => Some((*previous_len, *next_len, *file)),
        _ => None,
    }) else {
        panic!("expected unkeyed reorder identity warning, got {warnings:#?}");
    };

    assert_eq!(record.0, 3);
    assert_eq!(record.1, 3);
    assert!(
        record.2.ends_with("identity.rs"),
        "expected source location to point at this test, got {}",
        record.2
    );
}

#[test]
#[cfg(feature = "diagnostics")]
fn identity_diagnostics_record_duplicate_keyed_list_item_hash() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();
    let items = vec![10_u64, 20, 30];

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "duplicate-keyed-list-identity-diagnostics",
        move |cx| {
            let mut elements = Vec::new();
            cx.for_each_keyed(
                &items,
                |_item| 7_u64,
                |cx, _idx, _item| {
                    elements.push(cx.text("row"));
                },
            );
            elements
        },
    );
    ui.set_root(root);

    let warnings = app.with_global_mut(crate::elements::ElementRuntime::new, |runtime, _| {
        runtime
            .diagnostics_snapshot(window)
            .expect("diagnostics snapshot")
            .identity_warnings
    });

    let Some(record) = warnings.iter().find_map(|record| match record {
        crate::elements::IdentityDiagnosticsRecord::DuplicateKeyedListItemKeyHash {
            key_hash,
            first_index,
            second_index,
            file,
            ..
        } => Some((*key_hash, *first_index, *second_index, *file)),
        _ => None,
    }) else {
        panic!("expected duplicate keyed-list identity warning, got {warnings:#?}");
    };

    assert_ne!(record.0, 0);
    assert_eq!(record.1, 0);
    assert_eq!(record.2, 1);
    assert!(
        record.3.ends_with("identity.rs"),
        "expected source location to point at this test, got {}",
        record.3
    );
}

#[test]
fn root_state_is_root_scoped_shared_slot_per_type() {
    let mut app = TestHost::new();
    let window = AppWindowId::default();
    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );

    let first = crate::elements::with_element_cx(&mut app, window, bounds, "root-state", |cx| {
        two_root_scoped_counters(cx)
    });
    let second = crate::elements::with_element_cx(&mut app, window, bounds, "root-state", |cx| {
        two_root_scoped_counters(cx)
    });

    assert_eq!(first, (1, 2));
    assert_eq!(second, (3, 4));
}

#[test]
fn slot_state_is_independent_per_callsite() {
    let mut app = TestHost::new();
    let window = AppWindowId::default();
    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );

    let first = crate::elements::with_element_cx(&mut app, window, bounds, "slot-state", |cx| {
        two_callsite_scoped_counters(cx)
    });
    let second = crate::elements::with_element_cx(&mut app, window, bounds, "slot-state", |cx| {
        two_callsite_scoped_counters(cx)
    });

    assert_eq!(first, (1, 1));
    assert_eq!(second, (2, 2));
}

#[test]
fn element_tree_duplicate_ids_detect_duplicates_in_one_frame() {
    use crate::element::{ContainerProps, ElementKind};
    use crate::elements::GlobalElementId;

    let a = AnyElement::new(
        GlobalElementId(1),
        ElementKind::Container(ContainerProps::default()),
        Vec::new(),
    );
    let b = AnyElement::new(
        GlobalElementId(1),
        ElementKind::Container(ContainerProps::default()),
        Vec::new(),
    );

    let duplicates = super::super::mount::element_tree_duplicate_ids(&[a, b]);
    assert_eq!(duplicates, vec![GlobalElementId(1)]);
}

#[test]
fn local_model_assigns_distinct_callsite_slots_and_preserves_ids_across_frames() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let observed = Arc::new(std::sync::Mutex::new(Vec::<(ModelId, ModelId)>::new()));
    let mut root: Option<NodeId> = None;

    for frame in 0..2 {
        let observed = observed.clone();
        let root_node = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "local-model-callsite-stability",
            move |cx| {
                let first = cx.local_model(|| 1u32);
                let second = cx.local_model(|| 2u32);
                observed.lock().unwrap().push((first.id(), second.id()));
                vec![cx.text("leaf")]
            },
        );

        root.get_or_insert(root_node);
        if frame == 0 {
            ui.set_root(root_node);
        }
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        app.advance_frame();
    }

    let observed = observed.lock().unwrap();
    assert_eq!(observed.len(), 2);
    assert_ne!(
        observed[0].0, observed[0].1,
        "distinct local_model callsites should not share a slot"
    );
    assert_eq!(
        observed[0], observed[1],
        "expected local_model callsite slots to preserve model identity across frames"
    );
}

#[test]
fn local_model_keyed_preserves_model_identity_across_reorder() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let observed = Arc::new(std::sync::Mutex::new(Vec::<Vec<(u64, ModelId)>>::new()));
    let orders: [Vec<u64>; 2] = [vec![1, 2, 3], vec![3, 2, 1]];

    let mut root: Option<NodeId> = None;
    for (frame, items) in orders.into_iter().enumerate() {
        let observed = observed.clone();
        let root_node = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "local-model-keyed-reorder",
            move |cx| {
                let mut frame_results = Vec::new();
                let mut elements = Vec::new();
                for item in items {
                    let model = cx.local_model_keyed(item, || item);
                    frame_results.push((item, model.id()));
                    elements.push(cx.keyed(item, |cx| cx.text("row")));
                }
                observed.lock().unwrap().push(frame_results);
                elements
            },
        );

        root.get_or_insert(root_node);
        if frame == 0 {
            ui.set_root(root_node);
        }
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        app.advance_frame();
    }

    let observed = observed.lock().unwrap();
    assert_eq!(observed.len(), 2);
    let first: std::collections::HashMap<u64, ModelId> = observed[0].iter().copied().collect();
    let second: std::collections::HashMap<u64, ModelId> = observed[1].iter().copied().collect();
    assert_eq!(
        first, second,
        "expected local_model_keyed to preserve model identity by key across reorder"
    );
}

#[test]
fn slot_state_in_keyed_child_scope_preserves_state_across_reorder() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let observed = Arc::new(std::sync::Mutex::new(Vec::<Vec<(u64, u64)>>::new()));
    let orders: [Vec<u64>; 2] = [vec![1, 2, 3], vec![3, 2, 1]];

    let mut root: Option<NodeId> = None;
    for (frame, items) in orders.into_iter().enumerate() {
        let observed = observed.clone();
        let root_node = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "slot-state-keyed-child-scope",
            move |cx| {
                let mut elements = Vec::new();
                let mut frame_results = Vec::new();
                for item in items {
                    let el = cx.keyed(item, |cx| {
                        let remembered = remember_keyed_child_helper_slot(cx, item);
                        frame_results.push((item, remembered));
                        cx.text("row")
                    });
                    elements.push(el);
                }
                observed.lock().unwrap().push(frame_results);
                elements
            },
        );

        root.get_or_insert(root_node);
        if frame == 0 {
            ui.set_root(root_node);
        }
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        app.advance_frame();
    }

    let observed = observed.lock().unwrap();
    assert_eq!(observed.len(), 2);
    for &(item, remembered) in &observed[1] {
        assert_eq!(
            item, remembered,
            "expected keyed child scope to preserve helper slot_state identity across reorder"
        );
    }
}

#[cfg(debug_assertions)]
#[test]
fn repeated_call_diagnostics_reset_between_render_evaluations() {
    let mut app = TestHost::new();
    let window = AppWindowId::default();
    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );

    let first = crate::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "repeated-call-diagnostics",
        repeated_call_diagnostics_pair,
    );
    let second = crate::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "repeated-call-diagnostics",
        repeated_call_diagnostics_pair,
    );

    assert_eq!(first, (false, true));
    assert_eq!(second, (false, true));
}

#[cfg(debug_assertions)]
#[test]
fn repeated_call_diagnostics_only_warn_on_second_call_in_one_evaluation() {
    let mut app = TestHost::new();
    let window = AppWindowId::default();
    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );

    let result = crate::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "repeated-call-diagnostics-second-call",
        repeated_call_diagnostics_triple,
    );

    assert_eq!(result, (false, true, false));
}

#[test]
fn local_model_in_keyed_child_scope_preserves_model_identity_across_reorder() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let observed = Arc::new(std::sync::Mutex::new(Vec::<Vec<(u64, ModelId)>>::new()));
    let orders: [Vec<u64>; 2] = [vec![1, 2, 3], vec![3, 2, 1]];

    let mut root: Option<NodeId> = None;
    for (frame, items) in orders.into_iter().enumerate() {
        let observed = observed.clone();
        let root_node = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "local-model-keyed-child-scope",
            move |cx| {
                let mut elements = Vec::new();
                let mut frame_results = Vec::new();
                for item in items {
                    let el = cx.keyed(item, |cx| {
                        let model_id = keyed_child_helper_local_model_id(cx, item);
                        frame_results.push((item, model_id));
                        cx.text("row")
                    });
                    elements.push(el);
                }
                observed.lock().unwrap().push(frame_results);
                elements
            },
        );

        root.get_or_insert(root_node);
        if frame == 0 {
            ui.set_root(root_node);
        }
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        app.advance_frame();
    }

    let observed = observed.lock().unwrap();
    assert_eq!(observed.len(), 2);
    let first: std::collections::HashMap<u64, ModelId> = observed[0].iter().copied().collect();
    let second: std::collections::HashMap<u64, ModelId> = observed[1].iter().copied().collect();
    assert_eq!(
        first, second,
        "expected keyed child scope to preserve helper local_model identity across reorder"
    );
}

#[test]
fn model_for_preserves_model_identity_for_explicit_element_id() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let ids = Arc::new(std::sync::Mutex::new(Vec::<ModelId>::new()));

    let mut root: Option<NodeId> = None;
    for frame in 0..2 {
        let ids = ids.clone();
        let root_node = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "model-for-explicit-id",
            move |cx| {
                let overlay_id = GlobalElementId(0xC0FFEE);
                let model = cx.model_for(overlay_id, || frame as u64);
                ids.lock().unwrap().push(model.id());
                vec![cx.text("leaf")]
            },
        );

        root.get_or_insert(root_node);
        if frame == 0 {
            ui.set_root(root_node);
        }
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        app.advance_frame();
    }

    let ids = ids.lock().unwrap();
    assert_eq!(ids.len(), 2);
    assert_eq!(ids[0], ids[1]);
}

#[test]
#[cfg(feature = "diagnostics")]
fn named_scopes_appear_in_debug_paths() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let leaf_id = Arc::new(std::sync::Mutex::new(
        None::<crate::elements::GlobalElementId>,
    ));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "named-debug-path",
        |cx| {
            let leaf = cx.named("sidebar", |cx| cx.text("leaf"));
            *leaf_id.lock().unwrap() = Some(leaf.id);
            vec![leaf]
        },
    );
    ui.set_root(root);

    let leaf = leaf_id.lock().unwrap().expect("leaf id");
    let debug_path = app.with_global_mut(crate::elements::ElementRuntime::new, |runtime, _| {
        runtime.debug_path_for_element(window, leaf)
    });

    let debug_path = debug_path.expect("debug path");
    assert!(
        debug_path.contains("name=sidebar"),
        "expected named segment in debug path: {debug_path}"
    );
}

#[test]
#[cfg(feature = "diagnostics")]
fn debug_paths_survive_gc_when_touching_only_leaf_elements() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let leaf_id = Arc::new(std::sync::Mutex::new(
        None::<crate::elements::GlobalElementId>,
    ));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "debug-path-gc-touch-leaf",
        |cx| {
            let leaf = cx.named("a", |cx| cx.named("b", |cx| cx.text("leaf")));
            *leaf_id.lock().unwrap() = Some(leaf.id);
            vec![leaf]
        },
    );
    ui.set_root(root);

    let leaf = leaf_id.lock().unwrap().expect("leaf id");

    // Simulate a long run of cache-hit frames where only a subset of elements are touched for
    // liveness bookkeeping.
    for _ in 0..8 {
        app.advance_frame();
        let frame_id = app.frame_id();
        app.with_global_mut(crate::elements::ElementRuntime::new, |runtime, _| {
            runtime.prepare_window_for_frame(window, frame_id);
            runtime
                .for_window_mut(window)
                .touch_debug_identity_for_element(frame_id, leaf);
        });
    }

    let debug_path = app.with_global_mut(crate::elements::ElementRuntime::new, |runtime, _| {
        runtime.debug_path_for_element(window, leaf)
    });

    let debug_path = debug_path.expect("debug path");
    assert!(
        debug_path.contains("name=a") && debug_path.contains("name=b"),
        "expected named segments in debug path after GC touches: {debug_path}"
    );
}

#[test]
fn element_context_identity_docs_classify_component_internal_lane() {
    let api_source = CX_RS_SOURCE;

    assert!(api_source.contains(
        "Component/internal identity lane: accesses root-scoped state stored under the current"
    ));
    assert!(api_source.contains(
        "Component/internal identity lane: accesses helper-local state stored under a synthetic"
    ));
    assert!(api_source.contains(
        "Component/internal identity lane: returns a helper-local model handle stored under a"
    ));
    assert!(api_source.contains(
        "Component/internal identity lane: returns a model handle stored under an explicit element"
    ));
    assert!(api_source.contains(
        "Framework-internal diagnostics hook for repeated same-callsite use within one render"
    ));
    assert!(api_source.contains(
        "This is not a public authoring API; it exists so higher-level facade code can reuse the"
    ));
    assert!(api_source.contains("pub fn note_repeated_call_in_render_evaluation_at("));
    assert!(!api_source.contains("pub fn render_pass_id(&self) -> u64"));
}
