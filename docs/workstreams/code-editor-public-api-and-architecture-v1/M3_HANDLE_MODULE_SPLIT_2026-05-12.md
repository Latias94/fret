# M3 Handle Module Split

Status: Landed
Date: 2026-05-12

## Decision

The `CodeEditorHandle` struct and its method implementation moved out of
`ecosystem/fret-code-editor/src/editor/mod.rs` into
`ecosystem/fret-code-editor/src/editor/handle.rs`.

`editor/mod.rs` keeps the public re-export so crate callers still reach `CodeEditorHandle` through
the existing crate root. The public API is unchanged; only the internal owner moved.

## Coverage

The new module owns:

- public handle construction,
- model mutation setters,
- interaction setters/readouts,
- diagnostics/perf readouts,
- feature-payload and editor action entry points,
- editor/view surface builders.

The handle field `state` is `pub(super)` so `editor/mod.rs` can still wire `CodeEditor` and the
surface builder without exposing extra public API.

## Non-Goals

This slice does not split individual handle methods into smaller modules, change public method
signatures, change default state behavior, or change any cache/perf semantics.

## Evidence

- Handle owner module: `ecosystem/fret-code-editor/src/editor/handle.rs`
- Public re-export: `ecosystem/fret-code-editor/src/editor/mod.rs`
- Public surface regression: `ecosystem/fret-code-editor/tests/public_surface.rs`
- Editor behavior regression: `ecosystem/fret-code-editor/src/editor/tests/mod.rs`

## Gates

```powershell
cargo fmt -p fret-code-editor --check
cargo check -p fret-code-editor --features syntax-rust
cargo nextest run -p fret-code-editor --test public_surface --no-fail-fast
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
```
