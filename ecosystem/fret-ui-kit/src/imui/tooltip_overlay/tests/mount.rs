use super::*;

#[test]
fn tooltip_returns_false_without_trigger_id() {
    let mut app = App::new();
    fret_ui::elements::with_element_cx(
        &mut app,
        Default::default(),
        Default::default(),
        "test",
        |cx| {
            let mut out = Vec::new();
            let mut ui = TestWriter { cx, out: &mut out };
            assert!(!tooltip_text_with_options(
                &mut ui,
                "tooltip",
                ResponseExt::default(),
                Arc::from("tip"),
                TooltipOptions::default(),
            ));
            assert!(out.is_empty());
        },
    );
}
