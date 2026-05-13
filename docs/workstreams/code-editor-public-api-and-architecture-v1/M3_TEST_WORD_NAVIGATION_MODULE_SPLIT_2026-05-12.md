# M3 Test Word Navigation Module Split

Status: Landed
Date: 2026-05-12

## Decision

Move word deletion and word-boundary navigation tests out of
`ecosystem/fret-code-editor/src/editor/tests/mod.rs` into
`ecosystem/fret-code-editor/src/editor/tests/word_navigation.rs`.

## Rationale

Word movement and word deletion belong to the input navigation boundary, and they depend on the
shared `TextBoundaryMode` semantics from ADR 0179. Keeping these tests in the monolithic editor
test module hid that owner behind unrelated paint, cache, pointer, and platform text-input cases.

This split preserves the same assertions while making word navigation reviewable as a narrow
behavior cluster.

## Non-Goals

This slice does not change runtime text-boundary behavior, command routing, or delete-word
transaction semantics.

## Evidence

- Word navigation test owner: `ecosystem/fret-code-editor/src/editor/tests/word_navigation.rs`
- Parent test module registration: `ecosystem/fret-code-editor/src/editor/tests/mod.rs`
- Input navigation implementation: `ecosystem/fret-code-editor/src/editor/input/navigation.rs`
- Text boundary ADR: `docs/adr/0179-text-navigation-and-word-boundaries-v1.md`

## Gates

```powershell
cargo fmt -p fret-code-editor --check
cargo nextest run -p fret-code-editor word --no-fail-fast
cargo check -p fret-code-editor --features syntax-rust
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
```
