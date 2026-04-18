use super::super::*;
use fret::AppComponentCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::toggle as snippets;

pub(super) fn preview_toggle(cx: &mut AppComponentCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let outline = snippets::outline::render(cx);
    let with_text = snippets::with_text::render(cx);
    let size = snippets::size::render(cx);
    let disabled = snippets::disabled::render(cx);
    let rtl = snippets::rtl::render(cx);
    let children = snippets::children::render(cx);
    let label = snippets::label::render(cx);

    let api_reference = doc_layout::notes_block([
        "`toggle_uncontrolled(cx, false, |cx| ..)` and `toggle(cx, model, |cx| ..)` are the default first-party entry points; `variant(...)`, `size(...)`, `disabled(...)`, and `a11y_label(...)` cover the documented control surface.",
        "`toggle_uncontrolled(cx, ..)` and `toggle(cx, ..)` already provide the composable-children lane for source-shaped examples, so `Toggle::children([...])` stays the landed-content follow-up instead of widening the root API further.",
        "`Toggle::uncontrolled(...).children([...])` is the landed-element equivalent of upstream JSX child content when callers already own or want to reuse the inner content.",
        "`children([...])`, `leading_icon(...)`, and `label(...)` remain recipe-level content choices; the helper family simply keeps the common path builder-preserving.",
        "Toggle chrome, size presets, horizontal padding, and pressed-state colors remain recipe-owned because the upstream component source defines those defaults on the component itself.",
        "Surrounding toolbar layout, wrapping behavior, and label-to-control wiring remain caller-owned composition choices.",
        "Pressed semantics, keyboard activation, and focus-visible treatment are already covered by the existing toggle semantics/chrome gates; the remaining parity work here is docs/public-surface alignment rather than a `fret-ui` mechanism gap.",
        "No extra generic `asChild` / `compose()` surface is needed here: `children([...])` already covers the composable content story without widening the primitive contract.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .test_id_prefix("ui-gallery-toggle-api-reference")
        .description("Public surface summary and ownership notes.");
    let demo = DocSection::build(cx, "Demo", demo)
        .description("Default demo matching the upstream top-of-page preview.")
        .test_id_prefix("ui-gallery-toggle-demo")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .description(
            "Copyable minimal usage for `Toggle` using the builder-preserving helper family.",
        )
        .test_id_prefix("ui-gallery-toggle-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let outline = DocSection::build(cx, "Outline", outline)
        .description("Use `variant = outline` for stronger boundaries in dense toolbars.")
        .test_id_prefix("ui-gallery-toggle-outline")
        .code_rust_from_file_region(snippets::outline::SOURCE, "example");
    let with_text = DocSection::build(cx, "With Text", with_text)
        .description("Default variant with icon + text content.")
        .test_id_prefix("ui-gallery-toggle-with-text")
        .code_rust_from_file_region(snippets::with_text::SOURCE, "example");
    let size = DocSection::build(cx, "Size", size)
        .description("Size presets: `sm`, `default`, and `lg`.")
        .test_id_prefix("ui-gallery-toggle-size")
        .code_rust_from_file_region(snippets::size::SOURCE, "example");
    let disabled = DocSection::build(cx, "Disabled", disabled)
        .description("Disabled toggles stay readable while blocking interaction.")
        .test_id_prefix("ui-gallery-toggle-disabled")
        .code_rust_from_file_region(snippets::disabled::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("Toggle content order and alignment under RTL.")
        .test_id_prefix("ui-gallery-toggle-rtl")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let children = DocSection::build(cx, "Children (Fret)", children)
        .description(
            "Use the direct landed-element `children([...])` lane when the inner content is already built or caller-owned.",
        )
        .test_id_prefix("ui-gallery-toggle-children")
        .code_rust_from_file_region(snippets::children::SOURCE, "example");
    let label = DocSection::build(cx, "Label Association", label)
        .description(
            "Use `FieldLabel::for_control` plus `Toggle::control_id` when you want an explicit Fret label-click example outside the upstream docs path.",
        )
        .test_id_prefix("ui-gallery-toggle-label")
        .code_rust_from_file_region(snippets::label::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Toggle docs path after collapsing the top `ComponentPreview` into `Demo` and skipping `Installation`: `Demo`, `Usage`, `Outline`, `With Text`, `Size`, `Disabled`, and `RTL`. `Children (Fret)`, `Label Association`, and `API Reference` stay as explicit Fret follow-ups.",
        ),
        vec![
            demo,
            usage,
            outline,
            with_text,
            size,
            disabled,
            rtl,
            children,
            label,
            api_reference,
        ],
    );

    vec![body.test_id("ui-gallery-toggle").into_element(cx)]
}
