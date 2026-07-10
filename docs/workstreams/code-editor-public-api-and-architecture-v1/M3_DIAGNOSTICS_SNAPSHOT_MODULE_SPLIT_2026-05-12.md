# M3 Diagnostics Snapshot Module Split

Status: Landed
Date: 2026-05-12

## Decision

The editor diagnostics and paint-performance snapshot types moved out of
`ecosystem/fret-code-editor/src/editor/mod.rs` into
`ecosystem/fret-code-editor/src/editor/diagnostics.rs`.

The public API stays unchanged. `CodeEditorCacheStats`, `CodeEditorCacheSizeSnapshot`,
`CodeEditorMemorySnapshot`, and `CodeEditorPaintPerfFrame` remain exported from the
`fret-code-editor` crate root through `ecosystem/fret-code-editor/src/lib.rs`.

## Coverage

The new module owns:

- cache counter and getter compatibility across `syntax` and non-`syntax` builds,
- cache size and memory snapshot public structs,
- undo/redo text-byte estimate helpers,
- paint performance frame public struct,
- paint performance environment gating,
- visible-window paint-frame cache-floor helpers.

`editor/mod.rs` now consumes diagnostics helpers instead of owning the public snapshot definitions.
This keeps future perf-gate changes closer to the diagnostics owner without widening paint or state
ownership.

## Non-Goals

This slice does not change diagnostics bundle schemas, cache behavior, paint timing semantics,
public method names, or performance thresholds.

## Evidence

- New owner module: `ecosystem/fret-code-editor/src/editor/diagnostics.rs`
- Surface integration: `ecosystem/fret-code-editor/src/editor/mod.rs`
- Public surface re-export: `ecosystem/fret-code-editor/src/lib.rs`
- Public surface regression: `ecosystem/fret-code-editor/tests/public_surface.rs`

## Gates

```powershell
cargo fmt -p fret-code-editor --check
cargo check -p fret-code-editor --features syntax-rust
cargo nextest run -p fret-code-editor --test public_surface --no-fail-fast
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
```
