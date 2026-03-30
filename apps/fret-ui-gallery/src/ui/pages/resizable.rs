use super::super::*;
use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::resizable as snippets;
use fret::UiCx;

pub(super) fn preview_resizable(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let vertical = snippets::vertical::render(cx);
    let handle = snippets::handle::render(cx);
    let rtl = snippets::rtl::render(cx);
    let notes = snippets::notes::render(cx);
    let about = doc_layout::notes_block([
        "Reference stack: shadcn Resizable docs on the Base UI and Radix lanes.",
        "The current visual/chrome baseline comes from the default shadcn registry recipe, with parallel headless baselines in the base and radix registry variants.",
        "Unlike `slider` or `progress`, there is no direct `Resizable` primitive in Radix Primitives or Base UI; those libraries still inform general headless/mechanism decisions, but the concrete source axis here is shadcn plus the runtime panel-group contract.",
        "This page is docs/public-surface parity work, not a mechanism-layer gap: drag routing, hit-testing, focusable splitter semantics, and min-size clamping already live in `fret-ui`.",
    ]);
    let api_reference = doc_layout::notes_block([
        "`ResizablePanelGroup::new(model).entries([...])` and `shadcn::resizable_panel_group(cx, model, |cx| ..)` cover the documented authoring surface.",
        "`resizable_panel_group(cx, model, |cx| ..)` is already the composable children-equivalent lane for Fret: it keeps `ResizablePanel` / `ResizableHandle` ordering explicit while preserving root-level `.axis(...)`, `.style(...)`, `.test_id_prefix(...)`, and layout refinements.",
        "A generic composable children / `compose()` API is not warranted here: the typed `ResizableEntry` stream already carries the source-aligned `Panel / Handle / Panel` contract without hiding handle order or widening the public surface.",
        "`ResizablePanelGroup` owns the upstream `w-full h-full` fill behavior plus handle chrome, while surrounding `rounded-lg border`, `max-w-*`, and fixed preview heights remain caller-owned like the shadcn docs/examples.",
        "`ResizableHandle::with_handle(true)` maps the documented visible-grabber lane, while keyboard splitter semantics and focus order remain runtime-owned.",
    ]);
    let about = DocSection::build(cx, "About", about)
        .no_shell()
        .description(
            "Source axes and why this component is already on the right runtime/mechanism split.",
        )
        .test_id_prefix("ui-gallery-resizable-about");
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .description("Public surface summary, ownership notes, and the children-API decision.")
        .test_id_prefix("ui-gallery-resizable-api-reference");
    let notes = DocSection::build(cx, "Notes", notes)
        .no_shell()
        .description("Remaining parity notes and diagnostics anchors.")
        .test_id_prefix("ui-gallery-resizable-notes");
    let demo = DocSection::build(cx, "Demo", demo)
        .description("Nested vertical panels inside a horizontal group.")
        .test_id_prefix("ui-gallery-resizable-demo")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .description(
            "Copyable minimal usage for `resizable_panel_group(...)`, `ResizablePanel`, and `ResizableHandle`.",
        )
        .test_id_prefix("ui-gallery-resizable-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let vertical = DocSection::build(cx, "Vertical", vertical)
        .description("Vertical orientation.")
        .test_id_prefix("ui-gallery-resizable-vertical")
        .code_rust_from_file_region(snippets::vertical::SOURCE, "example");
    let handle = DocSection::build(cx, "Handle", handle)
        .description("A handle with a visual grabber (`withHandle`).")
        .test_id_prefix("ui-gallery-resizable-handle")
        .code_rust_from_file_region(snippets::handle::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("Direction provider coverage for hit-testing and handle affordances.")
        .test_id_prefix("ui-gallery-resizable-rtl")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn/Base UI Resizable docs path after collapsing the top `ComponentPreview` into `Demo` and skipping `Installation`: `Demo`, `About`, `Usage`, `Vertical`, `Handle`, `RTL`, and `API Reference`. `Notes` stays as the explicit Fret follow-up for parity conclusions and diagnostics anchors.",
        ),
        vec![
            demo,
            about,
            usage,
            vertical,
            handle,
            rtl,
            api_reference,
            notes,
        ],
    );

    let component = body.test_id("ui-gallery-resizable").into_element(cx);
    let page = ui::v_flex(move |_cx| vec![component])
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .items_start();

    vec![page.into_element(cx)]
}
