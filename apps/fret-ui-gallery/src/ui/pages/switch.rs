use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::switch as snippets;

pub(super) fn preview_switch(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::airplane_mode::render(cx);
    let usage = snippets::usage::render(cx);
    let description = snippets::description::render(cx);
    let choice_card = snippets::choice_card::render(cx);
    let disabled = snippets::disabled::render(cx);
    let invalid = snippets::invalid::render(cx);
    let sizes = snippets::sizes::render(cx);
    let rtl = snippets::rtl::render(cx);
    let label = snippets::label::render(cx);
    let style_override = snippets::bluetooth::render(cx);

    let api_reference = doc_layout::notes_block([
        "`Switch::new(model)` is the Fret equivalent of the upstream `<Switch />`; `size(...)`, `disabled(...)`, `aria_invalid(...)`, `control_id(...)`, and `a11y_label(...)` now cover the documented control-level surface.",
        "Track/thumb chrome and the intrinsic switch sizes remain recipe-owned because the upstream component source defines those defaults on the switch itself.",
        "Caller-owned layout stays explicit for `max-w-*`, stacked field groups, and surrounding page/grid negotiation; the recipe should not absorb those constraints.",
        "`FieldLabel::for_control(...)` plus `FieldLabel::wrap(...)` covers the source-aligned description and choice-card compositions without widening `Switch` into a generic children API.",
        "`SwitchStyle` remains a focused Fret follow-up for token-safe track/thumb overrides; it is not part of the upstream docs path, but it is the right escape hatch when product recipes need custom checked colors.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .test_id_prefix("ui-gallery-switch-api-reference")
        .description("Public surface summary and ownership notes.");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Switch docs path first: Demo, Usage, Description, Choice Card, Disabled, Invalid, Size, RTL, then keeps `Label Association`, `Style Override`, and `API Reference` as explicit Fret follow-ups.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("Default switch preview matching the upstream top-of-page demo.")
                .test_id_prefix("ui-gallery-switch-demo")
                .code_rust_from_file_region(snippets::airplane_mode::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .description("Copyable minimal usage for `Switch`.")
                .test_id_prefix("ui-gallery-switch-usage")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Description", description)
                .description("Field composition with label and helper text beside the switch.")
                .test_id_prefix("ui-gallery-switch-description")
                .code_rust_from_file_region(snippets::description::SOURCE, "example"),
            DocSection::new("Choice Card", choice_card)
                .description("Clickable card-style field labels that forward activation to the switch.")
                .test_id_prefix("ui-gallery-switch-choice-card")
                .code_rust_from_file_region(snippets::choice_card::SOURCE, "example"),
            DocSection::new("Disabled", disabled)
                .description("Disabled switches block interaction and let the surrounding field own the muted state.")
                .test_id_prefix("ui-gallery-switch-disabled")
                .code_rust_from_file_region(snippets::disabled::SOURCE, "example"),
            DocSection::new("Invalid", invalid)
                .description("Invalid state uses `aria_invalid` on the control and `Field::invalid(true)` on the composition.")
                .test_id_prefix("ui-gallery-switch-invalid")
                .code_rust_from_file_region(snippets::invalid::SOURCE, "example"),
            DocSection::new("Size", sizes)
                .description("Size presets: `sm` and `default`.")
                .test_id_prefix("ui-gallery-switch-size")
                .code_rust_from_file_region(snippets::sizes::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("Switch composition under an RTL direction provider.")
                .test_id_prefix("ui-gallery-switch-rtl")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Label Association", label)
                .description(
                    "Use `FieldLabel::for_control` plus `Switch::control_id` when you want a focused, non-card label-click example outside the upstream docs path.",
                )
                .test_id_prefix("ui-gallery-switch-label")
                .code_rust_from_file_region(snippets::label::SOURCE, "example"),
            DocSection::new("Style Override", style_override)
                .description("A Fret-specific follow-up showing `SwitchStyle` for custom checked-track color overrides.")
                .test_id_prefix("ui-gallery-switch-style")
                .code_rust_from_file_region(snippets::bluetooth::SOURCE, "example"),
            api_reference,
        ],
    );

    vec![body.test_id("ui-gallery-switch")]
}
