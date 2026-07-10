# M3 Feature Payload Store Module Split

Status: Landed
Date: 2026-05-12

## Decision

The editor feature payload store and its public diagnostics snapshot type moved out of
`ecosystem/fret-code-editor/src/editor/mod.rs` into
`ecosystem/fret-code-editor/src/editor/feature_payloads.rs`.

The public API stays unchanged: `CodeEditorFeaturePayloadSnapshot` remains exported from the crate
root through `fret-code-editor/src/lib.rs`. The split narrows the owner boundary for diagnostics,
decorations, gutter markers, semantic tokens, display-map pruning, and payload epochs without
changing handle methods or payload semantics.

## Coverage

The new module owns:

- `CodeEditorFeaturePayloadSnapshot`,
- `CodeEditorFeaturePayloadStore`,
- payload epoch readout,
- diagnostic line summary readout,
- buffer-change clearing,
- display-map gutter marker retention,
- idempotent setter storage,
- diagnostics snapshot counts.

`editor/mod.rs` now consumes the store through methods instead of reading store fields directly.
The remaining large-owner candidates are still `state/handle`, `input`, `paint`, `syntax`, `a11y`,
and diagnostics/decorations call-site ownership.

## Non-Goals

This slice does not change feature payload contracts, visual rendering, diagnostics bundle schema,
or public handle method names.

## Follow-up

The diagnostics/decorations call-site ownership item was later closed by
`M3_DIAGNOSTICS_DECORATIONS_OWNER_CLOSURE_2026-05-12.md`. The feature payload store remains the
widget-local storage owner; the view crate remains the data-contract owner.

## Evidence

- New owner module: `ecosystem/fret-code-editor/src/editor/feature_payloads.rs`
- Surface integration: `ecosystem/fret-code-editor/src/editor/mod.rs`
- Paint cache epoch integration: `ecosystem/fret-code-editor/src/editor/paint/mod.rs`
- Public surface re-export: `ecosystem/fret-code-editor/src/lib.rs`
- Focused feature tests:
  `ecosystem/fret-code-editor/src/editor/tests/feature_payloads.rs`

## Gates

```powershell
cargo check -p fret-code-editor --features syntax-rust
cargo nextest run -p fret-code-editor feature_payload --no-fail-fast
cargo nextest run -p fret-code-editor --test public_surface --no-fail-fast
```
