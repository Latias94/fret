use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::{AppComponentCx, UiChild};

fn parts_table(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    doc_layout::text_table(
        cx,
        ["Surface", "Notes"],
        [
            [
                "InlineCitationRoot",
                "Compound-parts root aligned with the AI Elements docs example. Provides `InlineCitationParts` in scope (open state + pager index + resolved sources) so the child parts can stay purely compositional. Prefer `InlineCitationRoot::into_element_parts(...)` for the common docs-shaped `Text + Card` composition to avoid extra closure boilerplate.",
            ],
            [
                "InlineCitationText/Card/Trigger/Body/Carousel/...",
                "Compound parts that mirror the upstream JSX taxonomy. They default to docs-style chrome (secondary header + 320px card width) and derive diagnostic ids from the root `test_id(...)`.",
            ],
            [
                "InlineCitation",
                "Convenience-first root surface: resolves hostname badge text from `source_ids(...) + sources(...)`, owns the hover-card pager, and can emit `select_source_model(...)` for transcript/source syncing.",
            ],
            [
                "with_children(...)",
                "Convenience lane for caller-owned inline copy on `InlineCitation`. The gallery demo uses the compound root so the code aligns more directly with the official AI Elements docs.",
            ],
            [
                "source_ids(...) + sources(...)",
                "Multi-source resolution contract. A single inline citation can page across multiple `SourceItem`s, matching the upstream AI Elements behavior.",
            ],
            [
                "test_id(...)",
                "Installs a stable trigger id and derives `-label`, `-card`, `-prev`, `-next`, and `-index` ids for diagnostics.",
            ],
        ],
        false,
    )
}

pub(super) fn preview_ai_inline_citation_demo(
    cx: &mut AppComponentCx<'_>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let usage = snippets::inline_citation_demo::render(cx);
    let features = doc_layout::notes_block([
        "Preview now follows the official AI Elements example more closely: one paragraph, one inline citation span, one hostname/count badge, and the same hover-card pager pattern.",
        "The gallery code uses `InlineCitationRoot` + compound parts so the snippet structure stays close to the upstream JSX example, while hover-card state and paging remain policy-level behavior in `fret-ui-ai`.",
        "This page intentionally keeps the default usage copyable; Fret-specific app hooks such as `select_source_model(...)` stay documented as an extra seam instead of crowding the main example.",
        "This detail page is feature-gated behind `gallery-dev`, which also enables the `fret-ui-ai` demo surfaces.",
    ]);
    let parts = parts_table(cx);
    let notes = doc_layout::notes_block([
        "Layering check: this is an AI Elements policy-layer surface built on shadcn `Badge` + `HoverCard`, not a `crates/fret-ui` mechanism change.",
        "Root `refine_layout(...)` now applies to the inline citation container instead of only the badge trigger, which makes docs-style inline usage less surprising.",
        "Keep `ui-ai-inline-citation-demo-*` selectors stable; `tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-inline-citation-demo-hovercard.json` depends on them.",
    ]);

    let body = crate::ui::doc_layout::render_doc_page_after(
        Some(
            "Preview mirrors the official AI Elements Inline Citation example more closely, then documents the current Fret authoring surface and remaining API-shape tradeoffs.",
        ),
        vec![
            DocSection::build(cx, "Usage", usage)
                .test_id_prefix("ui-gallery-ai-inline-citation-demo")
                .description(
                    "Docs-aligned inline paragraph with the same hostname badge + multi-source hover-card paging pattern.",
                )
                .code_rust_from_file_region(snippets::inline_citation_demo::SOURCE, "example"),
            DocSection::build(cx, "Features", features)
                .description("High-signal parity notes against the official AI Elements docs.")
                .no_shell(),
            DocSection::build(cx, "Parts & Props", parts)
                .description("What the current Fret surface owns, and where the authoring seam still differs from JSX.")
                .no_shell(),
            DocSection::build(cx, "Notes", notes)
                .description("Layering, diagnostics, and authoring-surface notes.")
                .no_shell()
                .test_id_prefix("ui-gallery-ai-inline-citation-notes"),
        ],
        cx,
    );

    vec![body.into_element(cx)]
}
