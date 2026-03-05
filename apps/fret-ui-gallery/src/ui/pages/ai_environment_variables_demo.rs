use super::super::*;

use crate::ui::doc_layout::DocSection;
use crate::ui::snippets::ai as snippets;

pub(super) fn preview_ai_environment_variables_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let demo = snippets::environment_variables_demo::render(cx);

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some("AI Elements are policy-level compositions built on top of lower-level primitives."),
        vec![
            DocSection::new("Environment Variables", demo)
                .descriptions([
                    "Value masking by default",
                    "Toggle visibility switch",
                    "Copy individual values",
                    "Export format support (export KEY=\"value\")",
                    "Required badge indicator",
                ])
                .test_id_prefix("ui-gallery-ai-environment-variables-demo")
                .code_rust_from_file_region(
                    snippets::environment_variables_demo::SOURCE,
                    "example",
                ),
        ],
    );

    vec![body]
}
