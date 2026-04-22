use super::*;

use fret_ui_kit::imui::ImUiMultiSelectState;
use fret_ui_kit::recipes::imui_drag_preview::{
    DragPreviewGhostOptions, drag_preview_ghost_with_options,
    publish_cross_window_drag_preview_ghost_with_options, render_cross_window_drag_preview_ghosts,
};
use fret_ui_kit::recipes::imui_sortable::{
    SortableInsertionSide, reorder_vec_by_key, sortable_row,
};

#[derive(Clone)]
struct TestDragPayload {
    label: Arc<str>,
}

#[derive(Clone, PartialEq, Eq)]
struct TestCollectionAsset {
    id: Arc<str>,
    label: Arc<str>,
    path: Arc<str>,
}

#[derive(Clone)]
struct TestCollectionDragPayload {
    ids: Arc<[Arc<str>]>,
    paths: Arc<[Arc<str>]>,
}

#[derive(Clone, PartialEq, Eq)]
struct TestSortableItem {
    id: Arc<str>,
    label: Arc<str>,
}

#[derive(Clone)]
struct TestSortablePayload {
    id: Arc<str>,
    label: Arc<str>,
}

fn test_sortable_items() -> Vec<TestSortableItem> {
    vec![
        TestSortableItem {
            id: Arc::from("camera"),
            label: Arc::from("Camera"),
        },
        TestSortableItem {
            id: Arc::from("cube"),
            label: Arc::from("Cube"),
        },
        TestSortableItem {
            id: Arc::from("key-light"),
            label: Arc::from("Key light"),
        },
    ]
}

fn test_collection_assets() -> Arc<[TestCollectionAsset]> {
    vec![
        TestCollectionAsset {
            id: Arc::from("alpha"),
            label: Arc::from("Alpha"),
            path: Arc::from("textures/alpha.ktx2"),
        },
        TestCollectionAsset {
            id: Arc::from("beta"),
            label: Arc::from("Beta"),
            path: Arc::from("textures/beta.ktx2"),
        },
        TestCollectionAsset {
            id: Arc::from("gamma"),
            label: Arc::from("Gamma"),
            path: Arc::from("textures/gamma.ktx2"),
        },
        TestCollectionAsset {
            id: Arc::from("delta"),
            label: Arc::from("Delta"),
            path: Arc::from("textures/delta.ktx2"),
        },
    ]
    .into()
}

fn selected_test_collection_assets<'a>(
    assets: &'a [TestCollectionAsset],
    selection: &ImUiMultiSelectState<Arc<str>>,
) -> Vec<&'a TestCollectionAsset> {
    selection
        .selected
        .iter()
        .filter_map(|id| assets.iter().find(|asset| asset.id == *id))
        .collect()
}

fn test_collection_drag_payload_for_asset(
    assets: &[TestCollectionAsset],
    selection: &ImUiMultiSelectState<Arc<str>>,
    dragged: &TestCollectionAsset,
) -> TestCollectionDragPayload {
    let selected_assets = selected_test_collection_assets(assets, selection);
    let payload_assets = if selection.is_selected(&dragged.id) && !selected_assets.is_empty() {
        selected_assets
    } else {
        vec![dragged]
    };

    let ids = payload_assets
        .iter()
        .map(|asset| asset.id.clone())
        .collect::<Vec<_>>();
    let paths = payload_assets
        .iter()
        .map(|asset| asset.path.clone())
        .collect::<Vec<_>>();

    TestCollectionDragPayload {
        ids: ids.into(),
        paths: paths.into(),
    }
}

fn test_sortable_order_line(items: &[TestSortableItem]) -> String {
    items
        .iter()
        .map(|item| item.label.as_ref())
        .collect::<Vec<_>>()
        .join(" -> ")
}

