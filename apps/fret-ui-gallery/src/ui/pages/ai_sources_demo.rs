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
                "Sources::into_element_parts(trigger, content, cx)",
                "Typed Fret analogue of the official `<Sources>...</Sources>` nesting. The root still owns the collapsible model, while trigger/content stay compositional.",
            ],
            [
                "SourcesTrigger::new(count)",
                "Docs-shaped trigger part. Default label is `Used {count} sources`, and `children(...)` lets callers override the visible row while `title(...)` remains the semantic fallback label.",
            ],
            [
                "SourcesContent::new(children)",
                "Collapsible content wrapper aligned with the upstream `mt-3 flex flex-col gap-2` slot. Use it for the copyable docs preview and custom examples.",
            ],
            [
                "Source::new(title)",
                "Fret analogue of the upstream `title` prop / default child fallback. `children(...)` unlocks custom visible row content, while `href(...) + with_open_url()` or `on_open_url(...)` owns activation.",
            ],
            [
                "SourcesBlock::new(items)",
                "Transcript-oriented convenience surface kept for `MessageParts` and similar assistant-output lanes. It is still useful, but it is not the first-party docs teaching surface anymore.",
            ],
        ],
        false,
    )
}

pub(super) fn preview_ai_sources_demo(
    cx: &mut AppComponentCx<'_>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let usage = snippets::sources_demo::render(cx);
    let custom = snippets::sources_custom_demo::render(cx);

    let features = doc_layout::notes_block([
        "Collapsible source/citation disclosure built on the existing shadcn/Radix-style `Collapsible` contract.",
        "Docs-shaped compound surface is now available: `Sources` + `SourcesTrigger` + `SourcesContent` + `Source`.",
        "Custom trigger content and custom source row children are both supported, which matches the official AI Elements custom-rendering example more closely.",
        "Intrinsic-width preview behavior remains guarded by the existing UI Gallery diagnostics script; the change is on the public authoring surface, not on `fret-ui` mechanisms.",
    ]);

    let notes = doc_layout::notes_block([
        "Mechanism health looks good here: the underlying collapsible semantics were already fine; the main drift was public-surface and teaching-surface parity.",
        "The official docs teach a compound component family. Previously Fret only exposed `SourcesBlock`, so the Gallery snippet had to teach the convenience lane first and could not reproduce the custom-rendering example honestly.",
        "Composable children are supported where they matter for parity: `SourcesTrigger::children(...)` and `Source::children(...)`. The root stays typed via `into_element_parts(...)`, which is the Rust/Fret analogue of JSX nesting because the collapsible open model still has to be wired explicitly.",
        "Link activation remains app-owned by default. For docs/demo parity, `Source::with_open_url()` is the lightweight Fret analogue of an anchor `href`, while `on_open_url(...)` keeps the effect seam explicit.",
        "This detail page is feature-gated behind `gallery-dev`, which also enables the `fret-ui-ai` demo surfaces.",
    ]);

    let parts = parts_table(cx);

    let body = crate::ui::doc_layout::render_doc_page_after(
        Some(
            "Docs-aligned Sources coverage for AI Elements: default preview, custom trigger/source rendering, and the current Fret authoring-surface notes.",
        ),
        vec![
            DocSection::build(cx, "Usage", usage)
                .description("Rust/Fret analogue of the official AI Elements Sources preview.")
                .test_id_prefix("ui-gallery-ai-sources-demo")
                .code_rust_from_file_region(snippets::sources_demo::SOURCE, "example"),
            DocSection::build(cx, "Custom Rendering", custom)
                .description(
                    "Docs-aligned custom trigger and custom source row example using the new compound parts surface.",
                )
                .test_id_prefix("ui-gallery-ai-sources-custom")
                .code_rust_from_file_region(snippets::sources_custom_demo::SOURCE, "example"),
            DocSection::build(cx, "Features", features)
                .description("High-signal parity findings for the Sources surface."),
            DocSection::build(cx, "Parts & Props", parts)
                .description("Current Fret API surface and where it intentionally differs from JSX."),
            DocSection::build(cx, "Notes", notes)
                .description("Layering, diagnostics, and authoring-surface notes for Sources."),
        ],
        cx,
    );

    vec![body.into_element(cx)]
}
