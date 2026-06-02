use super::*;

#[test]
fn scroll_option_viewport_test_id_lands_on_inner_scroll_root() {
    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds(),
        "scroll.viewport.test-id",
        |cx| {
            let element = scroll_container_element(
                cx,
                None,
                ScrollOptions {
                    viewport_test_id: Some(Arc::from("imui-scroll.viewport")),
                    ..Default::default()
                },
                |ui| ui.text("scroll"),
            );

            let inner = match &element.kind {
                ElementKind::Container(_) => element
                    .children
                    .first()
                    .expect("scroll helper should wrap an inner scroll root"),
                other => panic!("expected scroll helper outer container, got {other:?}"),
            };

            assert_eq!(
                inner
                    .semantics_decoration
                    .as_ref()
                    .and_then(|decoration| decoration.test_id.as_deref()),
                Some("imui-scroll.viewport")
            );
        },
    );
}
