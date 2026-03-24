use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::progress as snippets;

pub(super) fn preview_progress(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let label = snippets::label::render(cx);
    let controlled = snippets::controlled::render(cx);
    let rtl = snippets::rtl::render(cx);

    let api_reference = doc_layout::notes_block([
        "Upstream docs path: `repo-ref/ui/apps/v4/content/docs/components/radix/progress.mdx`. Base UI reference: `repo-ref/ui/apps/v4/content/docs/components/base/progress.mdx`.",
        "`Progress::from_value(...)` mirrors the upstream `value` prop for read-only snapshot usage. `Progress::new(...)`, `new_opt(...)`, and `new_values_first(...)` remain the model-backed lanes for timers, sliders, and shared state.",
        "Progress remains a leaf control on the shadcn/Radix lane: labels and surrounding value rows are composed with `Field` / `FieldLabel` rather than widening the recipe with a generic children/`compose()` API.",
        "Base UI's `ProgressLabel` / `ProgressValue` children API is a useful headless reference, but it belongs to a different public surface and is not promoted on the default shadcn lane.",
        "Standalone bars should set `a11y_label(...)`; the demo uses a one-shot timer (500ms) to update the value from 13 to 66, matching the upstream motion example.",
    ]);
    let notes = doc_layout::notes_block([
        "This page keeps the upstream docs path first (`Demo`, `Usage`, `Label`, `Controlled`, `RTL`) and leaves `API Reference` plus `Notes` as Fret follow-ups.",
        "The review did not indicate a missing mechanism-layer change: progress semantics, determinate/indeterminate value handling, and indicator geometry already live behind the shadcn recipe and its existing gates.",
        "Base UI remains a useful headless reference for future alternate surfaces, but it does not justify widening the default shadcn-facing authoring surface today.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .test_id_prefix("ui-gallery-progress-api-reference")
        .description("Public surface summary, docs-parity notes, and children API ownership.");
    let notes = DocSection::build(cx, "Notes", notes)
        .test_id_prefix("ui-gallery-progress-notes")
        .description("Parity notes and owner split.");
    let demo = DocSection::build(cx, "Demo", demo)
        .description("One-shot timer preview aligned with the upstream demo's 13% -> 66% update.")
        .test_id_prefix("ui-gallery-progress-demo")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .description("Copyable minimal usage for the upstream-shaped snapshot `value` lane.")
        .test_id_prefix("ui-gallery-progress-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let label = DocSection::build(cx, "Label", label)
        .description("Field + label row composition aligned with the Radix docs example.")
        .test_id_prefix("ui-gallery-progress-label")
        .code_rust_from_file_region(snippets::label::SOURCE, "example");
    let controlled = DocSection::build(cx, "Controlled", controlled)
        .description("Slider-driven model-backed progress for externally synchronized state.")
        .test_id_prefix("ui-gallery-progress-controlled")
        .code_rust_from_file_region(snippets::controlled::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("RTL progress fill plus localized labels and percent text.")
        .test_id_prefix("ui-gallery-progress-rtl")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn/Radix Progress docs path first: Demo, Usage, Label, Controlled, RTL. `API Reference` and `Notes` then explain the snapshot/value lane, the model-backed bridges, and why Base UI's children API stays out of the default shadcn surface.",
        ),
        vec![demo, usage, label, controlled, rtl, api_reference, notes],
    );

    vec![body.test_id("ui-gallery-progress").into_element(cx)]
}
