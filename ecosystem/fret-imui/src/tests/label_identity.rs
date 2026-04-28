use super::*;

use fret_ui_kit::imui::ButtonOptions;

fn current_focus_test_id(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    bounds: Rect,
) -> Option<String> {
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
    let focus = ui.focus()?;
    let snap = ui.semantics_snapshot()?;
    snap.nodes
        .iter()
        .find(|node| node.id == focus)
        .and_then(|node| node.test_id.as_deref().map(str::to_owned))
}

#[test]
fn label_identity_button_suffixes_hide_from_text_and_preserve_focus_across_reorder() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(180.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let flipped = Rc::new(Cell::new(false));
    let progress = Rc::new(Cell::new(41));

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.vertical(|ui| {
                let task_a = (
                    format!("Task {}%###task-a", progress.get()),
                    "imui-label-identity.task-a",
                );
                let task_b = (
                    String::from("Task 99%###task-b"),
                    "imui-label-identity.task-b",
                );
                let items = if flipped.get() {
                    vec![task_b, task_a]
                } else {
                    vec![task_a, task_b]
                };

                for (label, test_id) in items {
                    let _ = ui.button_with_options(
                        label,
                        ButtonOptions {
                            test_id: Some(Arc::from(test_id)),
                            ..Default::default()
                        },
                    );
                }

                let _ = ui.button_with_options(
                    "Play##toolbar",
                    ButtonOptions {
                        test_id: Some(Arc::from("imui-label-identity.play")),
                        ..Default::default()
                    },
                );
                let _ = ui.button_with_options(
                    "##hidden-button",
                    ButtonOptions {
                        test_id: Some(Arc::from("imui-label-identity.hidden")),
                        ..Default::default()
                    },
                );
            });
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-label-identity",
        |cx| render(cx),
    );

    let _task_a = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-label-identity.task-a",
    );

    progress.set(42);
    flipped.set(true);
    advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-label-identity",
        &render,
    );

    assert_eq!(
        current_focus_test_id(&mut ui, &mut app, &mut services, bounds),
        Some(String::from("imui-label-identity.task-a"))
    );
    assert!(services.prepared.iter().any(|text| text == "Task 41%"));
    assert!(services.prepared.iter().any(|text| text == "Task 42%"));
    assert!(services.prepared.iter().any(|text| text == "Play"));
    assert!(
        !services.prepared.iter().any(|text| text.contains("##")
            || text.contains("###")
            || text.contains("toolbar")
            || text.contains("hidden-button")),
        "label identity suffixes should not be painted: {:?}",
        services.prepared
    );
}
