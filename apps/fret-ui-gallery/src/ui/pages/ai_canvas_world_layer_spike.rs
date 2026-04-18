use super::super::*;

use crate::ui::doc_layout::DocSection;
use crate::ui::snippets::ai as snippets;
use fret::AppComponentCx;

pub(super) fn preview_ai_canvas_world_layer_spike(
    cx: &mut AppComponentCx<'_>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let demo = snippets::canvas_world_layer_spike::render(cx);

    let body = crate::ui::doc_layout::render_doc_page_after(
        Some("AI Elements are policy-level compositions built on top of lower-level primitives."),
        vec![
            DocSection::build(cx, "Canvas World Layer (Spike)", demo)
                .max_w(Px(1000.0))
                .test_id_prefix("ui-gallery-ai-canvas-world-layer-spike")
                .code_rust_from_file_region(snippets::canvas_world_layer_spike::SOURCE, "example"),
        ],
        cx,
    );

    vec![body.into_element(cx)]
}
