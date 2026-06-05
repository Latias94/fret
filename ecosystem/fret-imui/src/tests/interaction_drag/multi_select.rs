use super::*;
use fret_ui_kit::imui::ImUiMultiSelectState;

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
            selected_out.replace(state.selected().to_vec());
            anchor_out.replace(state.anchor().cloned());
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
