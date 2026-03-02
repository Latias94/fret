use super::super::*;
use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::resizable as snippets;

pub(super) fn preview_resizable(
    cx: &mut ElementContext<'_, App>,
    h_fractions: Model<Vec<f32>>,
    v_fractions: Model<Vec<f32>>,
) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx, h_fractions, v_fractions);
    let handle = snippets::handle::render(cx);
    let vertical = snippets::vertical::render(cx);
    let rtl = snippets::rtl::render(cx);
    let notes = snippets::notes::render(cx);

    let body = doc_layout::render_doc_page(
        cx,
        Some("Drag the handles to resize panels."),
        vec![
            DocSection::new("Demo", demo)
                .description("Nested vertical panels inside a horizontal group.")
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-resizable-demo")
                .code_rust_from_file_region(include_str!("../snippets/resizable/demo.rs"), "example"),
            DocSection::new("Handle", handle)
                .description("A handle with a visual grabber (`withHandle`).")
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-resizable-handle")
                .code_rust_from_file_region(
                    include_str!("../snippets/resizable/handle.rs"),
                    "example",
                ),
            DocSection::new("Vertical", vertical)
                .description("Vertical orientation.")
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-resizable-vertical")
                .code_rust_from_file_region(
                    include_str!("../snippets/resizable/vertical.rs"),
                    "example",
                ),
            DocSection::new("RTL", rtl)
                .description("Direction provider coverage for hit-testing and handle affordances.")
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-resizable-rtl")
                .code_rust_from_file_region(include_str!("../snippets/resizable/rtl.rs"), "example"),
            DocSection::new("Notes", notes)
                .description("Parity notes and references.")
                .max_w(Px(820.0))
                .test_id_prefix("ui-gallery-resizable-notes"),
        ],
    );

    vec![body.test_id("ui-gallery-resizable")]
}

