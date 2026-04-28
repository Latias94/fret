use super::*;
use fret_core::NodeId;

#[test]
fn identity_diagnostics_record_imui_unkeyed_reorder() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(180.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
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
            "imui-unkeyed-reorder-identity-diagnostics",
            move |cx| {
                crate::imui_raw(cx, |ui| {
                    ui.for_each_unkeyed(&items, |ui, _index, _item| {
                        let row = ui.cx_mut().text("row");
                        ui.add(row);
                    });
                })
            },
        );

        root.get_or_insert(root_node);
        if frame == 0 {
            ui.set_root(root_node);
        }
        app.advance_frame();
    }

    let warnings = app.with_global_mut(fret_ui::elements::ElementRuntime::new, |runtime, _| {
        runtime
            .diagnostics_snapshot(window)
            .expect("diagnostics snapshot")
            .identity_warnings
    });

    let Some((previous_len, next_len, file)) = warnings.iter().find_map(|record| match record {
        fret_ui::elements::IdentityDiagnosticsRecord::UnkeyedListOrderChanged {
            previous_len,
            next_len,
            file,
            ..
        } => Some((*previous_len, *next_len, *file)),
        _ => None,
    }) else {
        panic!("expected IMUI unkeyed reorder identity warning, got {warnings:#?}");
    };

    assert_eq!(previous_len, 3);
    assert_eq!(next_len, 3);
    assert!(
        file.ends_with("identity_diagnostics.rs"),
        "expected IMUI delegation to preserve the author callsite, got {file}"
    );
}
