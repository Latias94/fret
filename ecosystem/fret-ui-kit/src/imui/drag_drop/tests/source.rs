use super::*;

#[test]
fn drag_source_returns_inactive_without_trigger_id() {
    let mut app = App::new();
    fret_ui::elements::with_element_cx(
        &mut app,
        Default::default(),
        Default::default(),
        "test",
        |cx| {
            let mut out = Vec::new();
            let mut ui = TestWriter { cx, out: &mut out };
            let response = drag_source_with_options(
                &mut ui,
                ResponseExt::default(),
                42_u32,
                DragSourceOptions::default(),
            );
            assert!(!response.active());
            assert!(out.is_empty());
        },
    );
}
