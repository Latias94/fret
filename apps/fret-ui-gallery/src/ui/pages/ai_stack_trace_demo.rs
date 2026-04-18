use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::{AppComponentCx, UiChild};

pub(super) fn preview_ai_stack_trace_demo(
    cx: &mut AppComponentCx<'_>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let usage = snippets::stack_trace_usage::render(cx);
    let demo = snippets::stack_trace_demo::render(cx);
    let collapsed = snippets::stack_trace_collapsed::render(cx);
    let no_internal = snippets::stack_trace_no_internal::render(cx);
    let features = doc_layout::notes_block([
        "Parses standard JavaScript and Node.js stack trace lines into error type, message, and frames.",
        "Keeps the compound-parts API from the official docs: header, error row, actions, content, and frames stay caller-composable.",
        "Dims internal frames (`node_modules`, `node:` and `internal/`) while keeping user frames prominent.",
        "Supports collapsible content, copy affordance, optional file-path click seams, and stable diagnostics selectors.",
        "Large-list scroll and deep frame click seams stay covered on the dedicated StackTrace Large page.",
    ])
    .test_id("ui-gallery-ai-stack-trace-features");
    let props = stack_trace_props_table(cx).test_id("ui-gallery-ai-stack-trace-props");

    let notes = doc_layout::notes_block([
        "Mechanism health looked good here: the parity issue was in component defaults and the Gallery teaching surface, not in `crates/fret-ui` overlay or routing contracts.",
        "`StackTraceErrorType` no longer fabricates a fallback `\"Error\"` label when the trace lacks an error-type prefix, matching the official component semantics.",
        "Controlled disclosure stays model-driven in Fret: `open(Model<bool>)` is the analogue of upstream `open` + `onOpenChange`, so this remains a component-surface mapping rather than a missing runtime primitive.",
        "`StackTraceCopyButton` now exposes both `on_copy` and `on_error`, and the Gallery seam example reports real clipboard completion instead of optimistic copy intent.",
    ]);

    let usage_section = DocSection::build(cx, "Usage with Tool Errors", usage)
        .description("Rust/Fret analogue of the official AI Elements usage example.")
        .test_id_prefix("ui-gallery-ai-stack-trace-usage")
        .code_rust_from_file_region(snippets::stack_trace_usage::SOURCE, "example");
    let seams_section = DocSection::build(cx, "Copy + File Seams", demo)
        .description("Adds app-owned copy/file-open callbacks on top of the docs-aligned compound structure.")
        .test_id_prefix("ui-gallery-ai-stack-trace-demo")
        .code_rust_from_file_region(snippets::stack_trace_demo::SOURCE, "example");
    let features_section = DocSection::build(cx, "Features", features)
        .description("Official outcomes plus the Fret-owned seams that matter for parity review.")
        .no_shell();
    let collapsed_section = DocSection::build(cx, "Collapsed by Default", collapsed)
        .description("Matches the official collapsed example.")
        .test_id_prefix("ui-gallery-ai-stack-trace-collapsed")
        .code_rust_from_file_region(snippets::stack_trace_collapsed::SOURCE, "example");
    let no_internal_section = DocSection::build(cx, "Hide Internal Frames", no_internal)
        .description("Matches the official no-internal-frames example.")
        .test_id_prefix("ui-gallery-ai-stack-trace-no-internal")
        .code_rust_from_file_region(snippets::stack_trace_no_internal::SOURCE, "example");
    let props_section = DocSection::build(cx, "Props", props)
        .description("Fret builder surface corresponding to the official `StackTrace*` family.");
    let notes_section = DocSection::build(cx, "Notes", notes)
        .description("Layering + parity findings for StackTrace.");

    let body = crate::ui::doc_layout::render_doc_page_after(
        Some(
            "Docs-aligned StackTrace coverage for AI Elements: usage first, then focused seams, feature notes, official examples, props mapping, and parity notes.",
        ),
        vec![
            usage_section,
            seams_section,
            features_section,
            collapsed_section,
            no_internal_section,
            props_section,
            notes_section,
        ],
        cx,
    );

    vec![body.into_element(cx)]
}

fn stack_trace_props_table(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    doc_layout::text_table(
        cx,
        ["Surface", "Method", "Type", "Default", "Description"],
        [
            [
                "StackTrace",
                "new(trace)",
                "impl Into<Arc<str>>",
                "-",
                "Required raw stack trace payload that the component parses into error text and frames.",
            ],
            [
                "StackTrace",
                "into_element_with_children",
                "FnOnce(&mut ElementContext) -> Vec<AnyElement>",
                "-",
                "Primary compound-parts authoring lane for `StackTraceHeader`, `StackTraceContent`, and related parts.",
            ],
            [
                "StackTrace",
                "open / default_open",
                "Model<bool> / bool",
                "None / false",
                "Controlled or uncontrolled disclosure state; Fret uses a model instead of a separate change callback.",
            ],
            [
                "StackTrace",
                "show_internal_frames / max_height",
                "bool / Px",
                "true / 400px",
                "Sets the default frame visibility policy and content viewport height for the root-composed content.",
            ],
            [
                "StackTrace",
                "on_file_path_click",
                "Arc<dyn Fn(...)>",
                "None",
                "App-owned seam for opening a frame location when file paths are clicked.",
            ],
            [
                "StackTrace",
                "test_id_* / frame_test_id_prefix",
                "builder methods",
                "None",
                "Stable diagnostics selectors for the root, header trigger, content, viewport, and per-frame rows.",
            ],
            [
                "StackTraceHeader / Error / Actions",
                "new(children)",
                "IntoIterator<Item = AnyElement>",
                "-",
                "Compose the header row explicitly, matching the official docs structure.",
            ],
            [
                "StackTraceCopyButton",
                "on_copy / on_error / timeout",
                "builder methods",
                "None / None / 2000ms",
                "Observes clipboard write success or failure and controls how long the copied state stays visible.",
            ],
            [
                "StackTraceContent",
                "max_height / viewport_test_id",
                "builder methods",
                "400px / None",
                "Constrains the scrollable content area and exposes the frames viewport for diag scripts.",
            ],
            [
                "StackTraceFrames",
                "show_internal_frames / on_file_path_click",
                "builder methods",
                "inherit / inherit",
                "Overrides root defaults per frames list and lets stand-alone frame lists reuse the same click seam.",
            ],
        ],
        true,
    )
}
