use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::{UiChild, UiCx};

pub(super) fn preview_ai_snippet_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let usage = snippets::snippet_demo::render(cx);
    let plain = snippets::snippet_plain::render(cx);
    let composable = snippets::snippet_composable::render(cx);
    let features = doc_layout::notes_block([
        "Audit outcome: no new `crates/fret-ui` mechanism gap surfaced here; the remaining parity work lives in `fret-ui-ai` defaults plus the UI Gallery teaching surface.",
        "The default surface stays close to AI Elements: `Snippet` owns the code payload while `SnippetAddon`, `SnippetText`, `SnippetInput`, and `SnippetCopyButton` remain composable parts.",
        "Fret keeps two honest authoring lanes: `with_code(...).into_element_with_children(...)` for context-driven parts, and `Snippet::new([...])` with explicit values for eager compound children.",
        "SnippetCopyButton now also accepts caller-owned visual children while preserving the built-in copy/check icon as the zero-config path.",
    ]);
    let notes = doc_layout::notes_block([
        "Clipboard copy semantics are now honest: `SnippetCopyButton.on_copy` runs after write success, and `on_error` exposes structured clipboard failures.",
        "This page now mirrors the official AI Elements Snippet docs path after skipping Installation: Usage, Features, Without Prefix, and Props. `Composable Children (Fret)` and `Notes` stay as explicit Fret follow-ups.",
        "`gallery-dev` enables the UI Gallery AI Elements surface by turning on `gallery-ai`, so this page only appears when that feature set is compiled in.",
    ]);
    let props = snippet_props_table(cx);

    let usage_section = DocSection::build(cx, "Usage", usage)
        .descriptions([
            "Docs-aligned terminal command preview with a prefix addon and copied-state marker.",
            "Uses the context-driven parts lane so `SnippetInput::from_context()` and `SnippetCopyButton::from_context()` stay copyable.",
        ])
        .test_id_prefix("ui-gallery-ai-snippet-usage")
        .code_rust_from_file_region(snippets::snippet_demo::SOURCE, "example");
    let features_section = DocSection::build(cx, "Features", features)
        .description("High-signal parity findings and the current owner split for Snippet.")
        .test_id_prefix("ui-gallery-ai-snippet-features")
        .no_shell();
    let plain_section = DocSection::build(cx, "Without Prefix", plain)
        .description("Official plain variant without the terminal-style prompt prefix.")
        .test_id_prefix("ui-gallery-ai-snippet-plain")
        .code_rust_from_file_region(snippets::snippet_plain::SOURCE, "example");
    let composable_section = DocSection::build(cx, "Composable Children (Fret)", composable)
        .description("Fret-specific eager composition lane: build the root from explicit parts without relying on inherited snippet context.")
        .test_id_prefix("ui-gallery-ai-snippet-composable")
        .code_rust_from_file_region(snippets::snippet_composable::SOURCE, "example");
    let props_section = DocSection::build(cx, "Props", props)
        .description("Fret builder surface corresponding to the upstream `Snippet*` compound component family.")
        .test_id_prefix("ui-gallery-ai-snippet-props")
        .no_shell();
    let notes_section = DocSection::build(cx, "Notes", notes)
        .description(
            "Parity conclusions, remaining gaps, and feature-gate context for the Snippet surface.",
        )
        .test_id_prefix("ui-gallery-ai-snippet-notes");

    let body = crate::ui::doc_layout::render_doc_page_after(
        Some(
            "Docs-aligned Snippet coverage for AI Elements: primary usage preview, plain example, the current Fret composable-children lane, and a focused props table for the compound parts surface.",
        ),
        vec![
            usage_section,
            features_section,
            plain_section,
            composable_section,
            props_section,
            notes_section,
        ],
        cx,
    );

    vec![body.into_element(cx)]
}

fn snippet_props_table(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    doc_layout::text_table(
        cx,
        ["Part", "Method", "Type", "Default", "Description"],
        [
            [
                "Snippet",
                "new(children)",
                "IntoIterator<Item = AnyElement>",
                "-",
                "Eager compound-children lane. Use this when each part already has an explicit code value.",
            ],
            [
                "Snippet",
                "with_code / code",
                "impl Into<Arc<str>>",
                "Required",
                "Installs the code payload used by context-driven parts such as `SnippetInput::from_context()`.",
            ],
            [
                "Snippet",
                "into_element_with_children",
                "FnOnce(&mut ElementContext) -> Vec<AnyElement>",
                "-",
                "Preferred lazy builder when child parts need the inherited snippet context.",
            ],
            [
                "SnippetAddon",
                "align",
                "SnippetAddonAlign",
                "InlineStart",
                "Routes compound content to the inline-start or inline-end edge of the snippet row.",
            ],
            [
                "SnippetInput",
                "new / from_context",
                "builder methods",
                "from_context() in docs path",
                "Choose between explicit code and inherited snippet context for the read-only selectable text segment.",
            ],
            [
                "SnippetCopyButton",
                "new / from_context",
                "builder methods",
                "from_context() in docs path",
                "Binds the copy affordance to either an explicit code value or the nearest snippet context.",
            ],
            [
                "SnippetCopyButton",
                "children",
                "IntoIterator<Item = AnyElement>",
                "default copy/check icon",
                "Overrides the default icon with caller-owned button content while keeping the copy semantics and timeout state.",
            ],
            [
                "SnippetCopyButton",
                "on_copy / on_error / timeout",
                "builder methods",
                "None / None / 2s",
                "Runs app-owned side effects after clipboard write success, exposes structured failure callbacks, and controls how long the copied state stays active.",
            ],
            [
                "Snippet / CopyButton",
                "test_id / copied_marker_test_id",
                "impl Into<Arc<str>>",
                "None",
                "Stable diagnostics selectors for root, button, and copied-state gating in `fretboard-dev diag` scripts.",
            ],
        ],
        true,
    )
}
