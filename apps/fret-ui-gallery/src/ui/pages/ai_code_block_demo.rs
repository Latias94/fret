use super::super::*;

use crate::ui::doc_layout::{DocSection, DocTabsSizing};
use crate::ui::snippets::ai as snippets;
use fret::UiCx;

pub(super) fn preview_ai_code_block_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let demo = snippets::code_block_demo::render(cx);

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some(
            "Provides syntax highlighting, line numbers, and copy-to-clipboard ergonomics for code blocks. The Fret surface keeps rendering and scrolling in `fret-code-view`, while the AI layer owns the composable header/actions API.",
        ),
        vec![
            DocSection::build(cx, "CodeBlock", demo)
                .tabs_sizing(DocTabsSizing::FillRemaining)
                .max_w(Px(680.0))
                .descriptions([
                    "Composable header/title/actions structure aligned with the official AI Elements docs example.",
                    "Language switching reuses shadcn-style Select composition in the policy layer.",
                    "Copy affordance consumes the nearest CodeBlock context instead of duplicating code props.",
                    "Use the gallery theme preset switcher to inspect the same chrome under dark mode.",
                ])
                .test_id_prefix("ui-gallery-ai-code-block-demo")
                .code_rust_from_file_region(snippets::code_block_demo::SOURCE, "example"),
        ],
    );

    let body = body.test_id("ui-gallery-page-ai-code-block-demo");
    vec![body.into_element(cx)]
}
