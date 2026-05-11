# M3 State Schema Module Split

Status: Landed
Date: 2026-05-12

## Decision

The editor state schema and cache helper types moved out of
`ecosystem/fret-code-editor/src/editor/mod.rs` into
`ecosystem/fret-code-editor/src/editor/state.rs`.

This is a module-ownership split only. `CodeEditorHandle` and the existing `CodeEditorState` method
implementations remain in `editor/mod.rs` so this slice can stay behavior-preserving and avoid a
large public-handle rewrite.

## Coverage

The new module owns:

- `CodeEditorState` field layout,
- paint-frame overlay state,
- diagnostic string-cache state,
- IME surrounding-text cache state,
- row text/cache key helper structs,
- row scene key/cache helper structs,
- baseline-measure cache state,
- syntax-only rich-row cache entry state.

Fields remain `pub(super)` because the current parent module still owns handle construction,
state-method implementations, and paint/input call sites. A later slice can move methods once the
handle boundary is ready.

## Non-Goals

This slice does not change public handle methods, cache semantics, input behavior, paint behavior,
diagnostics bundle schemas, or performance thresholds.

## Evidence

- New owner module: `ecosystem/fret-code-editor/src/editor/state.rs`
- Surface integration: `ecosystem/fret-code-editor/src/editor/mod.rs`
- Public surface regression: `ecosystem/fret-code-editor/tests/public_surface.rs`
- State/paint behavior regression: `ecosystem/fret-code-editor/src/editor/tests/mod.rs`

## Gates

```powershell
cargo fmt -p fret-code-editor --check
cargo check -p fret-code-editor --features syntax-rust
cargo nextest run -p fret-code-editor --test public_surface --no-fail-fast
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
```
