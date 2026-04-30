# Fret Examples Build Latency v1 - M23 IMUI Editor Proof Source Gate - 2026-04-30

Status: complete

## Decision

Move three source-only IMUI editor proof marker checks out of the monolithic `fret-examples`
Rust unit-test module and into `tools/gate_imui_facade_teaching_source.py`.

## Migrated Checks

- `imui_editor_proof_non_raw_helpers_prefer_typed_return_signatures`
- `imui_editor_proof_authoring_immediate_column_uses_official_editor_adapters`
- `imui_editor_proof_keeps_app_owned_sortable_and_dock_helpers_explicit`

## Behavior

The IMUI facade/teaching source gate now covers:

- typed return signatures for non-raw editor proof helpers,
- official `fret-ui-editor` IMUI adapter usage in the authoring parity immediate column,
- app-owned sortable/dock helper markers and the matching app-owner audit note,
- exact `AnyElement` return-count coverage for the proof-local compact readout leaf helper.

No runtime behavior change is intended. This slice only moves pure source/document scanning away
from Rust unit tests that require compiling `fret-examples`.

## Evidence

- `tools/gate_imui_facade_teaching_source.py`
- `apps/fret-examples/src/lib.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/IMUI_EDITOR_PROOF_APP_OWNER_AUDIT_2026-04-16.md`

## Result

The Rust `#[test]` count in `apps/fret-examples/src/lib.rs` dropped from 66 to 63.

## Gates

```text
python tools/gate_imui_facade_teaching_source.py
python -m py_compile tools/gate_imui_facade_teaching_source.py
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
