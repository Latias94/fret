# Material3 TextField Semantics And Chrome Packet v2

Date: 2026-05-28
Task: M3PV2-023
Status: Done

## Truth

- Text controls can express declarative `labelled_by` and `described_by` relations without adding
  layout wrappers.
- Material TextField wires its visual label to the input as `labelled_by` and supporting text to
  the input as `described_by` once element ids stabilize.
- TextArea receives the same relation mechanism as TextInput so multiline Material fields do not
  lose the a11y relationship.
- The filled TextField visual harness identifies the actual implementation layers: filled
  container, separate bottom active-indicator canvas, and hover state-layer overlay.

## Sources

- Compose Material3 TextField/OutlinedTextField exposes `label`, `supportingText`, and `isError`
  as first-class field decoration inputs:
  `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/{TextField.kt,OutlinedTextField.kt}`.
- Fret field-family checklist:
  `.agents/skills/fret-material-source-alignment/references/material-field-family-checklist.md`.
- ADR 0033 semantics tree contract:
  `docs/adr/0033-semantics-tree-and-accessibility-bridge.md`.

## Layering

- Mechanism fix: `crates/fret-ui`.
  - `TextInputProps` and `TextAreaProps` now expose declarative `labelled_by_element` and
    `described_by_element` relation targets.
  - The declarative host resolves those element ids into portable semantics relation edges.
- Material recipe fix: `ecosystem/fret-ui-material3`.
  - TextField owns the visual label/supporting-text relationship and wires it into the text control.
- No `fret-ui-kit` policy change was required.

## Artifacts

- `crates/fret-ui/src/element.rs`
- `crates/fret-ui/src/declarative/host_widget/semantics.rs`
- `crates/fret-ui/src/declarative/tests/interactions/text_input.rs`
- `ecosystem/fret-ui-material3/src/text_field.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-ui-material3/tests/text_field_hover.rs`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`

## Proof

```powershell
cargo fmt --package fret-ui --package fret-ui-material3
cargo nextest run -p fret-ui --lib labelled_and_described
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_text_field_exposes_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test text_field_hover
git diff --check
```

Results:

- `fret-ui --lib labelled_and_described`: 2 passed, run id
  `26a1d393-a661-4072-868e-026eae7519ec`.
- `material3_text_field_exposes_stable_part_test_ids`: passed, run id
  `14ce22fc-b89c-4692-9afb-c7ebed566717`.
- `text_field_hover`: 6 passed, run id `0bdc1ccb-4a31-436b-bd11-644a8a446bab`.
- `git diff --check`: no whitespace errors. Git warned that
  `crates/fret-ui/src/declarative/host_widget/semantics.rs` will normalize CRLF to LF when touched.

## Residual Risk

- This packet closes TextField label/supporting-text relation wiring and repairs the filled chrome
  harness. It does not claim full floating-label geometry parity across every state.
- Multiline TextField has the relation mechanism available through TextArea, but this packet did
  not add a dedicated Material multiline scenario beyond the core TextArea relation gate.
