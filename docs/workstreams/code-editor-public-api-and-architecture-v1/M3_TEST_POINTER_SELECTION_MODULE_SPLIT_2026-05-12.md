# M3 Test Pointer Selection Module Split

Status: Landed
Date: 2026-05-12

## Decision

Move pointer selection, pointer hit-test, shift-click/drag, double-click, triple-click, and
preedit-cancel pointer tests out of `ecosystem/fret-code-editor/src/editor/tests/mod.rs` into
`ecosystem/fret-code-editor/src/editor/tests/pointer_selection.rs`.

## Rationale

Pointer selection behavior is an input owner concern that depends on display-map projection,
row-geometry cache freshness, fold/inlay mapping, inline preedit replacement, and text-boundary
selection. Keeping these tests in the monolithic editor test module made ownership harder to audit.

This split keeps the existing behavior and assertions intact while giving pointer selection its own
test owner next to the smaller pointer helper module.

## Non-Goals

This slice does not change pointer runtime behavior, add new selection gestures, or alter
performance thresholds.

## Evidence

- Pointer selection test owner: `ecosystem/fret-code-editor/src/editor/tests/pointer_selection.rs`
- Parent test module registration: `ecosystem/fret-code-editor/src/editor/tests/mod.rs`
- Pointer selection implementation: `ecosystem/fret-code-editor/src/editor/input/pointer.rs`
- Pointer hit-test implementation: `ecosystem/fret-code-editor/src/editor/mod.rs`

## Gates

```powershell
cargo fmt -p fret-code-editor --check
cargo nextest run -p fret-code-editor pointer --no-fail-fast
cargo check -p fret-code-editor --features syntax-rust
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
```
