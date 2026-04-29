# ImUi TextField Draft Controller API Proof v1 - Milestones

Status: closed narrow P1 lane
Last updated: 2026-04-29

## M0 - Lane Opened

Status: complete

- Started from the closed `imui-textfield-draft-buffer-contract-audit-v1` no-public-API verdict.
- Scoped the follow-on to an opaque operation controller instead of a public draft model handle.

## M1 - Opaque Draft Controller

Status: complete

Goal: land `TextFieldDraftController`, bind it from buffered `TextField`, and prove external
commit/discard in `editor_notes_demo.rs`.

Exit criteria:

- Controller commit/discard use the same internal session semantics as the field's built-in actions.
- `editor_notes_demo.rs` shows real `Commit draft` / `Discard draft` controls.
- Source-policy tests keep runtime and generic IMUI helper widening out of scope.

Evidence: `ecosystem/fret-ui-editor/src/controls/text_field.rs`,
`apps/fret-examples/src/editor_notes_demo.rs`,
`ecosystem/fret-ui-editor/tests/text_field_api_smoke.rs`, and
`apps/fret-examples/tests/editor_notes_editor_rail_surface.rs`.

## M2 - Closeout Decision

Status: complete

Goal: decide whether the opaque controller is enough for v1 or whether another follow-on is needed
for persistence, dirty-close, command-bus, or document-state integration.

Exit criteria:

- A launched diagnostics script proves `Commit draft` / `Discard draft` through stable editor-notes
  selectors.
- The proof records committed line count, last action, draft status, and app-owned status-row
  outcomes.
- The lane closes without adding persistence, dirty-close, command-bus, document-state, runtime, or
  generic IMUI helper surface.

Evidence: `tools/diag-scripts/ui-editor/editor-notes-demo/editor-notes-demo-draft-controller-proof.json`,
`docs/workstreams/imui-textfield-draft-controller-api-proof-v1/CLOSEOUT_AUDIT_2026-04-29.md`, and
the packed local diagnostics artifact recorded in `EVIDENCE_AND_GATES.md`.
