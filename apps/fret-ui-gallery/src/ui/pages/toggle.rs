use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::toggle as snippets;

pub(super) fn preview_toggle(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let outline = snippets::outline::render(cx);
    let with_text = snippets::with_text::render(cx);
    let size = snippets::size::render(cx);
    let disabled = snippets::disabled::render(cx);
    let rtl = snippets::rtl::render(cx);
    let label = snippets::label::render(cx);

    let api_reference = doc_layout::notes_block([
        "`Toggle::uncontrolled(false)` mirrors the upstream `<Toggle />` quick-start path; `variant(...)`, `size(...)`, `disabled(...)`, and `a11y_label(...)` cover the documented control surface.",
        "`children([...])` is the source-aligned Fret equivalent of upstream child content, while `label(...)` remains the ergonomic shortcut for the common icon-plus-text case.",
        "Toggle chrome, size presets, horizontal padding, and pressed-state colors remain recipe-owned because the upstream component source defines those defaults on the component itself.",
        "Surrounding toolbar layout, wrapping behavior, and label-to-control wiring remain caller-owned composition choices.",
        "No extra generic `asChild` / `compose()` surface is needed here: `children([...])` already covers the composable content story without widening the primitive contract.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .test_id_prefix("ui-gallery-toggle-api-reference")
        .description("Public surface summary and ownership notes.");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Toggle docs path first: Demo, Usage, Outline, With Text, Size, Disabled, RTL, then keeps `Label Association` and `API Reference` as focused Fret follow-ups.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("Default demo matching the upstream top-of-page preview.")
                .test_id_prefix("ui-gallery-toggle-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .description("Copyable minimal usage for `Toggle` using child content.")
                .test_id_prefix("ui-gallery-toggle-usage")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Outline", outline)
                .description("Use `variant = outline` for stronger boundaries in dense toolbars.")
                .test_id_prefix("ui-gallery-toggle-outline")
                .code_rust_from_file_region(snippets::outline::SOURCE, "example"),
            DocSection::new("With Text", with_text)
                .description("Default variant with icon + text content.")
                .test_id_prefix("ui-gallery-toggle-with-text")
                .code_rust_from_file_region(snippets::with_text::SOURCE, "example"),
            DocSection::new("Size", size)
                .description("Size presets: `sm`, `default`, and `lg`.")
                .test_id_prefix("ui-gallery-toggle-size")
                .code_rust_from_file_region(snippets::size::SOURCE, "example"),
            DocSection::new("Disabled", disabled)
                .description("Disabled toggles stay readable while blocking interaction.")
                .test_id_prefix("ui-gallery-toggle-disabled")
                .code_rust_from_file_region(snippets::disabled::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("Toggle content order and alignment under RTL.")
                .test_id_prefix("ui-gallery-toggle-rtl")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Label Association", label)
                .description(
                    "Use `FieldLabel::for_control` plus `Toggle::control_id` when you want an explicit Fret label-click example outside the upstream docs path.",
                )
                .test_id_prefix("ui-gallery-toggle-label")
                .code_rust_from_file_region(snippets::label::SOURCE, "example"),
            api_reference,
        ],
    );

    vec![body.test_id("ui-gallery-toggle")]
}
