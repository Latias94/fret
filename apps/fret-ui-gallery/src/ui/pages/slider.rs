use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::slider as snippets;

pub(super) fn preview_slider(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let label = snippets::label::render(cx);

    let notes = doc_layout::notes_block([
        "API reference: `ecosystem/fret-ui-shadcn/src/slider.rs` (Slider).",
        "`slider(model)` is the default controlled helper for first-party teaching surfaces, while `new_controllable(...)` stays available when the example needs a default-value bridge or element-owned state.",
        "Uncontrolled sliders store their values in element state; controlled sliders store values in a shared model.",
        "Prefer `on_value_commit` for expensive reactions (e.g. save, fetch) and use live updates for lightweight UI.",
        "Vertical sliders should have an explicit height to avoid zero-size layouts.",
    ]);
    let notes = DocSection::build(cx, "Notes", notes).description("Behavior notes.");

    let extras = snippets::extras::render(cx);
    let demo = DocSection::build(cx, "Demo", demo)
        .description("shadcn demo: single, range, multiple, vertical, and controlled.")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .description("Copyable minimal usage for `slider(model)`.")
        .test_id_prefix("ui-gallery-slider-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let label = DocSection::build(cx, "Label Association", label)
        .description("Use `FieldLabel::for_control`, `Slider::control_id`, and `Slider::test_id_prefix` on top of `slider(model)` to focus the active thumb and keep derived automation anchors stable.")
        .test_id_prefix("ui-gallery-slider-label")
        .code_rust_from_file_region(snippets::label::SOURCE, "example");
    let extras = DocSection::build(cx, "Extras", extras)
        .description("Fret extras: disabled, RTL, inverted, and onValueCommit.")
        .code_rust_from_file_region(snippets::extras::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Slider docs flow: Demo -> Usage. Extras cover label association and Fret-specific interaction variants.",
        ),
        vec![demo, usage, label, extras, notes],
    );

    vec![body.test_id("ui-gallery-slider")]
}
