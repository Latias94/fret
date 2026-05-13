# M3 State Methods Module Split

Status: Landed
Date: 2026-05-12

## Decision

The private `CodeEditorState` method implementation moved out of
`ecosystem/fret-code-editor/src/editor/mod.rs` into
`ecosystem/fret-code-editor/src/editor/state.rs`.

The methods are `pub(super)` because `editor/mod.rs`, `editor/paint`, and related sibling modules
still coordinate handle construction, paint, input, IME, and diagnostics call sites. This keeps the
state owner explicit without changing the public handle surface.

## Coverage

The state module now owns:

- font-stack cache invalidation,
- row-scene cache invalidation and epoch sync,
- feature-payload paint-cache invalidation,
- row cache invalidation,
- display-map refresh,
- paint-frame setup and overlay preparation,
- preedit state transitions,
- interaction-mode state transitions,
- cached IME surrounding-text lookup.

## Non-Goals

This slice does not change `CodeEditorHandle` methods, public API shape, input behavior, paint
behavior, cache semantics, diagnostics schemas, or performance thresholds. Public command/query APIs
remain available through `editor/mod.rs`'s public re-export of `CodeEditorHandle`.

## Follow-Up

The default state initializer moved from `CodeEditorHandle::new` to `CodeEditorState::new` in
`M3_STATE_INITIALIZER_BOUNDARY_2026-05-12.md`. The public handle wrapper remains in `editor/mod.rs`.

## Evidence

- State owner module: `ecosystem/fret-code-editor/src/editor/state.rs`
- Handle integration: `ecosystem/fret-code-editor/src/editor/mod.rs`
- Paint call-site coverage: `ecosystem/fret-code-editor/src/editor/paint/mod.rs`
- Public surface regression: `ecosystem/fret-code-editor/tests/public_surface.rs`
- State/paint/input behavior regression: `ecosystem/fret-code-editor/src/editor/tests/mod.rs`

## Gates

```powershell
cargo fmt -p fret-code-editor --check
cargo check -p fret-code-editor --features syntax-rust
cargo nextest run -p fret-code-editor --test public_surface --no-fail-fast
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
```
