use super::*;

#[test]
fn debug_draw_default_element_stays_noninteractive_canvas() {
    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui::elements::with_element_cx(&mut app, window, bounds(), "debug-draw.canvas", |cx| {
        let mut response = ResponseExt::default();
        let element = debug_draw_element(
            cx,
            empty_commands(),
            DebugDrawOptions {
                test_id: Some(Arc::from("imui.debug_draw")),
                ..Default::default()
            },
            &mut response,
        );

        assert!(matches!(element.kind, ElementKind::Canvas(_)));
        assert_eq!(
            element
                .semantics_decoration
                .as_ref()
                .and_then(|decoration| decoration.test_id.as_deref()),
            Some("imui.debug_draw")
        );
        assert!(!response.enabled());
    });
}

#[test]
fn debug_draw_interaction_wraps_canvas_in_pressable_response_surface() {
    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui::elements::with_element_cx(&mut app, window, bounds(), "debug-draw.pressable", |cx| {
        let mut response = ResponseExt::default();
        let element = debug_draw_element(
            cx,
            empty_commands(),
            DebugDrawOptions {
                test_id: Some(Arc::from("imui.debug_draw.interactive")),
                interaction: DebugDrawInteractionOptions::enabled()
                    .focusable(true)
                    .with_a11y_label("Debug draw canvas"),
                ..Default::default()
            },
            &mut response,
        );

        let ElementKind::Pressable(props) = &element.kind else {
            panic!("interactive debug draw should wrap the canvas in a pressable");
        };
        assert!(props.enabled);
        assert!(props.focusable);
        assert_eq!(props.a11y.label.as_deref(), Some("Debug draw canvas"));
        assert_eq!(props.a11y.test_id.as_deref(), None);
        assert_eq!(element.children.len(), 1);
        assert!(matches!(element.children[0].kind, ElementKind::Canvas(_)));
        assert_eq!(
            element.children[0]
                .semantics_decoration
                .as_ref()
                .and_then(|decoration| decoration.test_id.as_deref()),
            Some("imui.debug_draw.interactive")
        );
        assert!(response.enabled());
    });
}
