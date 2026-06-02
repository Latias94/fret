use super::*;

#[test]
fn drop_target_returns_empty_without_trigger_id() {
    let mut app = App::new();
    fret_ui::elements::with_element_cx(
        &mut app,
        Default::default(),
        Default::default(),
        "test",
        |cx| {
            let mut out = Vec::new();
            let mut ui = TestWriter { cx, out: &mut out };
            let response = drop_target_with_options::<_, _, u32>(
                &mut ui,
                ResponseExt::default(),
                DropTargetOptions::default(),
            );
            assert!(!response.active());
            assert!(!response.over());
            assert!(!response.delivered());
            assert!(response.preview_payload().is_none());
            assert!(response.delivered_payload().is_none());
            assert!(out.is_empty());
        },
    );
}
