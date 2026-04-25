# ImUi Edit Lifecycle Hardening v1 - Closeout Audit - 2026-04-25

## Verdict

Close `imui-edit-lifecycle-hardening-v1` as a shipped hardening lane. The lane proved the current
Dear ImGui-aligned value-edit lifecycle target without widening `crates/fret-ui`,
`fret-authoring::Response`, or public `fret-imui` contracts by default.

## Shipped Surface

- `DragValueCore` now commits only after a session with at least one meaningful value change, while
  still treating change-away-then-return as an edited session.
- Retained node portal text/number editors and public IMUI single-line text input keep stable field
  bounds through focus and edit.
- `BoundTextInput` and `BoundTextArea` now keep controlled text buffers synchronized with model
  revisions before semantics/diagnostics publish values.
- `NumericInput` reset, validation, and Escape-cancel outcomes are rendered proof, not only model
  assumptions.
- The editor-proof suite now covers drag-value outcomes, text/numeric baseline policy,
  numeric-input validation/reset, and numeric-input Escape-cancel.

## Evidence

- `docs/workstreams/imui-edit-lifecycle-hardening-v1/M1_DRAG_VALUE_CORE_SLICE_2026-04-24.md`
- `docs/workstreams/imui-edit-lifecycle-hardening-v1/M2_PORTAL_INPUT_STABILITY_SLICE_2026-04-25.md`
- `docs/workstreams/imui-edit-lifecycle-hardening-v1/M2_IMUI_INPUT_STABILITY_SLICE_2026-04-25.md`
- `docs/workstreams/imui-edit-lifecycle-hardening-v1/M3_IMUI_INPUT_BOUNDS_DIAG_GATE_2026-04-25.md`
- `docs/workstreams/imui-edit-lifecycle-hardening-v1/M3_NUMERIC_INPUT_RENDERED_PROOF_2026-04-25.md`
- `docs/workstreams/imui-edit-lifecycle-hardening-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-edit-lifecycle-hardening-v1/WORKSTREAM.json`

## Final Gate Set

The canonical gate list lives in `EVIDENCE_AND_GATES.md`. The final slice verified:

```bash
cargo nextest run -p fret-ui deferred_dirty_sync_does_not_consume_model_revision --jobs 2
cargo nextest run -p fret-ui forced_sync_applies_model_revision_even_when_dirty --jobs 2
cargo fmt --package fret-ui --package fret-ui-editor --check
cargo check -p fret-ui -p fret-ui-editor -p fret-examples --jobs 2
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-editor/imui/imui-editor-proof-numeric-input-validation.json --launch -- cargo run -p fret-demo --bin imui_editor_proof_demo
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-editor/imui/imui-editor-proof-numeric-input-escape-cancel.json --launch -- cargo run -p fret-demo --bin imui_editor_proof_demo
cargo run -p fretboard-dev -- diag registry check
cargo run -p fretboard-dev -- diag suite imui-editor-proof-edit-outcomes --launch -- cargo run -p fret-demo --bin imui_editor_proof_demo
python -m json.tool tools/diag-scripts/ui-editor/imui/imui-editor-proof-numeric-input-validation.json
python -m json.tool tools/diag-scripts/ui-editor/imui/imui-editor-proof-numeric-input-escape-cancel.json
python -m json.tool tools/diag-scripts/suites/imui-editor-proof-edit-outcomes/suite.json
python -m json.tool tools/diag-scripts/index.json
python -m json.tool docs/workstreams/imui-edit-lifecycle-hardening-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
git diff --check
```

## Residual Gap Routing

- Public `ResponseExt` widening, global item-state APIs, or `fret-authoring::Response` changes need
  a new contract-first lane with ADR evidence.
- Key-owner and shortcut ownership stay out of this lane; use the key-owner closeout records as the
  starting point for any future `SetItemKeyOwner()`-class work.
- Docking, tear-out, multi-window, runner/backend hand-feel, and broader editor-workbench product
  polish belong in narrower product or platform lanes.
- Further IMUI controls should first prove a concrete mismatch in a first-party demo or focused unit
  test before growing shared helpers.
