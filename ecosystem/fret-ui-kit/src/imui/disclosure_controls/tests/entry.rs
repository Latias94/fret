use super::*;

#[test]
fn collapsing_header_default_open_mounts_body() {
    let mut app = App::new();
    fret_ui::elements::with_element_cx(
        &mut app,
        Default::default(),
        Default::default(),
        "test",
        |cx| {
            let mut out = Vec::new();
            let mut ui = TestWriter { cx, out: &mut out };
            let response = collapsing_header_with_options(
                &mut ui,
                "header",
                Arc::from("Section"),
                CollapsingHeaderOptions {
                    default_open: true,
                    ..Default::default()
                },
                |ui| {
                    ui.text("Body");
                },
            );

            assert!(response.open());
            assert_eq!(out.len(), 1);
            assert!(contains_text(&out[0], "Section"));
            assert!(contains_text(&out[0], "Body"));
        },
    );
}