fn render_test_sortable_rows(
    items: &Rc<RefCell<Vec<TestSortableItem>>>,
    preview_status: &Rc<RefCell<String>>,
    delivered_status: &Rc<RefCell<String>>,
    order_status: &Rc<RefCell<String>>,
    delivered_flag: &Rc<Cell<bool>>,
) -> impl FnOnce(&mut ElementContext<'_, TestHost>) -> crate::Elements + use<> {
    let items = items.clone();
    let preview_status = preview_status.clone();
    let delivered_status = delivered_status.clone();
    let order_status = order_status.clone();
    let delivered_flag = delivered_flag.clone();

    move |cx| {
        crate::imui_raw(cx, |ui| {
            let snapshot = items.borrow().clone();
            let mut pending_reorder: Option<(
                Arc<str>,
                Arc<str>,
                Arc<str>,
                Arc<str>,
                SortableInsertionSide,
            )> = None;
            let mut preview = String::new();

            ui.vertical(|ui| {
                for item in &snapshot {
                    let row = ui.button_with_options(
                        item.label.clone(),
                        fret_ui_kit::imui::ButtonOptions {
                            test_id: Some(Arc::from(format!("imui-sortable-row.{}", item.id))),
                            ..Default::default()
                        },
                    );
                    let payload = TestSortablePayload {
                        id: item.id.clone(),
                        label: item.label.clone(),
                    };
                    let sortable = sortable_row(ui, row, payload);

                    if let Some(signal) = sortable.delivered_reorder() {
                        let dragged = signal.payload();
                        if dragged.id != item.id {
                            pending_reorder = Some((
                                dragged.id.clone(),
                                dragged.label.clone(),
                                item.id.clone(),
                                item.label.clone(),
                                signal.side(),
                            ));
                        }
                    } else if let Some(signal) = sortable.preview_reorder() {
                        let dragged = signal.payload();
                        let side = signal.side();
                        if dragged.id != item.id {
                            preview = format!(
                                "Preview: move {} {} {}",
                                dragged.label,
                                side.label(),
                                item.label
                            );
                        }
                    }
                }
            });

            let mut delivered_message = String::new();
            let mut delivered = false;
            if let Some((active_id, active_label, over_id, over_label, side)) = pending_reorder {
                delivered = reorder_vec_by_key(
                    &mut items.borrow_mut(),
                    active_id.as_ref(),
                    over_id.as_ref(),
                    side,
                    |item| item.id.as_ref(),
                );
                if delivered {
                    delivered_message =
                        format!("Moved {} {} {}", active_label, side.label(), over_label);
                }
            }

            preview_status.replace(preview);
            delivered_status.replace(delivered_message);
            delivered_flag.set(delivered);
            order_status.replace(test_sortable_order_line(&items.borrow()));
        })
    }
}
#[test]
fn multi_selectable_supports_plain_toggle_and_range_clicks() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(220.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let items = Arc::<[Arc<str>]>::from(vec![
        Arc::from("Alpha"),
        Arc::from("Beta"),
        Arc::from("Gamma"),
        Arc::from("Delta"),
    ]);
    let selection_model = app
        .models_mut()
        .insert(ImUiMultiSelectState::<Arc<str>>::default());
    let selected = Rc::new(RefCell::new(Vec::<Arc<str>>::new()));
    let anchor = Rc::new(RefCell::new(None::<Arc<str>>));

    let render = |cx: &mut ElementContext<'_, TestHost>,
                  selected_out: &Rc<RefCell<Vec<Arc<str>>>>,
                  anchor_out: &Rc<RefCell<Option<Arc<str>>>>| {
        crate::imui_raw(cx, |ui| {
            ui.vertical(|ui| {
                for (index, item) in items.iter().enumerate() {
                    let _ = ui.multi_selectable_with_options(
                        item.clone(),
                        &selection_model,
                        items.as_ref(),
                        item.clone(),
                        fret_ui_kit::imui::SelectableOptions {
                            test_id: Some(Arc::from(format!("imui-multi-select.option.{index}"))),
                            ..Default::default()
                        },
                    );
                }
            });

            let state = ui
                .cx_mut()
                .app
                .models()
                .get_cloned(&selection_model)
                .unwrap_or_default();
            selected_out.replace(state.selected.clone());
            anchor_out.replace(state.anchor.clone());
        })
    };

    let selected_out = selected.clone();
    let anchor_out = anchor.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-multi-select",
        |cx| render(cx, &selected_out, &anchor_out),
    );
    assert!(selected.borrow().is_empty());
    assert!(anchor.borrow().is_none());

    let beta = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-multi-select.option.1",
    );
    click_at(&mut ui, &mut app, &mut services, beta);

    app.advance_frame();
    let selected_out = selected.clone();
    let anchor_out = anchor.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-multi-select",
        |cx| render(cx, &selected_out, &anchor_out),
    );
    assert_eq!(selected.borrow().as_slice(), &[Arc::<str>::from("Beta")]);
    assert_eq!(anchor.borrow().as_deref(), Some("Beta"));

    let delta = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-multi-select.option.3",
    );
    click_at_with_modifiers(
        &mut ui,
        &mut app,
        &mut services,
        delta,
        Modifiers {
            meta: true,
            ..Default::default()
        },
    );

    app.advance_frame();
    let selected_out = selected.clone();
    let anchor_out = anchor.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-multi-select",
        |cx| render(cx, &selected_out, &anchor_out),
    );
    assert_eq!(
        selected.borrow().as_slice(),
        &[Arc::<str>::from("Beta"), Arc::<str>::from("Delta")]
    );
    assert_eq!(anchor.borrow().as_deref(), Some("Delta"));

    let alpha = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-multi-select.option.0",
    );
    click_at_with_modifiers(
        &mut ui,
        &mut app,
        &mut services,
        alpha,
        Modifiers {
            shift: true,
            ..Default::default()
        },
    );

    app.advance_frame();
    let selected_out = selected.clone();
    let anchor_out = anchor.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-multi-select",
        |cx| render(cx, &selected_out, &anchor_out),
    );
    assert_eq!(
        selected.borrow().as_slice(),
        &[
            Arc::<str>::from("Alpha"),
            Arc::<str>::from("Beta"),
            Arc::<str>::from("Gamma"),
            Arc::<str>::from("Delta"),
        ]
    );
    assert_eq!(anchor.borrow().as_deref(), Some("Delta"));
}
#[test]
fn collection_drag_payload_preserves_selected_keys_across_order_flip() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(220.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let assets = test_collection_assets();
    let selection_model = app
        .models_mut()
        .insert(ImUiMultiSelectState::<Arc<str>>::default());
    let reverse_order = Rc::new(Cell::new(false));
    let selected_ids = Rc::new(RefCell::new(Vec::<Arc<str>>::new()));
    let preview_ids = Rc::new(RefCell::new(Vec::<Arc<str>>::new()));
    let preview_paths = Rc::new(RefCell::new(Vec::<Arc<str>>::new()));
    let delivered_ids = Rc::new(RefCell::new(Vec::<Arc<str>>::new()));
    let delivered_paths = Rc::new(RefCell::new(Vec::<Arc<str>>::new()));

    let render = |cx: &mut ElementContext<'_, TestHost>,
                  reverse_order: &Rc<Cell<bool>>,
                  selected_out: &Rc<RefCell<Vec<Arc<str>>>>,
                  preview_ids_out: &Rc<RefCell<Vec<Arc<str>>>>,
                  preview_paths_out: &Rc<RefCell<Vec<Arc<str>>>>,
                  delivered_ids_out: &Rc<RefCell<Vec<Arc<str>>>>,
                  delivered_paths_out: &Rc<RefCell<Vec<Arc<str>>>>| {
        crate::imui_raw(cx, |ui| {
            let mut visible_assets = assets.iter().cloned().collect::<Vec<_>>();
            if reverse_order.get() {
                visible_assets.reverse();
            }
            let all_keys = visible_assets
                .iter()
                .map(|asset| asset.id.clone())
                .collect::<Vec<_>>();
            let selection_state = ui
                .cx_mut()
                .app
                .models()
                .get_cloned(&selection_model)
                .unwrap_or_default();

            ui.vertical(|ui| {
                for asset in &visible_assets {
                    ui.id(asset.id.clone(), |ui| {
                        let trigger = ui.multi_selectable_with_options(
                            asset.label.clone(),
                            &selection_model,
                            &all_keys,
                            asset.id.clone(),
                            fret_ui_kit::imui::SelectableOptions {
                                test_id: Some(Arc::from(format!(
                                    "imui-collection-dnd.asset.{}",
                                    asset.id
                                ))),
                                ..Default::default()
                            },
                        );
                        let _ = ui.drag_source(
                            trigger,
                            test_collection_drag_payload_for_asset(
                                &visible_assets,
                                &selection_state,
                                asset,
                            ),
                        );
                    });
                }

                let target = ui.button_with_options(
                    "Import",
                    fret_ui_kit::imui::ButtonOptions {
                        test_id: Some(Arc::from("imui-collection-dnd.target")),
                        ..Default::default()
                    },
                );
                let drop = ui.drop_target::<TestCollectionDragPayload>(target);
                preview_ids_out.replace(
                    drop.preview_payload()
                        .map(|payload| payload.ids.iter().cloned().collect())
                        .unwrap_or_default(),
                );
                preview_paths_out.replace(
                    drop.preview_payload()
                        .map(|payload| payload.paths.iter().cloned().collect())
                        .unwrap_or_default(),
                );
                delivered_ids_out.replace(
                    drop.delivered_payload()
                        .map(|payload| payload.ids.iter().cloned().collect())
                        .unwrap_or_default(),
                );
                delivered_paths_out.replace(
                    drop.delivered_payload()
                        .map(|payload| payload.paths.iter().cloned().collect())
                        .unwrap_or_default(),
                );
            });

            let state = ui
                .cx_mut()
                .app
                .models()
                .get_cloned(&selection_model)
                .unwrap_or_default();
            selected_out.replace(state.selected);
        })
    };

    let selected_out = selected_ids.clone();
    let preview_ids_out = preview_ids.clone();
    let preview_paths_out = preview_paths.clone();
    let delivered_ids_out = delivered_ids.clone();
    let delivered_paths_out = delivered_paths.clone();
    let reverse_order_out = reverse_order.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-collection-dnd",
        |cx| {
            render(
                cx,
                &reverse_order_out,
                &selected_out,
                &preview_ids_out,
                &preview_paths_out,
                &delivered_ids_out,
                &delivered_paths_out,
            )
        },
    );
    assert!(selected_ids.borrow().is_empty());
    assert!(preview_ids.borrow().is_empty());
    assert!(delivered_ids.borrow().is_empty());

    let beta = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-collection-dnd.asset.beta",
    );
    click_at(&mut ui, &mut app, &mut services, beta);

    app.advance_frame();
    let selected_out = selected_ids.clone();
    let preview_ids_out = preview_ids.clone();
    let preview_paths_out = preview_paths.clone();
    let delivered_ids_out = delivered_ids.clone();
    let delivered_paths_out = delivered_paths.clone();
    let reverse_order_out = reverse_order.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-collection-dnd",
        |cx| {
            render(
                cx,
                &reverse_order_out,
                &selected_out,
                &preview_ids_out,
                &preview_paths_out,
                &delivered_ids_out,
                &delivered_paths_out,
            )
        },
    );
    assert_eq!(
        selected_ids.borrow().as_slice(),
        &[Arc::<str>::from("beta")]
    );

    let delta = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-collection-dnd.asset.delta",
    );
    click_at_with_modifiers(
        &mut ui,
        &mut app,
        &mut services,
        delta,
        Modifiers {
            meta: true,
            ..Default::default()
        },
    );

    app.advance_frame();
    let selected_out = selected_ids.clone();
    let preview_ids_out = preview_ids.clone();
    let preview_paths_out = preview_paths.clone();
    let delivered_ids_out = delivered_ids.clone();
    let delivered_paths_out = delivered_paths.clone();
    let reverse_order_out = reverse_order.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-collection-dnd",
        |cx| {
            render(
                cx,
                &reverse_order_out,
                &selected_out,
                &preview_ids_out,
                &preview_paths_out,
                &delivered_ids_out,
                &delivered_paths_out,
            )
        },
    );
    assert_eq!(
        selected_ids.borrow().as_slice(),
        &[Arc::<str>::from("beta"), Arc::<str>::from("delta")]
    );

    reverse_order.set(true);

    app.advance_frame();
    let selected_out = selected_ids.clone();
    let preview_ids_out = preview_ids.clone();
    let preview_paths_out = preview_paths.clone();
    let delivered_ids_out = delivered_ids.clone();
    let delivered_paths_out = delivered_paths.clone();
    let reverse_order_out = reverse_order.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-collection-dnd",
        |cx| {
            render(
                cx,
                &reverse_order_out,
                &selected_out,
                &preview_ids_out,
                &preview_paths_out,
                &delivered_ids_out,
                &delivered_paths_out,
            )
        },
    );
    assert_eq!(
        selected_ids.borrow().as_slice(),
        &[Arc::<str>::from("beta"), Arc::<str>::from("delta")]
    );

    let drag_source = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-collection-dnd.asset.delta",
    );
    let target = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-collection-dnd.target",
    );

    pointer_down_at(&mut ui, &mut app, &mut services, drag_source);
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        target,
        MouseButtons {
            left: true,
            ..MouseButtons::default()
        },
    );

    app.advance_frame();
    let selected_out = selected_ids.clone();
    let preview_ids_out = preview_ids.clone();
    let preview_paths_out = preview_paths.clone();
    let delivered_ids_out = delivered_ids.clone();
    let delivered_paths_out = delivered_paths.clone();
    let reverse_order_out = reverse_order.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-collection-dnd",
        |cx| {
            render(
                cx,
                &reverse_order_out,
                &selected_out,
                &preview_ids_out,
                &preview_paths_out,
                &delivered_ids_out,
                &delivered_paths_out,
            )
        },
    );
    assert_eq!(
        preview_ids.borrow().as_slice(),
        &[Arc::<str>::from("beta"), Arc::<str>::from("delta")]
    );
    assert_eq!(
        preview_paths.borrow().as_slice(),
        &[
            Arc::<str>::from("textures/beta.ktx2"),
            Arc::<str>::from("textures/delta.ktx2")
        ]
    );
    assert!(delivered_ids.borrow().is_empty());

    pointer_up_at(&mut ui, &mut app, &mut services, target);

    app.advance_frame();
    let selected_out = selected_ids.clone();
    let preview_ids_out = preview_ids.clone();
    let preview_paths_out = preview_paths.clone();
    let delivered_ids_out = delivered_ids.clone();
    let delivered_paths_out = delivered_paths.clone();
    let reverse_order_out = reverse_order.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-collection-dnd",
        |cx| {
            render(
                cx,
                &reverse_order_out,
                &selected_out,
                &preview_ids_out,
                &preview_paths_out,
                &delivered_ids_out,
                &delivered_paths_out,
            )
        },
    );
    assert_eq!(
        delivered_ids.borrow().as_slice(),
        &[Arc::<str>::from("beta"), Arc::<str>::from("delta")]
    );
    assert_eq!(
        delivered_paths.borrow().as_slice(),
        &[
            Arc::<str>::from("textures/beta.ktx2"),
            Arc::<str>::from("textures/delta.ktx2")
        ]
    );
}
#[test]
fn drag_started_stopped_and_delta_are_consistent() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let started = Rc::new(Cell::new(false));
    let dragging = Rc::new(Cell::new(false));
    let stopped = Rc::new(Cell::new(false));
    let delta = Rc::new(Cell::new(Point::default()));

    let started_out = started.clone();
    let dragging_out = dragging.clone();
    let stopped_out = stopped.clone();
    let delta_out = delta.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-signals",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                started_out.set(resp.drag_started());
                dragging_out.set(resp.dragging());
                stopped_out.set(resp.drag_stopped());
                delta_out.set(resp.drag_delta());
            })
        },
    );
    assert!(!started.get());
    assert!(!dragging.get());
    assert!(!stopped.get());

    let start = first_child_point(&ui, root);
    pointer_down_at(&mut ui, &mut app, &mut services, start);

    app.advance_frame();
    let started_out = started.clone();
    let dragging_out = dragging.clone();
    let stopped_out = stopped.clone();
    let delta_out = delta.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-signals",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                started_out.set(resp.drag_started());
                dragging_out.set(resp.dragging());
                stopped_out.set(resp.drag_stopped());
                delta_out.set(resp.drag_delta());
            })
        },
    );
    assert!(!started.get());
    assert!(!dragging.get());
    assert!(!stopped.get());

    // Move below the threshold.
    let p1 = Point::new(Px(start.x.0 + 2.0), Px(start.y.0));
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        p1,
        MouseButtons {
            left: true,
            ..MouseButtons::default()
        },
    );

    app.advance_frame();
    let started_out = started.clone();
    let dragging_out = dragging.clone();
    let stopped_out = stopped.clone();
    let delta_out = delta.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-signals",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                started_out.set(resp.drag_started());
                dragging_out.set(resp.dragging());
                stopped_out.set(resp.drag_stopped());
                delta_out.set(resp.drag_delta());
            })
        },
    );
    assert!(!started.get());
    assert!(!dragging.get());
    assert!(!stopped.get());

    // Move past the threshold to start dragging (delta should be the frame delta, not the total).
    let p2 = Point::new(Px(start.x.0 + 6.0), Px(start.y.0));
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        p2,
        MouseButtons {
            left: true,
            ..MouseButtons::default()
        },
    );

    app.advance_frame();
    let started_out = started.clone();
    let dragging_out = dragging.clone();
    let stopped_out = stopped.clone();
    let delta_out = delta.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-signals",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                started_out.set(resp.drag_started());
                dragging_out.set(resp.dragging());
                stopped_out.set(resp.drag_stopped());
                delta_out.set(resp.drag_delta());
            })
        },
    );
    assert!(started.get());
    assert!(dragging.get());
    assert!(!stopped.get());
    assert_eq!(delta.get(), Point::new(Px(4.0), Px(0.0)));

    pointer_up_at_with_is_click(&mut ui, &mut app, &mut services, p2, false);

    app.advance_frame();
    let started_out = started.clone();
    let dragging_out = dragging.clone();
    let stopped_out = stopped.clone();
    let delta_out = delta.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-signals",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                started_out.set(resp.drag_started());
                dragging_out.set(resp.dragging());
                stopped_out.set(resp.drag_stopped());
                delta_out.set(resp.drag_delta());
            })
        },
    );
    assert!(!started.get());
    assert!(!dragging.get());
    assert!(stopped.get());
}
#[test]
fn drag_threshold_metric_controls_drag_start() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    fret_ui::Theme::with_global_mut(&mut app, |theme| {
        let mut cfg = fret_ui::theme::ThemeConfig {
            name: "Test".to_string(),
            ..fret_ui::theme::ThemeConfig::default()
        };
        cfg.metrics
            .insert("component.imui.drag_threshold_px".to_string(), 7.0);
        theme.apply_config_patch(&cfg);
    });
    let mut services = FakeTextService::default();

    let started = Rc::new(Cell::new(false));
    let dragging = Rc::new(Cell::new(false));
    let delta = Rc::new(Cell::new(Point::default()));

    let started_out = started.clone();
    let dragging_out = dragging.clone();
    let delta_out = delta.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-threshold-metric",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                started_out.set(resp.drag_started());
                dragging_out.set(resp.dragging());
                delta_out.set(resp.drag_delta());
            })
        },
    );
    assert!(!started.get());
    assert!(!dragging.get());

    let start = first_child_point(&ui, root);
    pointer_down_at(&mut ui, &mut app, &mut services, start);

    // Move below the configured threshold (7px).
    let p1 = Point::new(Px(start.x.0 + 6.0), Px(start.y.0));
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        p1,
        MouseButtons {
            left: true,
            ..MouseButtons::default()
        },
    );

    app.advance_frame();
    let started_out = started.clone();
    let dragging_out = dragging.clone();
    let delta_out = delta.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-threshold-metric",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                started_out.set(resp.drag_started());
                dragging_out.set(resp.dragging());
                delta_out.set(resp.drag_delta());
            })
        },
    );
    assert!(!started.get());
    assert!(!dragging.get());

    // Move past the threshold; delta should be the frame delta (8 - 6 = 2).
    let p2 = Point::new(Px(start.x.0 + 8.0), Px(start.y.0));
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        p2,
        MouseButtons {
            left: true,
            ..MouseButtons::default()
        },
    );

    app.advance_frame();
    let started_out = started.clone();
    let dragging_out = dragging.clone();
    let delta_out = delta.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-threshold-metric",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                started_out.set(resp.drag_started());
                dragging_out.set(resp.dragging());
                delta_out.set(resp.drag_delta());
            })
        },
    );
    assert!(started.get());
    assert!(dragging.get());
    assert_eq!(delta.get(), Point::new(Px(2.0), Px(0.0)));
}
#[test]
fn drag_drop_helper_previews_and_delivers_payload() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(160.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let source_active = Rc::new(Cell::new(false));
    let target_over = Rc::new(Cell::new(false));
    let delivered = Rc::new(Cell::new(false));
    let preview_label = Rc::new(RefCell::new(String::new()));
    let delivered_label = Rc::new(RefCell::new(String::new()));

    let source_active_out = source_active.clone();
    let target_over_out = target_over.clone();
    let delivered_out = delivered.clone();
    let preview_label_out = preview_label.clone();
    let delivered_label_out = delivered_label.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-drop-helper",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.horizontal(|ui| {
                    let source = ui.button_with_options(
                        "Asset",
                        fret_ui_kit::imui::ButtonOptions {
                            test_id: Some(Arc::from("imui-dnd-source")),
                            ..Default::default()
                        },
                    );
                    let source_state = ui.drag_source(
                        source,
                        TestDragPayload {
                            label: Arc::from("Stone"),
                        },
                    );
                    source_active_out.set(source_state.active());

                    let target = ui.button_with_options(
                        "Slot",
                        fret_ui_kit::imui::ButtonOptions {
                            test_id: Some(Arc::from("imui-dnd-target")),
                            ..Default::default()
                        },
                    );
                    let drop = ui.drop_target::<TestDragPayload>(target);
                    target_over_out.set(drop.over());
                    delivered_out.set(drop.delivered());
                    preview_label_out.replace(
                        drop.preview_payload()
                            .map(|payload| payload.label.as_ref().to_string())
                            .unwrap_or_default(),
                    );
                    delivered_label_out.replace(
                        drop.delivered_payload()
                            .map(|payload| payload.label.as_ref().to_string())
                            .unwrap_or_default(),
                    );
                });
            })
        },
    );

    assert!(!source_active.get());
    assert!(!target_over.get());
    assert!(!delivered.get());
    assert!(preview_label.borrow().is_empty());
    assert!(delivered_label.borrow().is_empty());

    let source_point =
        point_for_test_id(&mut ui, &mut app, &mut services, bounds, "imui-dnd-source");
    let target_point =
        point_for_test_id(&mut ui, &mut app, &mut services, bounds, "imui-dnd-target");

    pointer_down_at(&mut ui, &mut app, &mut services, source_point);
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        target_point,
        MouseButtons {
            left: true,
            ..MouseButtons::default()
        },
    );

    app.advance_frame();
    let source_active_out = source_active.clone();
    let target_over_out = target_over.clone();
    let delivered_out = delivered.clone();
    let preview_label_out = preview_label.clone();
    let delivered_label_out = delivered_label.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-drop-helper",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.horizontal(|ui| {
                    let source = ui.button_with_options(
                        "Asset",
                        fret_ui_kit::imui::ButtonOptions {
                            test_id: Some(Arc::from("imui-dnd-source")),
                            ..Default::default()
                        },
                    );
                    let source_state = ui.drag_source(
                        source,
                        TestDragPayload {
                            label: Arc::from("Stone"),
                        },
                    );
                    source_active_out.set(source_state.active());

                    let target = ui.button_with_options(
                        "Slot",
                        fret_ui_kit::imui::ButtonOptions {
                            test_id: Some(Arc::from("imui-dnd-target")),
                            ..Default::default()
                        },
                    );
                    let drop = ui.drop_target::<TestDragPayload>(target);
                    target_over_out.set(drop.over());
                    delivered_out.set(drop.delivered());
                    preview_label_out.replace(
                        drop.preview_payload()
                            .map(|payload| payload.label.as_ref().to_string())
                            .unwrap_or_default(),
                    );
                    delivered_label_out.replace(
                        drop.delivered_payload()
                            .map(|payload| payload.label.as_ref().to_string())
                            .unwrap_or_default(),
                    );
                });
            })
        },
    );

    assert!(source_active.get());
    assert!(target_over.get());
    assert!(!delivered.get());
    assert_eq!(preview_label.borrow().as_str(), "Stone");
    assert!(delivered_label.borrow().is_empty());

    pointer_up_at_with_is_click(&mut ui, &mut app, &mut services, target_point, false);

    app.advance_frame();
    let source_active_out = source_active.clone();
    let target_over_out = target_over.clone();
    let delivered_out = delivered.clone();
    let preview_label_out = preview_label.clone();
    let delivered_label_out = delivered_label.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-drop-helper",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.horizontal(|ui| {
                    let source = ui.button_with_options(
                        "Asset",
                        fret_ui_kit::imui::ButtonOptions {
                            test_id: Some(Arc::from("imui-dnd-source")),
                            ..Default::default()
                        },
                    );
                    let source_state = ui.drag_source(
                        source,
                        TestDragPayload {
                            label: Arc::from("Stone"),
                        },
                    );
                    source_active_out.set(source_state.active());

                    let target = ui.button_with_options(
                        "Slot",
                        fret_ui_kit::imui::ButtonOptions {
                            test_id: Some(Arc::from("imui-dnd-target")),
                            ..Default::default()
                        },
                    );
                    let drop = ui.drop_target::<TestDragPayload>(target);
                    target_over_out.set(drop.over());
                    delivered_out.set(drop.delivered());
                    preview_label_out.replace(
                        drop.preview_payload()
                            .map(|payload| payload.label.as_ref().to_string())
                            .unwrap_or_default(),
                    );
                    delivered_label_out.replace(
                        drop.delivered_payload()
                            .map(|payload| payload.label.as_ref().to_string())
                            .unwrap_or_default(),
                    );
                });
            })
        },
    );

    assert!(!source_active.get());
    assert!(!target_over.get());
    assert!(delivered.get());
    assert!(preview_label.borrow().is_empty());
    assert_eq!(delivered_label.borrow().as_str(), "Stone");
}
#[test]
fn drag_preview_ghost_follows_pointer_and_clears_on_release() {
    fn subtree_contains_bounds(
        ui: &UiTree<TestHost>,
        node: fret_core::NodeId,
        expected: Rect,
    ) -> bool {
        if ui.debug_node_bounds(node) == Some(expected) {
            return true;
        }

        ui.children(node)
            .iter()
            .copied()
            .any(|child| subtree_contains_bounds(ui, child, expected))
    }

    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(160.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let source_position = Rc::new(Cell::new(None::<Point>));
    let source_position_out = source_position.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-preview-ghost",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let source = ui.button_with_options(
                    "Asset",
                    fret_ui_kit::imui::ButtonOptions {
                        test_id: Some(Arc::from("imui-drag-preview-source")),
                        ..Default::default()
                    },
                );
                let source_state = ui.drag_source(
                    source,
                    TestDragPayload {
                        label: Arc::from("Stone"),
                    },
                );
                source_position_out.set(source_state.position());

                let _ = drag_preview_ghost_with_options(
                    ui,
                    "asset-ghost",
                    source_state,
                    DragPreviewGhostOptions {
                        test_id: Some(Arc::from("imui-drag-preview-ghost")),
                        ..Default::default()
                    },
                    fret_ui_kit::ui::container_build(|cx, out| {
                        let mut props = fret_ui::element::ContainerProps::default();
                        props.layout.size.width = fret_ui::element::Length::Px(Px(96.0));
                        props.layout.size.height = fret_ui::element::Length::Px(Px(28.0));
                        out.push(cx.container(props, |cx| vec![cx.text("Stone")]));
                    }),
                );
            })
        },
    );

    assert!(source_position.get().is_none());
    assert_eq!(ui.layer_ids_in_paint_order().len(), 1);

    let source_point = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-drag-preview-source",
    );
    let drag_point = Point::new(Px(source_point.x.0 + 24.0), Px(source_point.y.0 + 18.0));

    pointer_down_at(&mut ui, &mut app, &mut services, source_point);
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        drag_point,
        MouseButtons {
            left: true,
            ..MouseButtons::default()
        },
    );

    app.advance_frame();
    let source_position_out = source_position.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-preview-ghost",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let source = ui.button_with_options(
                    "Asset",
                    fret_ui_kit::imui::ButtonOptions {
                        test_id: Some(Arc::from("imui-drag-preview-source")),
                        ..Default::default()
                    },
                );
                let source_state = ui.drag_source(
                    source,
                    TestDragPayload {
                        label: Arc::from("Stone"),
                    },
                );
                source_position_out.set(source_state.position());

                let _ = drag_preview_ghost_with_options(
                    ui,
                    "asset-ghost",
                    source_state,
                    DragPreviewGhostOptions {
                        test_id: Some(Arc::from("imui-drag-preview-ghost")),
                        ..Default::default()
                    },
                    fret_ui_kit::ui::container_build(|cx, out| {
                        let mut props = fret_ui::element::ContainerProps::default();
                        props.layout.size.width = fret_ui::element::Length::Px(Px(96.0));
                        props.layout.size.height = fret_ui::element::Length::Px(Px(28.0));
                        out.push(cx.container(props, |cx| vec![cx.text("Stone")]));
                    }),
                );
            })
        },
    );

    assert_eq!(source_position.get(), Some(drag_point));
    let ghost_layer = *ui
        .layer_ids_in_paint_order()
        .last()
        .expect("expected ghost overlay layer");
    assert!(ui.is_layer_visible(ghost_layer));
    let ghost_root = ui.layer_root(ghost_layer).expect("ghost layer root");
    let expected_bounds = Rect::new(
        Point::new(Px(drag_point.x.0 + 12.0), Px(drag_point.y.0 + 12.0)),
        Size::new(Px(96.0), Px(28.0)),
    );
    assert!(subtree_contains_bounds(&ui, ghost_root, expected_bounds));

    pointer_up_at_with_is_click(&mut ui, &mut app, &mut services, drag_point, false);

    app.advance_frame();
    let source_position_out = source_position.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-preview-ghost",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let source = ui.button_with_options(
                    "Asset",
                    fret_ui_kit::imui::ButtonOptions {
                        test_id: Some(Arc::from("imui-drag-preview-source")),
                        ..Default::default()
                    },
                );
                let source_state = ui.drag_source(
                    source,
                    TestDragPayload {
                        label: Arc::from("Stone"),
                    },
                );
                source_position_out.set(source_state.position());

                let _ = drag_preview_ghost_with_options(
                    ui,
                    "asset-ghost",
                    source_state,
                    DragPreviewGhostOptions {
                        test_id: Some(Arc::from("imui-drag-preview-ghost")),
                        ..Default::default()
                    },
                    fret_ui_kit::ui::container_build(|cx, out| {
                        let mut props = fret_ui::element::ContainerProps::default();
                        props.layout.size.width = fret_ui::element::Length::Px(Px(96.0));
                        props.layout.size.height = fret_ui::element::Length::Px(Px(28.0));
                        out.push(cx.container(props, |cx| vec![cx.text("Stone")]));
                    }),
                );
            })
        },
    );

    assert!(source_position.get().is_none());
    assert!(!ui.is_layer_visible(ghost_layer));
}
#[test]
fn cross_window_drag_preview_ghost_transfers_between_windows() {
    use slotmap::KeyData;

    let window_a = AppWindowId::from(KeyData::from_ffi(1));
    let window_b = AppWindowId::from(KeyData::from_ffi(2));
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(160.0)),
    );

    let mut ui_a = UiTree::new();
    ui_a.set_window(window_a);
    let mut ui_b = UiTree::new();
    ui_b.set_window(window_b);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let render_scene = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            let source = ui.button_with_options(
                "Asset",
                fret_ui_kit::imui::ButtonOptions {
                    test_id: Some(Arc::from("imui-cross-window-drag-preview-source")),
                    ..Default::default()
                },
            );
            let source_state = ui.drag_source(
                source,
                TestDragPayload {
                    label: Arc::from("Stone"),
                },
            );

            let _ = publish_cross_window_drag_preview_ghost_with_options(
                ui,
                "asset-ghost",
                source_state,
                DragPreviewGhostOptions {
                    test_id: Some(Arc::from("imui-cross-window-drag-preview-ghost")),
                    ..Default::default()
                },
                |_cx| {
                    fret_ui_kit::ui::container_build(|cx, out| {
                        let mut props = fret_ui::element::ContainerProps::default();
                        props.layout.size.width = fret_ui::element::Length::Px(Px(96.0));
                        props.layout.size.height = fret_ui::element::Length::Px(Px(28.0));
                        out.push(cx.container(props, |cx| vec![cx.text("Stone")]));
                    })
                },
            );

            let _ = render_cross_window_drag_preview_ghosts(ui.cx_mut());
        })
    };

    let _ = run_frame(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds,
        "imui-cross-window-drag-preview",
        render_scene,
    );
    let _ = run_frame(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds,
        "imui-cross-window-drag-preview",
        render_scene,
    );
    assert!(!has_test_id(
        &mut ui_a,
        &mut app,
        &mut services,
        bounds,
        "imui-cross-window-drag-preview-ghost",
    ));
    assert!(!has_test_id(
        &mut ui_b,
        &mut app,
        &mut services,
        bounds,
        "imui-cross-window-drag-preview-ghost",
    ));

    let source_point = point_for_test_id(
        &mut ui_a,
        &mut app,
        &mut services,
        bounds,
        "imui-cross-window-drag-preview-source",
    );
    let drag_point = Point::new(Px(source_point.x.0 + 24.0), Px(source_point.y.0 + 18.0));

    pointer_down_at(&mut ui_a, &mut app, &mut services, source_point);
    pointer_move_at(
        &mut ui_a,
        &mut app,
        &mut services,
        drag_point,
        MouseButtons {
            left: true,
            ..MouseButtons::default()
        },
    );

    app.advance_frame();
    let _ = run_frame(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds,
        "imui-cross-window-drag-preview",
        render_scene,
    );
    let _ = run_frame(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds,
        "imui-cross-window-drag-preview",
        render_scene,
    );

    let ghost_layer_a = *ui_a
        .layer_ids_in_paint_order()
        .last()
        .expect("expected ghost overlay layer in window a");
    assert!(ui_a.is_layer_visible(ghost_layer_a));
    assert!(!has_test_id(
        &mut ui_b,
        &mut app,
        &mut services,
        bounds,
        "imui-cross-window-drag-preview-ghost",
    ));

    let drag = app.drag_mut(PointerId(0)).expect("drag session");
    drag.current_window = window_b;
    drag.cross_window_hover = true;
    drag.position = Point::new(Px(120.0), Px(72.0));

    app.advance_frame();
    let _ = run_frame(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds,
        "imui-cross-window-drag-preview",
        render_scene,
    );
    let _ = run_frame(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds,
        "imui-cross-window-drag-preview",
        render_scene,
    );

    assert!(!ui_a.is_layer_visible(ghost_layer_a));
    let ghost_layer_b = *ui_b
        .layer_ids_in_paint_order()
        .last()
        .expect("expected ghost overlay layer in window b");
    assert!(ui_b.is_layer_visible(ghost_layer_b));

    app.cancel_drag(PointerId(0));

    app.advance_frame();
    let _ = run_frame(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds,
        "imui-cross-window-drag-preview",
        render_scene,
    );
    let _ = run_frame(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds,
        "imui-cross-window-drag-preview",
        render_scene,
    );

    assert!(!ui_a.is_layer_visible(ghost_layer_a));
    assert!(!ui_b.is_layer_visible(ghost_layer_b));
}
#[test]
fn sortable_rows_reorder_using_drop_positions() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(220.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let items = Rc::new(RefCell::new(test_sortable_items()));
    let preview_status = Rc::new(RefCell::new(String::new()));
    let delivered_status = Rc::new(RefCell::new(String::new()));
    let order_status = Rc::new(RefCell::new(String::new()));
    let delivered_flag = Rc::new(Cell::new(false));

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-sortable-rows",
        render_test_sortable_rows(
            &items,
            &preview_status,
            &delivered_status,
            &order_status,
            &delivered_flag,
        ),
    );

    assert_eq!(
        order_status.borrow().as_str(),
        "Camera -> Cube -> Key light"
    );
    assert!(preview_status.borrow().is_empty());
    assert!(delivered_status.borrow().is_empty());
    assert!(!delivered_flag.get());

    let source_point = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-sortable-row.camera",
    );
    let _target_point = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-sortable-row.cube",
    );
    let target_bounds = bounds_for_test_id(&ui, "imui-sortable-row.cube");
    let target_lower = Point::new(
        Px(target_bounds.origin.x.0 + target_bounds.size.width.0 * 0.5),
        Px(target_bounds.origin.y.0 + target_bounds.size.height.0 * 0.75),
    );

    pointer_down_at(&mut ui, &mut app, &mut services, source_point);
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        target_lower,
        MouseButtons {
            left: true,
            ..MouseButtons::default()
        },
    );

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-sortable-rows",
        render_test_sortable_rows(
            &items,
            &preview_status,
            &delivered_status,
            &order_status,
            &delivered_flag,
        ),
    );

    assert_eq!(
        preview_status.borrow().as_str(),
        "Preview: move Camera after Cube"
    );
    assert!(delivered_status.borrow().is_empty());
    assert_eq!(
        order_status.borrow().as_str(),
        "Camera -> Cube -> Key light"
    );
    assert!(!delivered_flag.get());

    pointer_up_at_with_is_click(&mut ui, &mut app, &mut services, target_lower, false);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-sortable-rows",
        render_test_sortable_rows(
            &items,
            &preview_status,
            &delivered_status,
            &order_status,
            &delivered_flag,
        ),
    );

    assert!(preview_status.borrow().is_empty());
    assert_eq!(
        delivered_status.borrow().as_str(),
        "Moved Camera after Cube"
    );
    assert_eq!(
        order_status.borrow().as_str(),
        "Cube -> Camera -> Key light"
    );
    assert!(delivered_flag.get());
}
