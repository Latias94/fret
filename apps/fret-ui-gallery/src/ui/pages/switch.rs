use super::super::*;
use fret::AppComponentCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::switch as snippets;

pub(super) fn preview_switch(cx: &mut AppComponentCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::airplane_mode::render(cx);
    let usage = snippets::usage::render(cx);
    let description = snippets::description::render(cx);
    let choice_card = snippets::choice_card::render(cx);
    let disabled = snippets::disabled::render(cx);
    let read_only = snippets::read_only::render(cx);
    let command_gate = snippets::command_gate::render(cx);
    let invalid = snippets::invalid::render(cx);
    let sizes = snippets::sizes::render(cx);
    let rtl = snippets::rtl::render(cx);
    let label = snippets::label::render(cx);
    let style_override = snippets::bluetooth::render(cx);

    let api_reference = doc_layout::notes_block([
        "Reference stack: current shadcn Switch docs and new-york-v4 source, with current new-york-v4 Field/Form examples plus Base/Radix registry examples as secondary references.",
        "Current docs path stays `Demo` and `Usage`; richer field/form/base examples stay labeled as registry or Fret follow-ups instead of being treated as current shadcn docs-path sections.",
        "`Switch::new(model)` is the Fret equivalent of the upstream `<Switch />`; `size(...)`, `disabled(...)`, `read_only(...)`, `required(...)`, `aria_invalid(...)`, `control_id(...)`, and `a11y_label(...)` cover the documented control surface, while `Label::for_control(...)` / `FieldLabel::for_control(...)` carry the `htmlFor` teaching path.",
        "`Switch::required(true)` keeps required semantics on the root control surface; surrounding `Field` / label composition stays caller-owned and does not need a wrapper-level required API.",
        "Track/thumb chrome and the intrinsic switch sizes remain recipe-owned because the upstream component source defines those defaults on the switch itself.",
        "Caller-owned layout stays explicit for `max-w-*`, stacked field groups, and surrounding page/grid negotiation; the recipe should not absorb those constraints.",
        "Switch remains a leaf control surface: `Label::for_control(...)` covers the inline `Demo` / `Size` rows, and `FieldLabel::for_control(...)` plus `FieldLabel::wrap(...)` cover field, registry, and card-style compositions without widening `Switch` into a generic children API.",
        "Radix/Base UI expose `Root` / `Thumb` parts for DOM composition, but the self-drawn shadcn recipe keeps thumb painting internal here; if a future product truly needs custom parts, that belongs in a lower-level primitive/raw surface rather than the copyable docs-path `Switch`.",
        "The `RTL (Fret)` preview translates the registry-style one-row field example. `DirectionProvider(Rtl)` is sufficient here: `FieldContent` stays on the logical text side and `Switch` stays on the opposite edge without teaching an extra physical alignment prop.",
        "`SwitchStyle` remains a focused Fret follow-up for token-safe track/thumb overrides; it is not part of the upstream docs path, but it is the right escape hatch when product recipes need custom checked colors.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference (Fret)", api_reference)
        .no_shell()
        .test_id_prefix("ui-gallery-switch-api-reference")
        .description("Public surface summary and ownership notes.");

    let demo = DocSection::build(cx, "Demo", demo)
        .description("Default switch preview matching the upstream top-of-page demo.")
        .test_id_prefix("ui-gallery-switch-demo")
        .code_rust_from_file_region(snippets::airplane_mode::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .description("Copyable minimal usage for `Switch`.")
        .test_id_prefix("ui-gallery-switch-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let description = DocSection::build(cx, "Description (Registry)", description)
        .description("Field composition mirroring the current new-york-v4 `field-switch` registry example with label and helper text beside the switch.")
        .test_id_prefix("ui-gallery-switch-description")
        .code_rust_from_file_region(snippets::description::SOURCE, "example");
    let choice_card = DocSection::build(cx, "Choice Card (Fret)", choice_card)
        .description("Fret follow-up for clickable card-style field labels that forward activation to the switch.")
        .test_id_prefix("ui-gallery-switch-choice-card")
        .code_rust_from_file_region(snippets::choice_card::SOURCE, "example");
    let disabled = DocSection::build(cx, "Disabled (Base/Radix)", disabled)
        .description("Base/Radix registry disabled switch example; disabled switches block interaction and let the surrounding field own the muted state.")
        .test_id_prefix("ui-gallery-switch-disabled")
        .code_rust_from_file_region(snippets::disabled::SOURCE, "example");
    let read_only = DocSection::build(cx, "Read Only (Fret)", read_only)
        .description("Read-only switches remain observable and focusable while suppressing value changes and invoke semantics.")
        .test_id_prefix("ui-gallery-switch-read-only")
        .code_rust_from_file_region(snippets::read_only::SOURCE, "example");
    let command_gate = DocSection::build(cx, "Command Gate (Fret)", command_gate)
        .description("Diagnostics harness surface for external command availability changing a switch action across frames.")
        .test_id_prefix("ui-gallery-switch-command-gate")
        .code_rust_from_file_region(snippets::command_gate::SOURCE, "example");
    let invalid = DocSection::build(cx, "Invalid (Registry)", invalid)
        .description(
            "Invalid state uses root `Switch::aria_invalid(true)` on the control and caller-owned `Field::invalid(true)` on the composition. Current new-york-v4 Form switch examples remain the registry reference for this lane.",
        )
        .test_id_prefix("ui-gallery-switch-invalid")
        .code_rust_from_file_region(snippets::invalid::SOURCE, "example");
    let sizes = DocSection::build(cx, "Size (Base/Radix)", sizes)
        .description("Base/Radix registry size presets: `sm` and `default`.")
        .test_id_prefix("ui-gallery-switch-size")
        .code_rust_from_file_region(snippets::sizes::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL (Fret)", rtl)
        .description("Translated registry-style row with logical field text on inline-start and the switch on the opposite edge.")
        .test_id_prefix("ui-gallery-switch-rtl")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let label = DocSection::build(cx, "Label Association (Fret)", label)
        .description(
            "Use `FieldLabel::for_control` plus `Switch::control_id` when you want a focused, non-card label-click example outside the upstream docs path.",
        )
        .test_id_prefix("ui-gallery-switch-label")
        .code_rust_from_file_region(snippets::label::SOURCE, "example");
    let style_override = DocSection::build(cx, "Style Override (Fret)", style_override)
        .description("A Fret-specific follow-up showing `SwitchStyle` for custom checked-track color overrides.")
        .test_id_prefix("ui-gallery-switch-style")
        .code_rust_from_file_region(snippets::bluetooth::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the current shadcn Switch docs path first: `Demo` and `Usage`. `Description (Registry)` and `Invalid (Registry)` follow current new-york-v4 Field/Form examples; `Disabled (Base/Radix)` and `Size (Base/Radix)` follow Base/Radix registry examples; `Choice Card (Fret)`, `Read Only (Fret)`, `Command Gate (Fret)`, `RTL (Fret)`, `Label Association (Fret)`, `Style Override (Fret)`, and `API Reference (Fret)` stay as focused Fret follow-ups.",
        ),
        vec![
            demo,
            usage,
            description,
            choice_card,
            disabled,
            read_only,
            command_gate,
            invalid,
            sizes,
            rtl,
            label,
            style_override,
            api_reference,
        ],
    );

    vec![body.test_id("ui-gallery-switch").into_element(cx)]
}
