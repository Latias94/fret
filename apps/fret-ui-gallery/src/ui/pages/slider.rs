use super::super::*;
use fret::AppComponentCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::slider as snippets;

pub(super) fn preview_slider(cx: &mut AppComponentCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let range = snippets::range::render(cx);
    let multiple = snippets::multiple::render(cx);
    let vertical = snippets::vertical::render(cx);
    let controlled = snippets::controlled::render(cx);
    let disabled = snippets::disabled::render(cx);
    let rtl = snippets::rtl::render(cx);
    let label = snippets::label::render(cx);
    let extras = snippets::extras::render(cx);
    let api_reference = doc_layout::notes_block([
        "Reference stack: current shadcn Slider docs and new-york-v4 source, with Base/Radix registry examples as secondary references.",
        "Example axis: current shadcn slider demo and usage first; Base/Radix examples supply range, multiple-thumbs, vertical, controlled, and disabled follow-ups.",
        "Recipe axis: the default shadcn registry slider plus the base and radix registry variants.",
        "The current upstream docs surface intentionally splits the top-of-page preview (`[50]`, `w-[60%]`) from the `Usage` code block (`[33]`), so this page mirrors those two lanes instead of normalizing them to one demo value.",
        "Default first-party teaching should prefer `slider(model)`, while `new_controllable(...)` stays as the builder-preserving bridge for the upstream `defaultValue` lane and element-owned state.",
        "`slider(model)` is the default controlled helper for first-party teaching surfaces, while `new_controllable(...)` stays available when the example needs a default-value bridge or element-owned state.",
        "Slider remains a leaf recipe on the shadcn lane: labels, value readouts, and field layout are composed outside the control, so no extra generic composable children / `compose()` API is needed here.",
        "Base UI's `Slider.Root/Label/Value/Control/Track/Indicator/Thumb` family is a useful headless reference, but it belongs to a future `fret-ui-kit`-level surface rather than the `fret-ui-shadcn::Slider` recipe.",
        "This page is docs/public-surface parity work, not a mechanism-layer fix.",
    ]);
    let notes = doc_layout::notes_block([
        "API reference: `ecosystem/fret-ui-shadcn/src/slider.rs` (Slider).",
        "Uncontrolled sliders store their values in element state; controlled sliders store values in a shared model.",
        "Prefer `on_value_commit` for expensive reactions (e.g. save, fetch) and use live updates for lightweight UI.",
        "Vertical sliders keep the upstream `min-h-44` floor; examples can still pass an explicit height to bound the docs lane, but values below the floor clamp upward unless the caller asks for something taller.",
        "`test_id_prefix(...)` derives stable automation anchors for `track`, `range`, and `thumb-*` sub-parts.",
    ]);
    let demo = DocSection::build(cx, "Demo", demo)
        .description("Single slider preview aligned with the current upstream top-of-page demo.")
        .test_id_prefix("ui-gallery-slider-demo")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .description("Copyable minimal controlled usage for `slider(model)` on the first-party app surface; use `new_controllable(...)` for the upstream `defaultValue`-style lane.")
        .test_id_prefix("ui-gallery-slider-usage-section")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let range = DocSection::build(cx, "Range (Base/Radix)", range)
        .description("Mirrors the Base/Radix two-thumb range example (`[25, 50]`, `step(5)`).")
        .test_id_prefix("ui-gallery-slider-range-section")
        .code_rust_from_file_region(snippets::range::SOURCE, "example");
    let multiple = DocSection::build(cx, "Multiple Thumbs (Base/Radix)", multiple)
        .description("Mirrors the Base/Radix three-thumb example (`[10, 20, 70]`, `step(10)`).")
        .test_id_prefix("ui-gallery-slider-multiple-section")
        .code_rust_from_file_region(snippets::multiple::SOURCE, "example");
    let vertical = DocSection::build(cx, "Vertical (Base/Radix)", vertical)
        .description(
            "Mirrors the Base/Radix two-slider vertical example; the example still passes `h-40` in the call site, while the recipe-owned `min-h-44` floor remains the effective minimum unless the caller requests something taller.",
        )
        .test_id_prefix("ui-gallery-slider-vertical-section")
        .code_rust_from_file_region(snippets::vertical::SOURCE, "example");
    let controlled = DocSection::build(cx, "Controlled (Base/Radix)", controlled)
        .description("Mirrors the Base/Radix label + readout demo with `ControlId` / `Label::for_control(...)` on the Fret surface.")
        .test_id_prefix("ui-gallery-slider-controlled-section")
        .code_rust_from_file_region(snippets::controlled::SOURCE, "example");
    let disabled = DocSection::build(cx, "Disabled (Base/Radix)", disabled)
        .description("Disable the slider with `disabled(true)`.")
        .test_id_prefix("ui-gallery-slider-disabled-section")
        .code_rust_from_file_region(snippets::disabled::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL (Fret)", rtl)
        .description("Direction-provider parity for pointer and keyboard behavior under RTL.")
        .test_id_prefix("ui-gallery-slider-rtl-section")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let label = DocSection::build(cx, "Label Association (Fret)", label)
        .description("Use `FieldLabel::for_control`, `Slider::control_id`, and `Slider::test_id_prefix` on top of `slider(model)` to focus the active thumb and keep derived automation anchors stable.")
        .test_id_prefix("ui-gallery-slider-label-section")
        .code_rust_from_file_region(snippets::label::SOURCE, "example");
    let extras = DocSection::build(cx, "Extras (Fret)", extras)
        .description("Fret follow-ups: commit-only side effects and inverted value progression.")
        .test_id_prefix("ui-gallery-slider-extras-section")
        .code_rust_from_file_region(snippets::extras::SOURCE, "example");
    let api_reference = DocSection::build(cx, "API Reference (Fret)", api_reference)
        .description("Public surface summary, docs-parity notes, and children API ownership.")
        .no_shell()
        .test_id_prefix("ui-gallery-slider-api-reference");
    let notes = DocSection::build(cx, "Notes", notes)
        .description("Behavior notes.")
        .no_shell()
        .test_id_prefix("ui-gallery-slider-notes");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the current shadcn Slider docs path first: `Demo` and `Usage`. `Range`, `Multiple Thumbs`, `Vertical`, `Controlled`, and `Disabled` follow the Base/Radix example surface; `RTL`, `API Reference`, `Label Association`, `Extras`, and `Notes` stay as focused Fret follow-ups.",
        ),
        vec![
            demo,
            usage,
            range,
            multiple,
            vertical,
            controlled,
            disabled,
            rtl,
            api_reference,
            label,
            extras,
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-slider").into_element(cx)]
}
