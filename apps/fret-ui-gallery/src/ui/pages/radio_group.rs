use super::super::*;
use fret::AppComponentCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::radio_group as snippets;

pub(super) fn preview_radio_group(cx: &mut AppComponentCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let description = snippets::description::render(cx);
    let choice_card = snippets::choice_card::render(cx);
    let fieldset = snippets::fieldset::render(cx);
    let disabled = snippets::disabled::render(cx);
    let invalid = snippets::invalid::render(cx);
    let rtl = snippets::rtl::render(cx);
    let label = snippets::label::render(cx);

    let api_reference = doc_layout::notes_block([
        "`radio_group_uncontrolled(default, items)` and `radio_group(model, items)` remain the compact quick-start helpers for uncontrolled and controlled radio-group authoring.",
        "`RadioGroup::into_element_parts(cx, |cx, parts| ...)` is the typed docs-parity seam for rows that need external `Field`, `Label`, `FieldLabel::for_control(...)`, or `FieldDescription` composition around the radio control.",
        "`RadioGroup::required(true)` now marks the group root as required, matching the upstream group-level required semantics instead of scattering the state across individual radio items.",
        "`RadioGroupItem::child(...)` / `children(...)` and `variant(RadioGroupItemVariant::ChoiceCard)` remain recipe-owned shorthands for full-row content overrides, but the docs-path rows on this page now prefer `into_element_parts(...)` + `parts.control(...)`.",
        "Selection semantics, roving navigation, icon chrome, border, and focus ring remain recipe-owned; surrounding fieldset and row layout remain caller-owned composition, so a generic root children API is still unnecessary here.",
        "The `RTL` preview keeps the translated upstream three-row example shape. `DirectionProvider(Rtl)` plus `into_element_parts(...)`, `Field`, and `FieldContent` keep the label/description on the logical side and the indicator on the opposite edge without extra physical alignment props.",
        "`Label Association` stays after the upstream docs path because it documents the Fret-specific `control_id(...)` bridge rather than an upstream section heading.",
        "This page is docs/public-surface parity work, not a mechanism-layer fix.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .test_id_prefix("ui-gallery-radio-group-api-reference")
        .description("Public surface summary and ownership notes.");
    let demo = DocSection::build(cx, "Demo", demo)
        .test_id_prefix("ui-gallery-radio-group-demo")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .description("Copyable docs-shaped row composition via `into_element_parts(...)` and `parts.control(...)`.")
        .test_id_prefix("ui-gallery-radio-group-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let description = DocSection::build(cx, "Description", description)
        .description("Radio group rows with external field parts and label forwarding.")
        .test_id_prefix("ui-gallery-radio-group-description")
        .code_rust_from_file_region(snippets::description::SOURCE, "example");
    let choice_card = DocSection::build(cx, "Choice Card", choice_card)
        .description("Clickable card-style radio selection using `FieldLabel::wrap(...)` around a composed row.")
        .test_id_prefix("ui-gallery-radio-group-choice-card")
        .code_rust_from_file_region(snippets::choice_card::SOURCE, "example");
    let fieldset = DocSection::build(cx, "Fieldset", fieldset)
        .description(
            "Use `FieldSet` and `FieldLegend` to group radio items with a label and description.",
        )
        .test_id_prefix("ui-gallery-radio-group-fieldset")
        .code_rust_from_file_region(snippets::fieldset::SOURCE, "example");
    let disabled = DocSection::build(cx, "Disabled", disabled)
        .description("Use `disabled(true)` on `RadioGroupItem` to disable individual items.")
        .test_id_prefix("ui-gallery-radio-group-disabled")
        .code_rust_from_file_region(snippets::disabled::SOURCE, "example");
    let invalid = DocSection::build(cx, "Invalid", invalid)
        .description("Use `RadioGroupItem::aria_invalid(true)` on each item and caller-owned `Field::invalid(true)` on each composed row for validation styling.")
        .test_id_prefix("ui-gallery-radio-group-invalid")
        .code_rust_from_file_region(snippets::invalid::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("Translated upstream three-row RTL preview on the `into_element_parts(...)` docs-parity lane.")
        .test_id_prefix("ui-gallery-radio-group-rtl")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let label = DocSection::build(cx, "Label Association (Fret)", label)
        .description("Use `FieldLabel::for_control` plus `RadioGroup::control_id` on top of `radio_group(model, items)` to focus the active item on label click.")
        .test_id_prefix("ui-gallery-radio-group-label")
        .code_rust_from_file_region(snippets::label::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Radio Group docs path first: Demo, Usage, Description, Choice Card, Fieldset, Disabled, Invalid, RTL, and API Reference. The docs-path rows now use `into_element_parts(...)` for source-shaped composition, while `Label Association` stays as a focused Fret follow-up.",
        ),
        vec![
            demo,
            usage,
            description,
            choice_card,
            fieldset,
            disabled,
            invalid,
            rtl,
            api_reference,
            label,
        ],
    );

    vec![body.test_id("ui-gallery-radio-group").into_element(cx)]
}
