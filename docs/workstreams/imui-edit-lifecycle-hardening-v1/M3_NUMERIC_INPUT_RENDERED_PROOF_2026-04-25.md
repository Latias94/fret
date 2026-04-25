# ImUi Edit Lifecycle Hardening v1 - M3 Numeric Input Rendered Proof - 2026-04-25

## Decision

Promote the existing editor-proof numeric-input diagnostics scripts into the
`imui-editor-proof-edit-outcomes` suite, and fix the underlying controlled text-buffer sync bug
that made reset proof observe a stale rendered value. The correction belongs partly in
`crates/fret-ui` because bound text controls must not consume a model revision until the inner text
buffer actually accepts it. The editor layer still owns numeric-input draft/error policy and
property-row reset identity.

## Root Cause

The failing reset path was not a missing reset callback. The editor model changed from `0.250` back
to `0.750`, and `NumericInput` rendered from the updated value, but `BoundTextInput` could keep an
older internal text buffer because `sync_from_model` consumed `last_revision` before applying the
model text while dirty. The semantics/diagnostics value then read the stale buffer.

## Shipped Invariant

- `BoundTextInput` and `BoundTextArea` now consume a model revision only after the model text is
  applied to the inner text widget.
- Non-focused event, layout, and semantics paths treat the model as the authority, so diagnostics
  and accessibility reads do not publish stale buffers after an external update.
- `NumericInput` only rewrites draft/error state when the observed values actually change, and
  requests a follow-up frame when render-time synchronization changes those models.
- `PropertyRow` builds reset affordances inside the reset slot with keyed identity when available,
  routes `test_id` through pressable accessibility metadata, and notifies the host after reset
  activation.
- The numeric-input validation and Escape-cancel scripts scroll the advanced exposure control into
  view before interaction, which keeps the proof deterministic in the native 720px test window.
- `imui-editor-proof-edit-outcomes` now runs drag-value proof, text/numeric baseline policy proof,
  numeric-input validation proof, and numeric-input Escape-cancel proof as one promoted suite.

## Evidence

- `crates/fret-ui/src/text/input/bound.rs`
- `crates/fret-ui/src/text/area/bound.rs`
- `ecosystem/fret-ui-editor/src/composites/property_row.rs`
- `ecosystem/fret-ui-editor/src/primitives/numeric_text_entry.rs`
- `tools/diag-scripts/ui-editor/imui/imui-editor-proof-numeric-input-validation.json`
- `tools/diag-scripts/ui-editor/imui/imui-editor-proof-numeric-input-escape-cancel.json`
- `tools/diag-scripts/suites/imui-editor-proof-edit-outcomes/suite.json`
- `tools/diag-scripts/index.json`

## Verification

Verified on 2026-04-25:

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

Bundle evidence from the passing runs:

- `target/fret-diag/1777113547959-imui-editor-proof-numeric-input-validation`
- `target/fret-diag/1777114129957-imui-editor-proof-numeric-input-escape-cancel`
- `target/fret-diag/1777114748599-imui-editor-proof-drag-value-outcomes`
- `target/fret-diag/1777114771734-imui-editor-proof-numeric-input-escape-cancel`
- `target/fret-diag/1777114794864-imui-editor-proof-numeric-input-validation`
- `target/fret-diag/1777114836767-imui-editor-proof-text-numeric-baseline-policy`

## Residual Risk

This slice fixes controlled bound text synchronization and the editor numeric-input proof path. It
does not widen public IMUI lifecycle APIs, public `fret-authoring::Response`, or runtime input
contracts. Any future public API pressure should start as a narrower follow-on with separate
contract evidence.
