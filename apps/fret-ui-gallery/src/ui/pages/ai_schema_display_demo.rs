use super::super::*;

use crate::ui::doc_layout::DocSection;
use crate::ui::snippets::ai as snippets;
use fret::UiCx;

pub(super) fn preview_ai_schema_display_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let demo = snippets::schema_display_demo::render(cx);
    let demo_section = DocSection::build(cx, "SchemaDisplay", demo)
        .test_id_prefix("ui-gallery-ai-schema-display-demo")
        .code_rust_from_file_region(snippets::schema_display_demo::SOURCE, "example");

    let body = crate::ui::doc_layout::render_doc_page_after(
        Some("AI Elements are policy-level compositions built on top of lower-level primitives."),
        vec![demo_section],
        cx,
    );

    vec![body.into_element(cx)]
}
