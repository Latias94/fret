# M3 Test State Lifecycle Module Split

Status: Landed
Date: 2026-05-12

## Decision

Move state lifecycle tests out of `ecosystem/fret-code-editor/src/editor/tests/mod.rs` into
`ecosystem/fret-code-editor/src/editor/tests/state_lifecycle.rs`.

The extracted tests cover buffer replacement state reset, preservation of text-boundary mode across
buffer replacement, and clearing the text-boundary override.

## Rationale

State lifecycle behavior is a handle/model owner concern. These tests verify state reset and
configuration preservation contracts; keeping them separate from paint, row geometry, navigation,
and platform text-input tests makes the handle boundary easier to audit.

## Non-Goals

This slice does not change buffer replacement behavior, text-boundary mode semantics, cache reset
semantics, or public APIs.

## Evidence

- State lifecycle test owner: `ecosystem/fret-code-editor/src/editor/tests/state_lifecycle.rs`
- Parent test module registration: `ecosystem/fret-code-editor/src/editor/tests/mod.rs`
- Handle model owner: `ecosystem/fret-code-editor/src/editor/handle/model.rs`
- State schema/method owners:
  `ecosystem/fret-code-editor/src/editor/state.rs`

## Gates

```powershell
cargo fmt -p fret-code-editor --check
cargo nextest run -p fret-code-editor state_lifecycle --no-fail-fast
cargo check -p fret-code-editor --features syntax-rust
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
```
