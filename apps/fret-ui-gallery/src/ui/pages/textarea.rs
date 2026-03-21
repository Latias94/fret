use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::textarea as snippets;

pub(super) fn preview_textarea(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let label = snippets::label::render(cx);
    let disabled = snippets::disabled::render(cx);
    let field = snippets::field::render(cx);
    let with_text = snippets::with_text::render(cx);
    let invalid = snippets::invalid::render(cx);
    let button = snippets::button::render(cx);
    let rtl = snippets::rtl::render(cx);

    let api_reference = doc_layout::notes_block([
        "`Textarea::new(model)` is the Fret equivalent of the upstream `<Textarea />`; `placeholder(...)`, `disabled(...)`, `aria_invalid(...)`, `min_height(...)`, and `rows(...)` cover the documented core surface.",
        "`Textarea` keeps control chrome, root `w-full min-w-0`, minimum height, and resize-handle behavior recipe-owned; surrounding width caps and form layout stay caller-owned.",
        "`Field::build(...)` is the focused Fret lane for docs-path label/description association, while explicit `control_id(...)` plus `FieldLabel::for_control(...)` stays in the separate `Label Association` follow-up.",
        "No extra generic `compose()` / `asChild` surface is needed here: upstream composition happens around the textarea via `Field` or simple stacked layout, and Fret already matches that layering.",
        "Placeholder text is shown when the model is empty, `rows(...)` raises the initial height without lowering the default 64px minimum, and the resize affordance is available unless the textarea is disabled.",
    ]);
    let notes = doc_layout::notes_block([
        "This page mirrors the upstream Textarea docs first, with `Field` and `RTL` examples now staying closer to the upstream feedback copy and four-row composition.",
        "Reach for plain `Textarea` when you only need multi-line editing; compose around it with `Field` parts or stacked actions instead of widening the textarea surface itself.",
        "The most fragile outcomes here are resize-handle behavior, placeholder visibility, rows-driven initial height, and label/control wiring, so stable `ui-gallery-textarea-*` ids are part of the regression surface.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .test_id_prefix("ui-gallery-textarea-api-reference")
        .description("Public surface summary and ownership notes.");
    let notes = DocSection::build(cx, "Notes", notes)
        .no_shell()
        .test_id_prefix("ui-gallery-textarea-notes")
        .description("Usage guidance and parity notes.");
    let demo = DocSection::build(cx, "Demo", demo)
        .description("Default textarea preview matching the upstream top-of-page demo.")
        .test_id_prefix("ui-gallery-textarea-demo")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .description("Copyable minimal usage for `Textarea`.")
        .test_id_prefix("ui-gallery-textarea-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let field = DocSection::build(cx, "Field", field)
        .description("Field composition with inherited label/description association and a four-row initial height.")
        .test_id_prefix("ui-gallery-textarea-field")
        .code_rust_from_file_region(snippets::field::SOURCE, "example");
    let disabled = DocSection::build(cx, "Disabled", disabled)
        .description("Disabled textareas block input and keep muted chrome.")
        .test_id_prefix("ui-gallery-textarea-disabled")
        .code_rust_from_file_region(snippets::disabled::SOURCE, "example");
    let invalid = DocSection::build(cx, "Invalid", invalid)
        .description("Invalid state uses `aria_invalid` and field-level error styling.")
        .test_id_prefix("ui-gallery-textarea-invalid")
        .code_rust_from_file_region(snippets::invalid::SOURCE, "example");
    let button = DocSection::build(cx, "Button", button)
        .description("Pair the textarea with a submit button in a stacked action layout.")
        .test_id_prefix("ui-gallery-textarea-button")
        .code_rust_from_file_region(snippets::button::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("Feedback-style textarea composition under an RTL direction provider.")
        .test_id_prefix("ui-gallery-textarea-rtl")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let with_text = DocSection::build(cx, "With Text", with_text)
        .description("A helper-text composition that stays as a Fret follow-up example.")
        .test_id_prefix("ui-gallery-textarea-with-text")
        .code_rust_from_file_region(snippets::with_text::SOURCE, "example");
    let label = DocSection::build(cx, "Label Association", label)
        .description(
            "Use `FieldLabel::for_control` plus `Textarea::control_id` so label clicks focus the textarea and preserve labelled-by semantics.",
        )
        .test_id_prefix("ui-gallery-textarea-label")
        .code_rust_from_file_region(snippets::label::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Textarea docs path first: Demo, Usage, Field, Disabled, Invalid, Button, RTL, and API Reference. The docs-path `Field` and `RTL` examples now stay closer to the upstream feedback copy and four-row composition, while `With Text` and `Label Association` stay as focused Fret follow-ups.",
        ),
        vec![
            demo,
            usage,
            field,
            disabled,
            invalid,
            button,
            rtl,
            api_reference,
            with_text,
            label,
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-textarea").into_element(cx)]
}
