use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::radio_group as snippets;

pub(super) fn preview_radio_group(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let description = snippets::description::render(cx);
    let choice_card = snippets::choice_card::render(cx);
    let fieldset = snippets::fieldset::render(cx);
    let disabled = snippets::disabled::render(cx);
    let invalid = snippets::invalid::render(cx);
    let rtl = snippets::rtl::render(cx);
    let label = snippets::label::render(cx);

    let api_reference = doc_layout::notes(
        cx,
        [
            "`RadioGroup::uncontrolled(default)` and `RadioGroup::new(model)` cover the documented uncontrolled and controlled authoring paths.",
            "`RadioGroupItem::children(...)` and `variant(RadioGroupItemVariant::ChoiceCard)` cover the richer description and choice-card compositions without introducing a generic `compose()` API.",
            "Selection semantics, roving navigation, icon chrome, border, and focus ring remain recipe-owned; surrounding fieldset and row layout remain caller-owned composition.",
            "`Label Association` stays after the upstream docs path because it documents the Fret-specific `control_id(...)` bridge rather than an upstream section heading.",
            "This page is docs/public-surface parity work, not a mechanism-layer fix.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Radio Group docs path first: Demo, Usage, Description, Choice Card, Fieldset, Disabled, Invalid, RTL, and API Reference. `Label Association` stays as a focused Fret follow-up.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .test_id_prefix("ui-gallery-radio-group-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .description("Copyable minimal usage for `RadioGroup`.")
                .test_id_prefix("ui-gallery-radio-group-usage")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Description", description)
                .description("Radio group items with a description using field parts.")
                .test_id_prefix("ui-gallery-radio-group-description")
                .code_rust_from_file_region(snippets::description::SOURCE, "example"),
            DocSection::new("Choice Card", choice_card)
                .description("Clickable card-style radio selection.")
                .test_id_prefix("ui-gallery-radio-group-choice-card")
                .code_rust_from_file_region(snippets::choice_card::SOURCE, "example"),
            DocSection::new("Fieldset", fieldset)
                .description("Use `FieldSet` and `FieldLegend` to group radio items with a label and description.")
                .test_id_prefix("ui-gallery-radio-group-fieldset")
                .code_rust_from_file_region(snippets::fieldset::SOURCE, "example"),
            DocSection::new("Disabled", disabled)
                .description("Use `disabled(true)` on `RadioGroup` to disable all items.")
                .test_id_prefix("ui-gallery-radio-group-disabled")
                .code_rust_from_file_region(snippets::disabled::SOURCE, "example"),
            DocSection::new("Invalid", invalid)
                .description("Use `aria_invalid(true)` on each item and `Field::invalid(true)` on the composed row for validation styling.")
                .test_id_prefix("ui-gallery-radio-group-invalid")
                .code_rust_from_file_region(snippets::invalid::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("Radio group layout and content order under RTL.")
                .test_id_prefix("ui-gallery-radio-group-rtl")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("API Reference", api_reference)
                .no_shell()
                .test_id_prefix("ui-gallery-radio-group-api-reference")
                .description("Public surface summary and ownership notes."),
            DocSection::new("Label Association (Fret)", label)
                .description("Use `FieldLabel::for_control` plus `RadioGroup::control_id` to focus the active item on label click.")
                .test_id_prefix("ui-gallery-radio-group-label")
                .code_rust_from_file_region(snippets::label::SOURCE, "example"),
        ],
    );

    vec![body.test_id("ui-gallery-radio-group")]
}
