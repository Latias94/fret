use super::fixtures::{
    TestCollectionDragPayload, test_collection_assets, test_collection_drag_payload_for_asset,
};
use super::*;
use fret_ui_kit::imui::ImUiMultiSelectState;

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
            selected_out.replace(state.selected().to_vec());
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
